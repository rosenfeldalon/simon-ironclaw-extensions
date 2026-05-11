# Simon Telegram Channel

This is the intended IronClaw-native wrapper for Simon's Telegram path.

It is a custom WASM channel, not a normal tool. The baseline `0.2.10-baseline.1` clone proved that the package installs and activates as `CHANNEL` when installed with explicit `kind: "wasm_channel"`. The current `1.17` layer keeps built-in-style owner and pairing admission, persists canonical Simon family identities and per-recipient Telegram routing state for the install pack, tells verified Telegram turns about the active Family Calendar read/write contract, and supports inline keyboard callback conversations.

## Current Scope

- Preserves the built-in Telegram channel behavior for polling, webhook handling, pairing, attachments, status updates, and replies.
- Uses the custom package/install name `simon_telegram_channel`.
- Uses WIT `near:agent@0.3.0`.
- Uses the private setup secret `simon_telegram_channel_bot_token` so it does not collide with the built-in Telegram channel token.
- Sends official pairing-code instructions to unpaired private Telegram senders and does not emit those messages to the agent.
- Admits senders through built-in-style `owner_id`, `pairing_read_allow_from`, and `pairing_resolve_identity` checks.
- Restores the May 2 owner-scope invariant: runtime `user_id` must stay owner-scoped for IronClaw routing, while the private Telegram thread stays canonical as `telegram-private:<identity>`.
- Seeds the canonical Simon family registry at `state/simon_family_profiles.json` with `alon` and dormant `shlomit`.
- Resolves paired Telegram senders to canonical Simon identities where the pairing owner matches a canonical family user.
- Persists per-identity Telegram chat bindings instead of one Alon-only chat slot.
- Prepends a plain-text Simon Telegram verified-sender context to admitted messages so the agent sees the canonical sender identity.
- Sends Telegram `reply_markup.inline_keyboard` when the assistant response ends with a `<telegram_reply_markup>...</telegram_reply_markup>` control block containing callback-data buttons.
- Admits Telegram `callback_query` button presses through the same trusted sender path and emits the selected `callback_data` back to Simon as a verified Telegram turn.
- Keeps Shlomit modeled from day one, but dormant until her pairing and readiness gates pass.
- Produces an upload bundle with canonical filenames:
  - `simon_telegram_channel.wasm`
  - `simon_telegram_channel.capabilities.json`
- Marks the shared family registry and canonical per-user chat bindings as durable workspace paths so restarts can preserve the state that `simon_family_identity`, validator smokes, and recipient-aware routing expect.

## Setup Secrets

Do not put real Telegram IDs, usernames, bot tokens, or webhook secrets in this repo.

The channel setup collects only Telegram channel credentials:

- `simon_telegram_channel_bot_token`: Telegram Bot API token.
- `simon_telegram_channel_webhook_secret`: optional webhook secret. Polling mode works without it.

On a clean install, a private Telegram DM should receive a pairing code. The authenticated GUI/admin user who approves that pairing is treated as Alon for the current slice. The channel does not collect or store parent Telegram sender IDs as setup secrets.

## Build

```bash
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
```

## Install

Hosted IronClaw's URL installer expects a direct `.tar.gz` bundle URL. Build the upload bundle from the repo root:

```bash
./scripts/build-ironclaw-upload-bundles.sh
```

Publish this generated file as a GitHub Release asset or another direct HTTPS download:

```text
dist/ironclaw-upload/simon_telegram_channel.tar.gz
```

Install it through the extension URL installer with:

```text
Name: simon_telegram_channel
Kind: wasm_channel
URL: <direct HTTPS URL ending in simon_telegram_channel.tar.gz>
```

Do not use the Settings import flow for this bundle; that endpoint expects a settings JSON export.

The currently committed `1.17` source adds inline keyboard callback support. Before changing live install/preseed URLs, build and prove the bundle locally with the inline-keyboard lab:

```bash
./scripts/build-ironclaw-upload-bundles.sh
cd /Users/alonr/projects/simon-ironclaw-lab
ICLAB_SIMON_TELEGRAM_BUNDLE_URL=file:///Users/alonr/projects/simon-docs/dist/ironclaw-upload/simon_telegram_channel.tar.gz \
  iclab telegram inline-keyboard
```

The previous `1.16` bundle added the durable workspace paths for:

- `state/simon_family_profiles.json`
- `state/simon_telegram_chat_id__alon`
- `state/simon_telegram_chat_id__shlomit`

Use the public distribution bundle only after that release exists:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-owner-scope-2026-05-05-r2/bundles/simon_telegram_channel/1.15.tar.gz
```

Do not use raw GitHub URLs from private `simon-docs` for hosted installs.

For local CLI or self-hosted installs, if channel installation from a local file is available:

```bash
ironclaw channel install ./target/wasm32-wasip2/release/simon_telegram_channel.wasm \
  --capabilities ./simon-telegram.capabilities.json \
  --name simon_telegram_channel
```

If the hosted UI path is used, use the extension URL installer/API with `kind: wasm_channel`. The Settings > Channels import flow is not the raw `.tar.gz` extension-bundle installer.

## Multi-User Boundary

- Telegram private DMs remain separate parent conversations.
- Pairing and routing should target canonical Simon users such as `alon` and `shlomit`, not a single owner-scoped catch-all.
- A missing route for one parent should not block the other's outbound delivery.

## Deploy Acceptance

- Do not treat `/api/extensions`, Gateway-only smokes, or pending-pairing emptiness as deploy acceptance.
- A channel-code deploy is only accepted after reinstall if needed, Railway restart, one real Alon Telegram read smoke, and a passing `scripts/validate-simon-railway-deploy.py` run with a post-smoke timestamp.
- The validator must prove remote runtime channel loading, recent Telegram calendar-tool usage with no refusal regressions, `simon_family_identity` workspace resolution, and remote family-registry readiness without printing raw Telegram targets.
