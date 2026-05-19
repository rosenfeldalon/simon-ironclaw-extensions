# Changelog

## 2026-05-13 Simon pickup coordination bundle publish

- Publishes `simon_pickup_coordination` `0.1.0` as a public bundle so Railway-hosted IronClaw can preseed the pickup coordination tool without depending on the private `simon-docs` repo.
- Keeps the durable install-pack split intact: Telegram channel transport, family identity registry, daily briefing, setup, and pickup coordination each remain separate hosted artifacts.

## 2026-05-06 Simon Telegram durable workspace release

- Publishes `simon_telegram_channel` `1.16` with restart-durable workspace persistence for the shared family registry and canonical per-user Telegram chat-id bindings.
- Keeps the May 5 owner-scope routing fix in place while closing the deploy-loop gap where Telegram turns could succeed live but `simon_family_identity` and deploy validation fell back to defaults after restart.
- Keeps the Railway install-pack bundle set aligned on `simon_google_calendar` `0.2.8`, `simon_daily_briefing` `0.2.1`, `simon_family_identity` `0.1.0`, and `simon_setup` `0.1.0`.

## 2026-05-05 Simon calendar write bundle set

- Publishes `simon_telegram_channel` `1.15`, `simon_google_calendar` `0.2.8`, and `simon_daily_briefing` `0.2.1` as one Railway install set.
- Updates the Telegram handoff bundle so trusted parent turns know Family Calendar create/update/delete actions are available through `simon_google_calendar`.
- Updates Daily Briefing output to use cleaner Hebrew section text and collapse multiline locations for Telegram.

## simon_google_calendar 0.2.8

- Allows both trusted parent identities, `alon` and `shlomit`, to read, create, update, and delete Family Calendar events after IronClaw resolves identity.
- Fails closed for empty or unknown actors before any Google Calendar API call, while preserving trusted admin/runtime actors for diagnostics.

## 1.12

- Routes admitted Telegram turns into the resolved owner scope when `pairing_resolve_identity` succeeds, instead of using the canonical Simon family identity as the runtime `user_id`.
- Keeps the canonical Simon identity in prompt-visible handoff context and private thread naming, so Telegram replies still speak as Simon to Alon while workspace/tool/secrets access stays in the owner scope.
- Targets the live Railway failure where web-gateway `simon_google_calendar` calls succeed but Telegram-originated turns appear to land outside the owner workspace/tool domain.

## simon_google_calendar 0.2.4

- Grants workspace read access for `.system/simon_google_calendar/` so the Family calendar alias can be resolved from IronClaw workspace state.
- Fails closed with `CALENDAR_ALIAS_NOT_CONFIGURED` when the Family calendar alias is absent, empty, or unreadable, instead of falling back to Google Calendar `primary`.
- Keeps GUI Family calendar ID setup as a follow-up until IronClaw can bridge setup fields into a tool-readable path.

## 1.10

- Stops forwarding `AuthRequired` and `AuthCompleted` status cards to Telegram; auth remains a GUI/admin surface concern.
- Moves private Telegram messages onto a fresh `telegram-private:safety-1:<identity>` thread namespace to avoid stale pre-1.10 engine context.
- Keeps the `1.9` ignored-message admission gate, Alon verified-sender context, Shlomit future profile, and outbound raw-error sanitization.

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
