# Simon Telegram Channel

This is the intended IronClaw-native wrapper for Simon's Telegram path.

It is a custom WASM channel, not a normal tool. The baseline `0.2.10-baseline.1` clone proved that the package installs and activates as `CHANNEL` when installed with explicit `kind: "wasm_channel"`. The current `1.12` layer keeps the `1.11` admission/safety behavior, but fixes the runtime ownership boundary so admitted Telegram turns use the resolved owner scope for workspace/tool/secrets access while preserving Simon's canonical family identity in the Telegram handoff and private thread namespace.

## Current Scope

- Preserves the built-in Telegram channel behavior for polling, webhook handling, pairing, attachments, status updates, and replies.
- Uses the custom package/install name `simon_telegram_channel`.
- Uses `/webhook/simon_telegram_channel`, matching IronClaw's installed WASM channel route.
- Uses WIT `near:agent@0.3.0`.
- Uses the private setup secret `simon_telegram_channel_bot_token` so it does not collide with the built-in Telegram channel token.
- Sends official pairing-code instructions to unpaired private Telegram senders and does not emit those messages to the agent.
- Logs ignored unapproved private/group messages with redacted admission metadata and without raw Telegram IDs, usernames, tokens, or message content.
- Admits senders through built-in-style `owner_id`, `pairing_read_allow_from`, and `pairing_resolve_identity` checks.
- Treats an admitted sender as canonical `alon` with role `primary_parent_admin` for this slice.
- Uses the resolved pairing owner scope as the runtime `user_id` when available, so Telegram-originated turns land in the same owner workspace/tool domain as the web gateway.
- Prepends a plain-text Simon Telegram verified-sender context to admitted messages so the agent sees the canonical sender identity, Simon-vs-IronClaw assistant boundary, and the expected `simon_google_calendar` JSON action shape.
- Keeps Shlomit modeled for a future explicit onboarding path, but unpaired/TBD until Alon explicitly approves her.
- Sanitizes obvious raw tool/runtime/schema/auth errors before sending Telegram replies.
- Suppresses auth-required/completed status cards in Telegram so Google Calendar setup stays in the GUI/admin surface.
- Uses `telegram-private:safety-2:<identity>` for private threads to avoid stale Telegram engine context from earlier channel versions.
- Produces an upload bundle with canonical filenames:
  - `simon_telegram_channel.wasm`
  - `simon_telegram_channel.capabilities.json`

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

For the current `1.12` rebuild, use the public distribution bundle:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.12/bundles/simon_telegram_channel/1.12.tar.gz
```

Do not use raw GitHub URLs from private `simon-docs` for hosted installs.

The `1.12` bundle includes the `1.11` route/admission/thread-context behavior and adds the owner-scope runtime-user fix for paired Telegram senders. If hosted logs do not show `Simon Telegram channel runtime version 1.12`, IronClaw is still running an older channel artifact or active runtime.

For local CLI or self-hosted installs, if channel installation from a local file is available:

```bash
ironclaw channel install ./target/wasm32-wasip2/release/simon_telegram_channel.wasm \
  --capabilities ./simon-telegram.capabilities.json \
  --name simon_telegram_channel
```

If the hosted UI path is used, use the extension URL installer/API with `kind: wasm_channel`. The Settings > Channels import flow is not the raw `.tar.gz` extension-bundle installer.

## Calendar Boundary

The identity layer now sanitizes obvious outbound raw tool/runtime/schema/auth errors and gives admitted Telegram turns enough context to call `simon_google_calendar` with the `family` alias. Calendar write testing remains approval-gated by the active slice.
