# Simon Telegram Channel

This is the intended IronClaw-native wrapper for Simon's Telegram path.

It is a custom WASM channel, not a normal tool. The baseline `0.2.10-baseline.1` clone proved that the package installs and activates as `CHANNEL` when installed with explicit `kind: "wasm_channel"`. The current `1.4` layer preserves the built-in Telegram behavior, uses token-only setup, privately binds the first private human DM as canonical `alon`, sends a compact identity handoff, and emits a direct `/start` route fingerprint for diagnostics.

## Current Scope

- Preserves the built-in Telegram channel behavior for polling, webhook handling, pairing, attachments, status updates, and replies.
- Uses the custom package/install name `simon_telegram_channel`.
- Uses WIT `near:agent@0.3.0`.
- Uses the private setup secret `simon_telegram_channel_bot_token` so it does not collide with the built-in Telegram channel token.
- Binds the first private human Telegram sender as canonical `alon` inside channel workspace on a clean install.
- Prepends a compact `CHANNEL_CONTEXT` block to verified messages so the agent sees the canonical sender identity even if hidden channel metadata is not used in the reply.
- Sends a direct Telegram fingerprint on `/start` so we can prove the custom channel, not another Telegram integration, is handling the chat.
- Leaves Shlomit's optional sender ID empty until Alon approves pairing.
- Produces an upload bundle with canonical filenames:
  - `simon_telegram_channel.wasm`
  - `simon_telegram_channel.capabilities.json`

## Setup Secrets

Do not put real Telegram IDs, usernames, bot tokens, or webhook secrets in this repo.

The channel setup collects only Telegram channel credentials:

- `simon_telegram_channel_bot_token`: Telegram Bot API token.
- `simon_telegram_channel_webhook_secret`: optional webhook secret. Polling mode works without it.

On a clean install, the first private human sender is stored privately as canonical `alon`. Do not share the bot link before Alon sends the first DM.

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

For the current `1.4` rebuild, use the tracked raw GitHub bundle after the tag is pushed:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.4/bundles/simon_telegram_channel/1.4.tar.gz
```

For local CLI or self-hosted installs, if channel installation from a local file is available:

```bash
ironclaw channel install ./target/wasm32-wasip2/release/simon_telegram_channel.wasm \
  --capabilities ./simon-telegram.capabilities.json \
  --name simon_telegram_channel
```

If the hosted UI path is used, use the extension URL installer/API with `kind: wasm_channel`. The Settings > Channels import flow is not the raw `.tar.gz` extension-bundle installer.

## Calendar Boundary

The identity layer does not yet block Telegram calendar write intents or sanitize outbound raw tool/runtime/schema/auth errors. Keep Telegram calendar write testing stopped until the next safety layer has been added and validated.
