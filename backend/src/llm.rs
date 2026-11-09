//! Server-only structured model calls. Production credentials are sent only to
//! the selected provider's fixed HTTPS endpoint; provider failures stay visible
//! through the caller's deterministic_fallback provenance.

use std::{sync::LazyLock, time::Duration};

use serde_json::{json, Value};
use tracing::warn;

static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(45))
        .build()
        .expect("failed to build model HTTP client")
});

#[derive(Clone, Copy, PartialEq, Eq)]
enum Provider {
    OpenAi,
    OpenRouter,
}

impl Provider {
    fn name(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::OpenRouter => "openrouter",
        }
    }

    fn endpoint(self) -> &'static str {
        match self {
            Self::OpenAi => "https://api.openai.com/v1/chat/completions",
            Self::OpenRouter => "https://openrouter.ai/api/v1/chat/completions",
        }
    }
}

// No Debug implementation: api_key must never appear in diagnostics.
struct Config {
    provider: Provider,
    model: String,
    api_key: String,
}

fn nonempty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn credential(value: Option<String>) -> Option<String> {
    nonempty(value).filter(|value| {
        !value.starts_with("your_") && !value.starts_with("your-") && !value.starts_with("sk-your")
    })
}

impl Config {
    fn from_values(
        provider: Option<String>,
        model: Option<String>,
        openai_key: Option<String>,
        openrouter_key: Option<String>,
    ) -> Result<Self, &'static str> {
        let openai_key = credential(openai_key);
        let openrouter_key = credential(openrouter_key);
        let provider = match nonempty(provider).as_deref().map(str::to_lowercase) {
            Some(value) if value == "openai" => Provider::OpenAi,
            Some(value) if value == "openrouter" => Provider::OpenRouter,
            Some(_) => return Err("Unsupported LLM_PROVIDER"),
            None if openai_key.is_none() && openrouter_key.is_some() => Provider::OpenRouter,
            None => Provider::OpenAi,
        };
        let (api_key, default_model) = match provider {
            Provider::OpenAi => (openai_key, "gpt-4o-mini"),
            Provider::OpenRouter => (openrouter_key, "openrouter/free"),
        };
        Ok(Self {
            provider,
            model: nonempty(model).unwrap_or_else(|| default_model.into()),
            api_key: api_key.ok_or("Selected model provider has no API key")?,
        })
    }

    fn from_env() -> Result<Self, &'static str> {
        Self::from_values(
            std::env::var("LLM_PROVIDER").ok(),
            std::env::var("LLM_MODEL").ok(),
            std::env::var("OPENAI_API_KEY").ok(),
            std::env::var("OPENROUTER_API_KEY").ok(),
        )
    }
}

pub fn configured() -> bool {
    Config::from_env().is_ok()
}

pub struct StructuredOutput {
    pub value: Value,
    pub parser_mode: String,
}

fn decode(payload: Value, config: &Config) -> Result<StructuredOutput, &'static str> {
    if payload.get("error").is_some() {
        return Err("Model provider returned an error envelope");
    }
    if payload
        .pointer("/choices/0/finish_reason")
        .and_then(Value::as_str)
        == Some("length")
    {
        return Err("Model output was truncated");
    }
    let content = payload
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or("Model response has no text content")?;
    let value: Value =
        serde_json::from_str(content).map_err(|_| "Model output is not valid JSON")?;
    if !value.is_object() {
        return Err("Model output is not a JSON object");
    }
    // A router alias is not evidence of the model selected. Use response metadata.
    let model = payload
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .unwrap_or("unreported");
    Ok(StructuredOutput {
        value,
        parser_mode: format!("{}:{model}", config.provider.name()),
    })
}

async fn request(
    client: &reqwest::Client,
    config: &Config,
    endpoint: &str,
    system: &str,
    prompt: &str,
) -> Result<StructuredOutput, String> {
    let mut body = json!({
        "model": config.model,
        "temperature": 0.0,
        "response_format": {"type": "json_object"},
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]
    });
    if config.provider == Provider::OpenRouter {
        body["provider"] = json!({"require_parameters": true});
        body["max_tokens"] = json!(4096);
    }
    let response = client
        .post(endpoint)
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| {
            if error.is_timeout() {
                "Model request timed out"
            } else {
                "Model provider request failed"
            }
            .to_string()
        })?;
    if !response.status().is_success() {
        // Do not log provider bodies: they may echo prompts or credentials.
        return Err(format!(
            "Model provider returned HTTP {}",
            response.status()
        ));
    }
    let payload = response
        .json::<Value>()
        .await
        .map_err(|_| "Model response body is not valid JSON".to_string())?;
    decode(payload, config).map_err(str::to_string)
}

pub async fn structured_json(system: &str, prompt: &str) -> Option<StructuredOutput> {
    let config = Config::from_env().ok()?;
    match request(&HTTP, &config, config.provider.endpoint(), system, prompt).await {
        Ok(output) => Some(output),
        Err(error) => {
            warn!(provider = config.provider.name(), "{error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        http::{HeaderMap, StatusCode},
        routing::post,
        Json, Router,
    };

    fn router_config() -> Config {
        Config::from_values(
            Some("openrouter".into()),
            None,
            None,
            Some("test-router-key".into()),
        )
        .ok()
        .unwrap()
    }

    #[test]
    fn provider_selection_keeps_credentials_separate() {
        let config = router_config();
        assert_eq!(config.provider.name(), "openrouter");
        assert_eq!(
            config.provider.endpoint(),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(config.model, "openrouter/free");
        assert!(Config::from_values(
            Some("openrouter".into()),
            None,
            Some("openai-only-key".into()),
            None
        )
        .is_err());
        assert!(
            Config::from_values(Some("other".into()), None, None, Some("test-key".into())).is_err()
        );
        assert!(Config::from_values(
            Some("openrouter".into()),
            None,
            None,
            Some("your_key_here".into())
        )
        .is_err());
    }

    #[test]
    fn legacy_openai_and_router_only_configuration_are_supported() {
        let legacy = Config::from_values(None, None, Some("test-openai-key".into()), None)
            .ok()
            .unwrap();
        assert_eq!(legacy.provider.name(), "openai");
        assert_eq!(legacy.model, "gpt-4o-mini");
        let router = Config::from_values(
            None,
            Some("vendor/chosen-model".into()),
            None,
            Some("test-key".into()),
        )
        .ok()
        .unwrap();
        assert_eq!(router.provider.name(), "openrouter");
        assert_eq!(router.model, "vendor/chosen-model");
    }

    #[test]
    fn provenance_uses_resolved_model_and_rejects_unusable_output() {
        let config = router_config();
        let payload = json!({"model":"vendor/selected-model:free","choices":[{"message":{"content":"{\"kind\":\"all\"}"},"finish_reason":"stop"}]});
        let output = decode(payload.clone(), &config).ok().unwrap();
        assert_eq!(output.parser_mode, "openrouter:vendor/selected-model:free");
        let mut missing = payload.clone();
        missing.as_object_mut().unwrap().remove("model");
        assert_eq!(
            decode(missing, &config).ok().unwrap().parser_mode,
            "openrouter:unreported"
        );
        for bad in [
            json!({"error":{"message":"not a completion"}}),
            json!({"choices":[{"message":{"content":"not json"}}]}),
            json!({"choices":[{"message":{"content":"[]"}}]}),
            json!({"choices":[{"message":{"content":"{}"},"finish_reason":"length"}]}),
            json!({"choices":[{"message":{"content":null}}]}),
        ] {
            assert!(decode(bad, &config).is_err());
        }
    }

    #[tokio::test]
    async fn openrouter_contract_sends_json_mode_and_records_selected_model() {
        let app = Router::new().route("/chat", post(|headers: HeaderMap, Json(body): Json<Value>| async move {
            assert_eq!(headers["authorization"], "Bearer test-router-key");
            assert_eq!(body["model"], "openrouter/free");
            assert_eq!(body["response_format"]["type"], "json_object");
            assert_eq!(body["provider"]["require_parameters"], true);
            assert_eq!(body["messages"][1]["content"], "Need Raspberry Pi boards");
            Json(json!({"model":"vendor/actual-free-model","choices":[{"message":{"content":"{\"quantity\":2}"}}]}))
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}/chat", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let output = request(
            &client,
            &router_config(),
            &endpoint,
            "Return JSON",
            "Need Raspberry Pi boards",
        )
        .await;
        task.abort();
        let output = output.unwrap();
        assert_eq!(output.value["quantity"], 2);
        assert_eq!(output.parser_mode, "openrouter:vendor/actual-free-model");
    }

    #[tokio::test]
    async fn provider_failure_does_not_expose_response_body() {
        let app = Router::new().route(
            "/chat",
            post(|| async { (StatusCode::TOO_MANY_REQUESTS, "sensitive-provider-body") }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}/chat", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let output = request(&client, &router_config(), &endpoint, "Return JSON", "test").await;
        task.abort();
        let error = output.err().unwrap();
        assert!(error.contains("429"));
        assert!(!error.contains("sensitive-provider-body"));
        assert!(!error.contains("test-router-key"));
    }
}
