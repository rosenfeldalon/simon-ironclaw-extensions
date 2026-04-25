# Simon IronClaw Extensions

Public distribution repo for Simon-specific IronClaw WASM channels and tools.

This repo intentionally contains only distributable extension source, release bundles, and install notes. Private Simon docs, prompts, logs, setup values, Telegram sender IDs, bot tokens, Google Calendar IDs, OAuth details, and household notes belong in the private `simon-docs` repo.

## Extensions

- `extensions/simon-telegram-channel/`: custom IronClaw Telegram channel package named `simon_telegram_channel`.

## Latest Bundle

The latest public bundle is:

```text
bundles/simon_telegram_channel/1.7.tar.gz
```

After pushing tag `ironclaw-simon-telegram-1.7`, the direct install URL is:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.7/bundles/simon_telegram_channel/1.7.tar.gz
```

Important: `1.7` fixes the custom channel webhook route to `/webhook/simon_telegram_channel`, matching IronClaw's WASM channel route registration. It passed the local Simon IronClaw Lab fake Telegram plus Ollama comparison against the public HTTPS bundle. It is still accepted for production only after hosted Telegram pairing and identity-continuity smoke tests pass.

Install through IronClaw's extension URL installer/API with explicit channel kind:

```json
{
  "name": "simon_telegram_channel",
  "url": "https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.7/bundles/simon_telegram_channel/1.7.tar.gz",
  "kind": "wasm_channel"
}
```

Do not use the Settings import flow for this `.tar.gz`; that path is for settings imports, not raw WASM channel bundles.

## Build

```bash
rustup target add wasm32-wasip2
cargo fmt --check && cargo test --manifest-path extensions/simon-telegram-channel/Cargo.toml
IRONCLAW_SIMON_BUNDLE_VERSION=1.7 ./scripts/build-ironclaw-upload-bundles.sh
```

The build script writes:

```text
dist/ironclaw-upload/simon_telegram_channel.tar.gz
bundles/simon_telegram_channel/<version>.tar.gz
```

## Public Safety Rule

Use placeholders only. Do not commit real Telegram IDs, usernames, bot tokens, webhook secrets, calendar IDs, OAuth details, or private Simon family context.

## Release Verification

Before sharing an install URL, verify the pushed raw GitHub URL returns `200` and inspect the packaged capabilities JSON for:

- `name: "simon_telegram_channel"`
- `version: "1.7"`
- `type: "channel"`
- `wit_version: "0.3.0"`

Hosted installs must use public URLs from this repo. Do not use raw GitHub URLs from the private `simon-docs` repo.

Raw URL and capabilities checks are necessary release checks, but not success criteria. A release is accepted only after the real hosted Telegram transcript shows the expected pairing handshake before approval and durable Simon identity/context after approval.

## Diagnostic Context

The reusable lab at `/Users/alonr/projects/simon-ironclaw-lab` compares the built-in `telegram` channel and this custom `simon_telegram_channel` under the same fake Telegram pairing scenario.

That lab found the `1.6` failure: the custom channel still used the built-in `/webhook/telegram` route, while IronClaw registers custom WASM channels at `/webhook/{channel_name}`. Version `1.7` aligns source, capabilities, and bundle metadata on `/webhook/simon_telegram_channel`.
