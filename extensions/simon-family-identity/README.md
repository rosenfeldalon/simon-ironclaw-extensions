# Simon Family Identity Tool

This is Simon's canonical family identity registry tool for IronClaw.

It is a custom WASM tool named `simon_family_identity`. Version `0.1.0` reads the structured Simon family registry persisted by `simon_telegram_channel` and exposes recipient-first identity data for other Simon routines, prompts, and operators.

## Current Scope

- Reads the shared family registry from `channels/simon_telegram_channel/state/simon_family_profiles.json`.
- Falls back to a deterministic default registry when the Telegram channel has not seeded runtime state yet.
- Exposes canonical users such as `alon` and `shlomit`.
- Returns recipient delivery readiness and Telegram routing targets without exposing secrets.
- Renders per-user workspace seed docs for `AGENTS.md`, `IDENTITY.md`, `SOUL.md`, and `TOOLS.md`.
- Does not write runtime state; the current IronClaw WASM tool host is read-only for workspace access.

## Build

```bash
rustup target add wasm32-wasip2
cargo fmt --check
cargo test
cargo build --target wasm32-wasip2 --release
```

From the repo root, `./scripts/build-ironclaw-upload-bundles.sh` also packages:

```text
dist/ironclaw-upload/simon_family_identity.tar.gz
```

## Install

Hosted IronClaw's URL installer expects a direct `.tar.gz` bundle URL:

```text
Name: simon_family_identity
Kind: wasm_tool
URL: <direct HTTPS URL ending in simon_family_identity.tar.gz>
```
