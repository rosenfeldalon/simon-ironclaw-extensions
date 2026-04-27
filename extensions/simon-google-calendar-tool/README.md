# Simon Google Calendar Tool

This is Simon's project-specific IronClaw Google Calendar tool.

It is a custom WASM tool named `simon_google_calendar`. The first version is intentionally read-only and mirrors the local contract proven by `/Users/alonr/projects/simon-ironclaw-lab` with `iclab calendar contract`.

## Current Scope

- Accepts only `calendar.events.list` and `calendar.events.find`.
- Accepts only configured calendar aliases; V1 supports `family`.
- Requires explicit RFC3339 `timeMin` and `timeMax` bounds.
- Derives actor identity from trusted IronClaw job context, not model parameters.
- Allows `alon` and `local_ironclaw_bot`.
- Keeps `shlomit` modeled for later onboarding but unauthorized in V1.
- Returns shaped DTOs with opaque `eventRef` values instead of raw Google event IDs.
- Makes no Google API call for unauthorized actors, invalid windows, unsupported aliases, unsupported actions, or missing OAuth setup.

## Setup Secrets

Do not put real Google OAuth values, refresh tokens, raw calendar IDs, or raw event IDs in this repo.

The tool uses dedicated Simon credential names:

- `simon_google_calendar_oauth_token`: stored by IronClaw after OAuth.
- `simon_google_calendar_oauth_client_id`: Google OAuth Web application Client ID.
- `simon_google_calendar_oauth_client_secret`: Google OAuth Web application Client Secret.

Google Cloud Console provides the Client ID and Client Secret for an OAuth Web application. IronClaw uses those during `tool auth`; the OAuth token is created later by Google after the account authorizes Calendar access.

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
