# Simon Google Calendar Tool

This is Simon's project-specific IronClaw Google Calendar tool.

It is a custom WASM tool named `simon_google_calendar`. Version `0.2.5` supports the read/write Family calendar slice plus a redacted calendar-list diagnostic while keeping raw Google calendar IDs and event IDs out of model-facing output.

## Current Scope

- Accepts `calendar.events.list`, `calendar.events.find`, `calendar.events.create`, `calendar.events.update`, `calendar.events.delete`, and `calendar.calendars.list`.
- Accepts only configured calendar aliases; `0.2.5` supports `family`.
- Requires explicit RFC3339 `timeMin` and `timeMax` bounds.
- Requires explicit RFC3339 `start` and `end` bounds for creates; updates may patch title, start/end, location, or notes.
- Derives actor identity from trusted IronClaw job context, not model parameters.
- Allows `alon` and `local_ironclaw_bot`.
- Keeps `shlomit` modeled for later onboarding but unauthorized in this slice.
- Returns shaped DTOs with opaque `eventRef` values instead of raw Google event IDs.
- Uses prior opaque `eventRef` values for update/delete.
- Makes no Google API call for unauthorized actors, invalid windows, unsupported aliases, unsupported actions, invalid event refs, or missing OAuth setup.
- `calendar.calendars.list` is read-only and redacts raw calendar IDs while marking which returned calendar currently matches the configured `family` alias.

The `family` alias reads a private calendar ID from IronClaw workspace path `.system/simon_google_calendar/family_calendar_id`. If that file is absent, empty, or unreadable, the tool returns `CALENDAR_ALIAS_NOT_CONFIGURED` before any Google API call. Do not commit the real Family calendar ID.

GUI setup for the Family calendar ID is a follow-up. IronClaw setup fields are currently persisted as settings, while WASM tools read extension-owned workspace paths; adding a setup field without an IronClaw bridge would make the GUI look configured while the tool still could not read the value.

## Setup Secrets

Do not put real Google OAuth values, refresh tokens, raw calendar IDs, or raw event IDs in this repo.

The tool uses dedicated Simon credential names:

- `simon_google_calendar_oauth_token`: stored by IronClaw after OAuth.
- `simon_google_calendar_oauth_client_id`: Google OAuth Web application Client ID.
- `simon_google_calendar_oauth_client_secret`: Google OAuth Web application Client Secret.

Google Cloud Console provides the Client ID and Client Secret for an OAuth Web application. IronClaw uses those during `tool auth`; the OAuth token is created later by Google after the account authorizes Calendar access with `https://www.googleapis.com/auth/calendar.events`.

## Build

```bash
rustup target add wasm32-wasip2
cargo fmt --check
cargo test
cargo build --target wasm32-wasip2 --release
```

From the repo root, `./scripts/build-ironclaw-upload-bundles.sh` also packages:

```text
dist/ironclaw-upload/simon_google_calendar.tar.gz
```

Do not publish or install this hosted until local fake-contract tests, capabilities inspection, and a live non-sensitive OAuth smoke pass.
