# Simon Daily Briefing Tool

This is Simon's deterministic Daily Briefing tool for IronClaw.

It is a custom WASM tool named `simon_daily_briefing`. Version `0.2.1` reads the same Family calendar alias and OAuth secret path used by the live `simon_google_calendar` tool, but now splits the work into a shared-facts stage plus a recipient-render stage for canonical Simon users.

## Current Scope

- Accepts three actions:
  - `generate_daily_briefing`
  - `generate_family_briefing_facts`
  - `render_daily_briefing`
- Supports only `calendarAlias: "family"` in `0.2.1`.
- Defaults `date` to today's local day in `Asia/Jerusalem` when omitted.
- Defaults static heading language from the canonical recipient profile when available, otherwise Hebrew.
- Uses `.system/simon_google_calendar/family_calendar_id` as the Family calendar source of truth.
- Reads recipient identity defaults from `channels/simon_telegram_channel/state/simon_family_profiles.json` when that shared registry exists.
- Uses the shared Simon Google Calendar OAuth secret names so the briefing tool can follow the same configured calendar path as `simon_google_calendar`.
- Returns one compact `messageText` plus structured `allDayEvents`, `timedEvents`, `eventCount`, `windowStart`, and `windowEnd`.
- Makes only `GET` requests to Google Calendar and never creates, edits, deletes, invites, notifies, or sets reminders.
- Keeps Shlomit's delivery routine dormant by default, but supports `recipientIdentity: "shlomit"` so the install pack is ready once onboarding gates pass.

## Build

```bash
rustup target add wasm32-wasip2
cargo fmt --check
cargo test
cargo build --target wasm32-wasip2 --release
```

From the repo root, `./scripts/build-ironclaw-upload-bundles.sh` also packages:

```text
dist/ironclaw-upload/simon_daily_briefing.tar.gz
```

## Install

For local CLI installs:

```bash
ironclaw tool install ./target/wasm32-wasip2/release/simon_daily_briefing.wasm \
  --capabilities ./simon-daily-briefing.capabilities.json \
  --name simon_daily_briefing

ironclaw tool setup simon_daily_briefing
ironclaw tool auth simon_daily_briefing
```

If `simon_google_calendar` already completed setup and OAuth with the same shared secret names, `setup` and `auth` may already be satisfied. Still verify the tool state after install instead of assuming it.

Hosted IronClaw's URL installer expects a direct `.tar.gz` bundle URL:

```text
Name: simon_daily_briefing
Kind: wasm_tool
URL: <direct HTTPS URL ending in simon_daily_briefing.tar.gz>
```

Do not use private `simon-docs` raw GitHub URLs for hosted installs.
