# Simon Telegram Channel

This is the intended IronClaw-native wrapper for Simon's Telegram path.

It is a custom WASM channel, not a normal tool. The baseline `0.2.10-baseline.1` clone proved that the package installs and activates as `CHANNEL` when installed with explicit `kind: "wasm_channel"`. The current `1.5` layer preserves token-only setup and uses IronClaw's official pairing store as the identity boundary before emitting canonical Simon identity.

## Current Scope

- Preserves the built-in Telegram channel behavior for polling, webhook handling, pairing, attachments, status updates, and replies.
- Uses the custom package/install name `simon_telegram_channel`.
- Uses WIT `near:agent@0.3.0`.
- Uses the private setup secret `simon_telegram_channel_bot_token` so it does not collide with the built-in Telegram channel token.
- Sends official pairing-code instructions to unpaired private Telegram senders and does not emit those messages to the agent.
- Resolves paired senders with `pairing_resolve_identity("simon_telegram_channel", <telegram_user_id>)`.
- Treats a successfully paired sender as canonical `alon` with role `primary_parent_admin` for this slice.
- Prepends a compact `CHANNEL_CONTEXT` block only to pairing-verified messages so the agent sees the canonical sender identity even if hidden channel metadata is not used in the reply.
- Leaves Shlomit unpaired/TBD until Alon explicitly approves a future path.
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

For the current `1.5` rebuild, use the public distribution bundle after the tag is pushed:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.5/bundles/simon_telegram_channel/1.5.tar.gz
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
