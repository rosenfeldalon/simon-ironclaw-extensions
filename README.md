# Simon IronClaw Extensions

Public distribution repo for Simon-specific IronClaw WASM channels and tools.

This repo intentionally contains only distributable extension source, release bundles, and install notes. Private Simon docs, prompts, logs, setup values, Telegram sender IDs, bot tokens, Google Calendar IDs, OAuth details, and household notes belong in the private `simon-docs` repo.

## Extensions

- `extensions/simon-telegram-channel/`: custom IronClaw Telegram channel package named `simon_telegram_channel`.

## Current Bundle

The current public bundle is:

```text
bundles/simon_telegram_channel/1.4.tar.gz
```

After pushing tag `ironclaw-simon-telegram-1.4`, the direct install URL is:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.4/bundles/simon_telegram_channel/1.4.tar.gz
```

Install through IronClaw's extension URL installer/API with explicit channel kind:

```json
{
  "name": "simon_telegram_channel",
  "url": "https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.4/bundles/simon_telegram_channel/1.4.tar.gz",
  "kind": "wasm_channel"
}
```

Do not use the Settings import flow for this `.tar.gz`; that path is for settings imports, not raw WASM channel bundles.

## Build

```bash
rustup target add wasm32-wasip2
cargo fmt --check && cargo test --manifest-path extensions/simon-telegram-channel/Cargo.toml
IRONCLAW_SIMON_BUNDLE_VERSION=1.4 ./scripts/build-ironclaw-upload-bundles.sh
```

The build script writes:

```text
dist/ironclaw-upload/simon_telegram_channel.tar.gz
bundles/simon_telegram_channel/<version>.tar.gz
```

## Public Safety Rule

Use placeholders only. Do not commit real Telegram IDs, usernames, bot tokens, webhook secrets, calendar IDs, OAuth details, or private Simon family context.
