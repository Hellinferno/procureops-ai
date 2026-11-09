import { chromium } from "@playwright/test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdir, copyFile, writeFile, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const frontend = fileURLToPath(new URL("..", import.meta.url));
const root = resolve(frontend, "..");
const artifacts = resolve(root, ".verification");
const data = resolve(artifacts, "browser-data");
const api = "http://localhost:8000";
const site = "http://127.0.0.1:13000";
const children = [];
const checks = [];
let browser;
let output = "";
const delay = (ms) => new Promise((done) => setTimeout(done, ms));
function start(file, args, env) {
  const child = spawn(file, args, { cwd: root, windowsHide: true, env: { ...process.env, ...env }, stdio: ["ignore", "pipe", "pipe"] });
  child.stdout.on("data", (data) => { output += data.toString(); });
  child.stderr.on("data", (data) => { output += data.toString(); });
  children.push(child);
  return child;
}
async function waitFor(url) {
  for (let attempt = 0; attempt < 120; attempt++) {
    try { if ((await fetch(url, { signal: AbortSignal.timeout(1000) })).ok) return; } catch {}
    await delay(250);
  }
  throw new Error(`Service did not start: ${url}\n${output.slice(-4000)}`);
}
async function ledger() { return (await (await fetch(`${api}/api/transactions`)).json()).transactions; }
async function lastPurchase() { return (await ledger()).at(-1); }
async function check(name, action) { await action(); checks.push(name); console.log(`PASS: ${name}`); }

try {
  await mkdir(data, { recursive: true });
  await mkdir(resolve(artifacts, "screenshots"), { recursive: true });
  for (const endpoint of [`${api}/api/health`, site]) {
    let occupied = false;
    try { await fetch(endpoint, { signal: AbortSignal.timeout(800) }); occupied = true; } catch {}
    assert.equal(occupied, false, `Port already in use: ${endpoint}. Stop that service before running verification.`);
  }
  for (const name of ["catalog.json", "seed_transactions.json"]) await copyFile(resolve(root, "backend/data", name), resolve(data, name));
  await copyFile(resolve(data, "seed_transactions.json"), resolve(data, "transactions.json"));
  start(resolve(root, "backend/target/debug/procureops-backend.exe"), [], {
    PAYPAL_MODE: "simulation", PAYPAL_CLIENT_ID: "", PAYPAL_CLIENT_SECRET: "",
    LLM_PROVIDER: "openai", OPENAI_API_KEY: "", OPENROUTER_API_KEY: "", LLM_MODEL: "",
    PORT: "8000", BIND_HOST: "127.0.0.1", DATA_DIR: data, FRONTEND_ORIGINS: site,
  });
  start(process.execPath, [resolve(frontend, "node_modules/next/dist/bin/next"), "start", frontend, "--port", "13000"], {});
  await waitFor(`${api}/api/health`); await waitFor(site);
  browser = await chromium.launch({ channel: "chrome", headless: true });
  const page = await browser.newPage({ viewport: { width: 1512, height: 1100 } });
  const pageErrors = [];
  page.on("pageerror", (error) => pageErrors.push(String(error)));
  await page.goto(site);
  await page.getByText("25 records", { exact: true }).waitFor();
  await check("Ledger loads, simulation is visible, and seed rows have no payout action", async () => {
    await page.getByText("Simulation · no money moves", { exact: true }).waitFor();
    assert.equal(await page.getByRole("button", { name: "PayPal Payout", exact: true }).count(), 0);
  });
  await page.screenshot({ path: resolve(artifacts, "screenshots/cockpit.png"), fullPage: true });
  await check("Mandate prepares an order, then explicit approval captures it", async () => {
    await page.getByRole("button", { name: "Prepare checkout", exact: true }).click();
    const approval = page.getByRole("button", { name: "Simulate buyer approval & capture", exact: true });
    await approval.waitFor();
    assert.equal((await lastPurchase()).status, "AWAITING_APPROVAL");
    await approval.click();
    await page.getByText(/Capture ID: SIM-CAP-/).waitFor();
    assert.equal((await lastPurchase()).status, "COMPLETED");
  });
  await check("Payout acceptance stays pending until its status is checked", async () => {
    const transaction = await lastPurchase();
    // Search the new ID so AG Grid brings its action into the visible page.
    await page.getByPlaceholder("Search purchases...").fill(transaction.order_id);
    await page.getByRole("button", { name: "PayPal Payout", exact: true }).click();
    const refresh = page.getByRole("button", { name: "Check payout", exact: true });
    await refresh.waitFor();
    assert.equal(await page.locator("dialog[open]").count(), 0, "Payout actions must not open the row inspector");
    assert.equal((await lastPurchase()).status, "PAYOUT_PENDING");
    await refresh.click();
    await page.getByRole("region", { name: "Purchase ledger" }).getByText("Settled", { exact: true }).waitFor();
    assert.equal((await lastPurchase()).status, "SETTLED");
    await page.getByPlaceholder("Search purchases...").fill("");
  });
  await check("Budget breach and rogue vendor are blocked before payment", async () => {
    for (const preset of ["Over budget", "Unapproved supplier"]) {
      await page.getByRole("button", { name: new RegExp(preset) }).click();
      const completed = page.waitForResponse((response) => response.url().endsWith("/api/mandate") && response.request().method() === "POST");
      await page.getByRole("button", { name: "Prepare checkout", exact: true }).click();
      await completed;
      const details = page.locator(".execution-details");
      if (!(await details.evaluate(element => element.open))) await details.locator("summary").click();
      await page.getByText("Stage 4: PayPal checkout", { exact: true }).waitFor();
      assert.equal((await lastPurchase()).status, "BLOCKED_BY_GUARDRAIL");
      assert.equal((await lastPurchase()).paypal_order_id, null);
    }
  });
  await check("Treasury query filters the ledger and identifies its parser", async () => {
    await page.getByPlaceholder(/Ask your treasury/).fill("Show all blocked orders or orders over $500");
    await page.getByRole("button", { name: "Ask Treasury", exact: true }).click();
    await page.getByRole("region", { name: "Purchase ledger" }).getByText("Parser: deterministic", { exact: true }).waitFor();
  });
  await page.screenshot({ path: resolve(artifacts, "screenshots/verified-workflow.png"), fullPage: true });
  await check("Purchase views, policy panel and native audit inspector work", async () => {
    await page.getByRole("tab", { name: "Needs approval", exact: true }).click();
    await page.getByText("0 records", { exact: true }).waitFor();
    await page.getByRole("tab", { name: "All purchases", exact: true }).click();
    await page.getByText(String((await ledger()).length) + " records", { exact: true }).waitFor();
    const allTab = page.getByRole("tab", { name: "All purchases", exact: true });
    await allTab.focus(); await page.keyboard.press("ArrowRight");
    assert.equal(await page.getByRole("tab", { name: "Needs approval", exact: true }).getAttribute("aria-selected"), "true");
    await page.keyboard.press("Home");
    assert.equal(await allTab.getAttribute("aria-selected"), "true");
    await page.getByRole("button", { name: "Policy controls", exact: true }).click();
    const policy = page.getByRole("dialog", { name: "Your purchasing policy" });
    await policy.waitFor(); await policy.getByText("$2,000.00", { exact: true }).waitFor();
    await page.keyboard.press("Escape"); await policy.waitFor({ state: "hidden" });
    await page.getByRole("button", { name: /^Inspect / }).first().click();
    const inspector = page.getByRole("dialog", { name: "Purchase details" });
    await inspector.waitFor(); await page.keyboard.press("Escape"); await inspector.waitFor({ state: "hidden" });
  });
  await check("CSV export includes the current ledger and payment provenance", async () => {
    const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "Export CSV", exact: true }).click()]);
    const csv = await readFile(await download.path(), "utf8");
    assert.ok(csv.includes('"Execution mode"'));
    assert.equal(csv.split("\r\n").length, (await ledger()).length + 1);
  });
  await check("Mobile, tablet and compact desktop layouts contain page overflow", async () => {
    for (const width of [390, 768, 1024]) {
      await page.setViewportSize({ width, height: 1000 }); await page.goto(site);
      await page.getByText(String((await ledger()).length) + " records", { exact: true }).waitFor();
      await page.evaluate(() => document.fonts.ready);
      const widthCheck = await page.evaluate(() => ({ page: document.documentElement.scrollWidth, viewport: innerWidth }));
      assert.ok(widthCheck.page <= widthCheck.viewport + 1, "Page overflow at " + width + ": " + JSON.stringify(widthCheck));
      await page.screenshot({ path: resolve(artifacts, "screenshots/redesign-" + width + ".png"), fullPage: true });
    }
  });
  await check("Mobile navigation closes and New purchase focuses the request", async () => {
    await page.setViewportSize({ width: 390, height: 900 }); await page.goto(site);
    await page.getByRole("button", { name: "Open navigation", exact: true }).click();
    await page.getByRole("navigation", { name: "Main navigation" }).getByRole("button", { name: "All purchases", exact: true }).click();
    assert.equal(await page.getByRole("button", { name: "Open navigation", exact: true }).getAttribute("aria-expanded"), "false");
    await page.getByRole("button", { name: "New purchase", exact: true }).click();
    assert.equal(await page.locator("#purchase-prompt").evaluate(element => element === document.activeElement), true);
  });
  await check("Phone purchase cards support search, capture, payout and inspection", async () => {
    const latest = (await ledger()).sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp))[0];
    assert.equal(await page.locator(".purchase-card-main").first().getAttribute("aria-label"), "Inspect " + latest.order_id);
    await page.locator("#purchase-prompt").fill("Buy 1 Raspberry Pi 5 under $100 within 3 days");
    await page.locator("#purchase-budget").fill("100");
    await page.getByRole("button", { name: "Prepare checkout", exact: true }).click();
    await page.getByRole("button", { name: "Simulate buyer approval & capture", exact: true }).click();
    await page.getByText(/Capture ID: SIM-CAP-/).waitFor();
    const current = await lastPurchase();
    await page.getByPlaceholder("Search purchases...").fill(current.order_id);
    const cards = page.getByLabel("Purchase cards", { exact: true });
    await cards.getByRole("button", { name: "PayPal Payout", exact: true }).click();
    await cards.getByRole("button", { name: "Check payout", exact: true }).click();
    await cards.getByText("Settled", { exact: true }).waitFor();
    assert.equal((await lastPurchase()).status, "SETTLED");
    await cards.getByRole("button", { name: "Inspect " + current.order_id, exact: true }).click();
    const inspector = page.getByRole("dialog", { name: "Purchase details" });
    await inspector.waitFor(); await page.keyboard.press("Escape"); await inspector.waitFor({ state: "hidden" });
    await page.getByPlaceholder("Search purchases...").fill("");
    const before = await cards.locator(".purchase-card-main").first().getAttribute("aria-label");
    await page.getByRole("button", { name: "Next purchases", exact: true }).click();
    const after = await cards.locator(".purchase-card-main").first().getAttribute("aria-label");
    assert.notEqual(before, after);
    await page.getByRole("button", { name: "Previous purchases", exact: true }).click();
    assert.equal(await page.locator(".ag-row").count(), 0);
    await page.evaluate(() => window.scrollTo(0, 0));
    await page.screenshot({ path: resolve(artifacts, "screenshots/redesign-phone-workflow.png"), fullPage: true });
  });
  await check("No uncaught browser errors", async () => assert.deepEqual(pageErrors, []));

  // Exercise the v6 callback contract without claiming a real PayPal session.
  const sdkPage = await browser.newPage({ viewport: { width: 1512, height: 1100 } });
  let sandboxOrder;
  let capturedByCallback = false;
  const cors = { "access-control-allow-origin": site, "access-control-allow-methods": "GET, POST, OPTIONS", "access-control-allow-headers": "content-type" };
  await sdkPage.route("**/api/paypal/config", (route) => route.fulfill({ headers: cors, json: {
    mode: "sandbox", configured: true, client_id: "test-client-id", sdk_url: "https://www.sandbox.paypal.com/web-sdk/v6/core",
    payouts_enabled: false, webhook_configured: false, message: "Sandbox SDK contract test",
  } }));
  await sdkPage.route("https://www.sandbox.paypal.com/web-sdk/v6/core", (route) => route.fulfill({ contentType: "text/javascript", body: `
    window.paypal = { createInstance: async (options) => {
      window.sdkOptions = options;
      return { findEligibleMethods: async () => ({ isEligible: () => true }),
        createPayPalOneTimePaymentSession: async (callbacks) => ({
          start: async (_, order) => { const data = await order; await callbacks.onApprove(data); }
        }) };
    } };
    customElements.define('paypal-button', class extends HTMLElement { connectedCallback() { this.textContent = 'Approve SDK test payment'; this.setAttribute('role','button'); this.tabIndex=0; } });
  ` }));
  await sdkPage.route("**/api/mandate", async (route) => {
    const response = await route.fetch(); const result = await response.json();
    sandboxOrder = { ...result.transaction, payment_mode: "sandbox", paypal_order_id: "SDK-TEST-ORDER" };
    await route.fulfill({ headers: cors, json: { ...result, transaction: sandboxOrder } });
  });
  await sdkPage.route("**/api/paypal/capture", async (route) => {
    assert.equal(route.request().postDataJSON().order_id, sandboxOrder.order_id);
    capturedByCallback = true;
    await route.fulfill({ headers: cors, json: { success: true, transaction: { ...sandboxOrder, status: "COMPLETED", capture_id: "SDK-TEST-CAPTURE" }, paypal_raw: {}, message: "SDK callback confirmed" } });
  });
  await check("JavaScript SDK v6 initializes and onApprove calls server capture", async () => {
    await sdkPage.goto(site); await sdkPage.getByRole("heading", { name: "Purchasing overview" }).waitFor();
    await sdkPage.getByRole("button", { name: "Prepare checkout", exact: true }).click();
    await sdkPage.getByText(/Sandbox SDK contract test/).waitFor();
    await sdkPage.getByRole("button", { name: "Approve PayPal Sandbox payment", exact: true }).click();
    await sdkPage.getByText(/Capture ID: SDK-TEST-CAPTURE/).waitFor();
    assert.equal(capturedByCallback, true);
    assert.deepEqual(await sdkPage.evaluate(() => window.sdkOptions), { clientId: "test-client-id", components: ["paypal-payments"] });
  });
  await writeFile(resolve(artifacts, "browser-results.json"), JSON.stringify({ checks, real_paypal_verified: false, real_ai_verified: false }, null, 2));
  console.log(`Verified ${checks.length} browser checks. Artifacts: ${artifacts}`);
} catch (error) {
  console.error(error);
  await writeFile(resolve(artifacts, "browser-failure.log"), `${String(error)}\n${output}`);
  process.exitCode = 1;
} finally {
  await browser?.close();
  for (const child of children.reverse()) {
    if (child.exitCode === null && child.pid) {
      if (process.platform === "win32") {
        await new Promise((done) => spawn("taskkill", ["/PID", String(child.pid), "/T", "/F"], { windowsHide: true, stdio: "ignore" }).on("close", done));
      } else child.kill("SIGTERM");
    }
  }
}
