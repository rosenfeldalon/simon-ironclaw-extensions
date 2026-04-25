# Changelog

## 1.5

- Restored official IronClaw pairing as the Telegram identity gate via `pairing_resolve_identity`.
- Removed first-private-DM binding and the diagnostic `/start` route fingerprint.
- Emits canonical Simon `alon` context only after pairing resolves.
- Packaged as an IronClaw channel bundle with `type: "channel"` and WIT `0.3.0`.

## 1.4

- Added a direct `/start` route fingerprint to prove the custom channel is receiving Telegram DMs.
- Kept token-only setup and first-private-DM identity binding from `1.2`/`1.3`.
- Packaged as an IronClaw channel bundle with `type: "channel"` and WIT `0.3.0`.

## 0.2.10-baseline.1

- Established a custom-named clone of IronClaw's built-in Telegram channel.
- Proved hosted installation works as `CHANNEL` when installed with explicit `kind: "wasm_channel"`.
