# Simon IronClaw Extensions

Public distribution repo for Simon-specific IronClaw WASM channels and tools.

This repo intentionally contains only distributable extension source, release bundles, and install notes. Private Simon docs, prompts, logs, setup values, Telegram sender IDs, bot tokens, Google Calendar IDs, OAuth details, and household notes belong in the private `simon-docs` repo.

## Extensions

- `extensions/simon-telegram-channel/`: custom IronClaw Telegram channel package named `simon_telegram_channel`.

## Latest Bundle

The latest public bundle is:

```text
bundles/simon_telegram_channel/1.6.tar.gz
```

After pushing tag `ironclaw-simon-telegram-1.6`, the direct install URL is:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.6/bundles/simon_telegram_channel/1.6.tar.gz
```

Important: `1.6` is published for reproduction and inspection, but it did not pass hosted behavioral acceptance. The channel installed and responded, but the expected built-in-style Telegram pairing handshake and durable Simon identity/context still did not work. Do not treat this bundle as known-good.

Install through IronClaw's extension URL installer/API with explicit channel kind only when intentionally reproducing the failed slice:

```json
{
  "name": "simon_telegram_channel",
  "url": "https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.6/bundles/simon_telegram_channel/1.6.tar.gz",
  "kind": "wasm_channel"
}
```

Do not use the Settings import flow for this `.tar.gz`; that path is for settings imports, not raw WASM channel bundles.

## Build

```bash
rustup target add wasm32-wasip2
cargo fmt --check && cargo test --manifest-path extensions/simon-telegram-channel/Cargo.toml
IRONCLAW_SIMON_BUNDLE_VERSION=1.6 ./scripts/build-ironclaw-upload-bundles.sh
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
- `version: "1.6"`
- `type: "channel"`
- `wit_version: "0.3.0"`

Hosted installs must use public URLs from this repo. Do not use raw GitHub URLs from the private `simon-docs` repo.

Raw URL and capabilities checks are necessary release checks, but not success criteria. A release is accepted only after the real hosted Telegram transcript shows the expected pairing handshake before approval and durable Simon identity/context after approval.
