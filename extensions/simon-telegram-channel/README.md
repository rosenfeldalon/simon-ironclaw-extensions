# Simon Telegram Channel

This is the intended IronClaw-native wrapper for Simon's Telegram path.

It is a custom WASM channel, not a normal tool. The baseline `0.2.10-baseline.1` clone proved that the package installs and activates as `CHANNEL` when installed with explicit `kind: "wasm_channel"`. The current `1.9` layer restores built-in-style owner/pairing admission, adds Simon identity context for admitted Telegram senders, uses the custom channel webhook route, logs hosted admission decisions, ignores unapproved messages before agent emission, and sanitizes outbound raw internal errors.

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
- Prepends a plain-text Simon Telegram verified-sender context to admitted messages so the agent sees the canonical sender identity.
- Keeps Shlomit modeled for a future explicit onboarding path, but unpaired/TBD until Alon explicitly approves her.
- Sanitizes obvious raw tool/runtime/schema/auth errors before sending Telegram replies.
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

For the current `1.9` rebuild, use the public distribution bundle:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.9/bundles/simon_telegram_channel/1.9.tar.gz
```

Do not use raw GitHub URLs from private `simon-docs` for hosted installs.

The `1.9` bundle includes the `1.7` route fix, `1.8` redacted hosted diagnostics, unapproved-message ignore logging, future-ready Shlomit identity structure, and outbound raw-error sanitization. If hosted logs do not show `Simon Telegram channel runtime version 1.9`, IronClaw is still running an older channel artifact or active runtime.

For local CLI or self-hosted installs, if channel installation from a local file is available:

```bash
ironclaw channel install ./target/wasm32-wasip2/release/simon_telegram_channel.wasm \
  --capabilities ./simon-telegram.capabilities.json \
  --name simon_telegram_channel
```

If the hosted UI path is used, use the extension URL installer/API with `kind: wasm_channel`. The Settings > Channels import flow is not the raw `.tar.gz` extension-bundle installer.

## Calendar Boundary

The identity layer now sanitizes obvious outbound raw tool/runtime/schema/auth errors, but Telegram calendar write testing remains stopped until identity/context behavior is validated in a real hosted transcript.
