# Submission release report — October 27, 2026

Prepared for the owner's October 28 target, as a solo entry. This is local preparation; repository publication, YouTube upload, private connected judge access and Devpost submission remain explicit owner steps.

## Final fixes and clean source verification

The recording rehearsal found a genuine catalog search mismatch: OpenRouter returned `raspberry pi 5 board`, while the catalog stores `Raspberry Pi 5 (8GB) Single Board Computer`. Matching now requires all query tokens within a catalog field, allowing intervening words and punctuation. Model numbers and capacities remain exact token requirements. A regression test rejects Pi 4, 4GB, Pi 50 and an empty punctuation-only query against the Pi 5 8GB product.

The first clean `npm ci` exposed a malformed optional `@emnapi/runtime` lockfile entry without a version. The entry was removed and the lockfile regenerated using npm metadata. A clean source snapshot then installed 367 packages successfully. Lint and the production TypeScript/Next.js build passed. Rust `cargo check --locked --offline` and all **35 tests** passed from the release source copy, using the cached dependency target directory. This was a source build check, not an independent fresh download of every Rust crate.

The release is produced from an explicit file allowlist. It excludes `.env`, actual payment history, browser profiles, verification scripts, binaries, dependencies and legacy Python prototypes. `backend/data/transactions.json` is copied from the immutable 25-record seed. The ZIP inventory and SHA-256 fingerprints are supplied in the local `submission/release-manifest.json`. Exact hashes and ZIP contents are checked after packaging.

## Recorded connected demonstration

An additional actual Sandbox transaction was created for the video; earlier records were preserved:

| Field | Result |
| --- | --- |
| Local order | ORD-4ed59f73c71b4caf9404 |
| Mandate model | openrouter:recursive/flint-v1.0:free |
| Purchase | One Raspberry Pi 5 board from DigiKey; 78.50 USD |
| Capture | 8V126807U41618146; COMPLETED |
| Payout batch | XXU6Y4X7G6DP8; matching successful recipient item reconciled |
| Final application state | SETTLED |
| Budget test | Two boards under 50 USD; BLOCKED_BY_GUARDRAIL; no PayPal order |
| Budget-test model | openrouter:nvidia/nemotron-3.5-super-vl:free |
| Treasury question | Find Mouser orders costing less than $600. |
| Treasury model | openrouter:dots-studio/dots-3-note-preview:free |
| Treasury result | Four independently checked records; vendor Mouser AND amount below 600 |

One free-router request returned HTTP 429 and the application correctly identified fallback parsing. A subsequent actual model request passed and supplies the final query footage. The last read-only recording also checked that the payment ledger did not change. Free-router availability remains variable and dedicated judge capacity is an owner setup item.

The final video is 159.55 seconds (about 2:40) at 1920×1080. It uses original working-application footage, synthetic English narration, no music, and shortened waiting time. Some frames are held to explain the result. The PayPal approval popup and capture/payout footage refer to the same new Sandbox purchase; the treasury query and read-only audit close-ups were recorded separately. Credentials were entered before the payment recording and are excluded from media. Genuine webhook delivery remains unverified. The catalog remains illustrative and no physical goods were ordered.

All 12 Chrome checks passed again after the release fixes, including responsive purchase flows, policy rejection, payout reconciliation, CSV provenance and the SDK callback contract. They use isolated simulation data and fixture callbacks; actual provider proof is recorded separately above. A before/after SHA-256 check confirmed that this regression suite left the owner's actual Sandbox ledger unchanged. The configured Sandbox service was restored afterward.

## External completion

The entry is not submitted. Follow the local `submission/PUBLISH.md` and `FINAL_CHECKLIST.md` to publish the reviewed source, upload the public video, provide connected test access and submit the Devpost form. Maintain free judge access through December 15, 2026. [Official requirements](https://paypalaihackathon.devpost.com/rules).
