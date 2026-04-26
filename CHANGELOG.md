# Changelog

## 1.9

- Logs unapproved Telegram messages with redacted admission metadata and ignores them before agent emission.
- Keeps private unapproved sender onboarding through the pairing-code reply.
- Adds outbound sanitization for obvious raw tool/runtime/schema/auth errors before Telegram delivery.
- Refactors Simon identity handling around explicit identity profiles, with Shlomit modeled for future onboarding but not active or auto-bound.
- Packaged as an IronClaw channel bundle with `type: "channel"` and WIT `0.3.0`.

## 1.8

- Added hosted runtime and redacted admission diagnostics to identify stale runtime or admission-state behavior.
- Kept the custom `/webhook/simon_telegram_channel` route fix from `1.7`.

## 1.7

- Changed the custom webhook path from `/webhook/telegram` to `/webhook/simon_telegram_channel`.
- Passed the blocker-lab built-in-vs-custom Telegram pairing comparison with the public HTTPS bundle.

## 1.6

- Hosted behavioral acceptance failed: the bot responded, but the expected built-in-style Telegram pairing handshake and durable Simon identity/context were still not achieved.
- Restores built-in-style Telegram admission using owner, allow-list, and pairing signals before emitting messages to Simon.
- Sends the official pairing-code reply for unadmitted private senders and stops before the agent sees the message.
- Emits a plain-text verified Simon sender context for admitted Alon messages.
- Packaged as an IronClaw channel bundle with `type: "channel"` and WIT `0.3.0`.

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
