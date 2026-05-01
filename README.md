# Simon IronClaw Extensions

Public distribution repo for Simon-specific IronClaw WASM channels and tools.

This repo intentionally contains only distributable extension source, release bundles, and install notes. Private Simon docs, prompts, logs, setup values, Telegram sender IDs, bot tokens, Google Calendar IDs, OAuth details, and household notes belong in the private `simon-docs` repo.

## Extensions

- `extensions/simon-telegram-channel/`: custom IronClaw Telegram channel package named `simon_telegram_channel`.
- `extensions/simon-google-calendar-tool/`: Simon-specific Google Calendar read/write tool package named `simon_google_calendar`.

## Latest Bundle

The latest public bundle is:

```text
bundles/simon_telegram_channel/1.11.tar.gz
```

After pushing tag `ironclaw-simon-telegram-1.11`, the direct install URL is:

```text
https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.11/bundles/simon_telegram_channel/1.11.tar.gz
```

Important: `1.11` keeps the `1.10` safety boundary, reinforces the Simon-vs-IronClaw assistant identity and `simon_google_calendar` JSON action shape in Telegram handoff context, and moves private Telegram messages onto a fresh thread namespace to avoid stale engine context. It should log `Simon Telegram channel runtime version 1.11` at startup.

Install through IronClaw's extension URL installer/API with explicit channel kind:

```json
{
  "name": "simon_telegram_channel",
  "url": "https://raw.githubusercontent.com/rosenfeldalon/simon-ironclaw-extensions/ironclaw-simon-telegram-1.11/bundles/simon_telegram_channel/1.11.tar.gz",
  "kind": "wasm_channel"
}
```

Do not use the Settings import flow for this `.tar.gz`; that path is for settings imports, not raw WASM channel bundles.

## Build

```bash
rustup target add wasm32-wasip2
cargo fmt --check && cargo test --manifest-path extensions/simon-telegram-channel/Cargo.toml
IRONCLAW_SIMON_TELEGRAM_BUNDLE_VERSION=1.11 \
IRONCLAW_SIMON_CALENDAR_BUNDLE_VERSION=0.2.4 \
  ./scripts/build-ironclaw-upload-bundles.sh
```

The build script writes:

```text
dist/ironclaw-upload/simon_telegram_channel.tar.gz
dist/ironclaw-upload/simon_google_calendar.tar.gz
bundles/simon_telegram_channel/<version>.tar.gz
bundles/simon_google_calendar/<version>.tar.gz
```

## Public Safety Rule

Use placeholders only. Do not commit real Telegram IDs, usernames, bot tokens, webhook secrets, calendar IDs, OAuth details, or private Simon family context.

For `simon_google_calendar`, keep live Google OAuth Client IDs, Client Secrets, refresh tokens, raw calendar IDs, and raw Google event IDs out of fixtures, docs, logs, bundle metadata, and reports.

## Release Verification

Before sharing an install URL, verify the pushed raw GitHub URL returns `200` and inspect the packaged capabilities JSON for:

- `name: "simon_telegram_channel"`
- `version: "1.11"`
- `type: "channel"`
- `wit_version: "0.3.0"`

Hosted installs must use public URLs from this repo. Do not use raw GitHub URLs from the private `simon-docs` repo.

Raw URL and capabilities checks are necessary release checks, but not success criteria. A release is accepted only after the real hosted Telegram transcript shows durable Simon identity/context and correct calendar tool routing after approval.

`simon_google_calendar` `0.2.4` is the current read/write hosted-install candidate. It fails closed if the `family` alias is not configured at `.system/simon_google_calendar/family_calendar_id`, instead of silently querying Google Calendar `primary`. Its local lab gate is the report from `/Users/alonr/projects/simon-ironclaw-lab`:

```bash
iclab calendar contract
```

Only publish the calendar tool after local fake-contract tests, capabilities inspection, and an explicit non-sensitive OAuth smoke pass.

## Diagnostic Context

The reusable lab at `/Users/alonr/projects/simon-ironclaw-lab` compares the built-in `telegram` channel and this custom `simon_telegram_channel` under the same fake Telegram pairing scenario.

That lab found the `1.6` failure: the custom channel still used the built-in `/webhook/telegram` route, while IronClaw registers custom WASM channels at `/webhook/{channel_name}`. Version `1.7` aligned source, capabilities, and bundle metadata on `/webhook/simon_telegram_channel`; `1.8` added hosted diagnostics for the remaining admission/identity gap; `1.9` added the safety behavior for ignored unapproved Telegram messages and sanitized outbound raw errors; `1.10` suppresses Telegram auth status cards and starts a fresh private thread namespace; `1.11` refreshes the namespace again and injects stronger Simon/calendar tool handoff context for admitted Telegram turns.
