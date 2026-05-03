# Simon Setup Tool

This is Simon's install-pack setup and bootstrap planning tool for IronClaw.

It is a custom WASM tool named `simon_setup`. Version `0.1.0` generates the reusable Simon install manifest, family registry preview, bootstrap runbook, and per-user workspace seed docs for new Simon deployments.

## Current Scope

- Provides one install-pack manifest covering `simon_telegram_channel`, `simon_google_calendar`, `simon_daily_briefing`, `simon_family_identity`, and `simon_setup`.
- Generates a structured family registry preview for canonical users such as `alon` and `shlomit`.
- Generates per-user workspace seed docs for `AGENTS.md`, `IDENTITY.md`, `SOUL.md`, and `TOOLS.md`.
- Stores onboarding field values in `extensions.simon_setup.*` owner-scoped settings through IronClaw's setup-field persistence.
- Does not yet write workspace files directly; current WASM tool host access is read-only for workspace state.

## Build

```bash
rustup target add wasm32-wasip2
cargo fmt --check
cargo test
cargo build --target wasm32-wasip2 --release
```

From the repo root, `./scripts/build-ironclaw-upload-bundles.sh` also packages:

```text
dist/ironclaw-upload/simon_setup.tar.gz
```

## Install

Hosted IronClaw's URL installer expects a direct `.tar.gz` bundle URL:

```text
Name: simon_setup
Kind: wasm_tool
URL: <direct HTTPS URL ending in simon_setup.tar.gz>
```
