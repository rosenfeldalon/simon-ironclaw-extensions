# Simon Google Calendar Tool

This is Simon's project-specific IronClaw Google Calendar tool.

It is a custom WASM tool named `simon_google_calendar`. Version `0.2.2` supports the read/write Family calendar slice while keeping raw Google calendar IDs and event IDs out of model-facing output.

## Current Scope

- Accepts `calendar.events.list`, `calendar.events.find`, `calendar.events.create`, `calendar.events.update`, and `calendar.events.delete`.
- Accepts only configured calendar aliases; `0.2.2` supports `family`.
- Requires explicit RFC3339 `timeMin` and `timeMax` bounds.
- Requires explicit RFC3339 `start` and `end` bounds for creates; updates may patch title, start/end, location, or notes.
- Derives actor identity from trusted IronClaw job context, not model parameters.
- Allows `alon` and `local_ironclaw_bot`.
- Keeps `shlomit` modeled for later onboarding but unauthorized in this slice.
- Returns shaped DTOs with opaque `eventRef` values instead of raw Google event IDs.
- Uses prior opaque `eventRef` values for update/delete.
- Makes no Google API call for unauthorized actors, invalid windows, unsupported aliases, unsupported actions, invalid event refs, or missing OAuth setup.

The `family` alias reads a private calendar ID from IronClaw workspace path `.system/simon_google_calendar/family_calendar_id`. If that file is absent or empty, it falls back to Google Calendar's `primary` calendar. Do not commit the real Family calendar ID.

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
