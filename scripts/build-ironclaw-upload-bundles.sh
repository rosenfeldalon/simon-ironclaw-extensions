#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/ironclaw-upload"
TELEGRAM_BUNDLE_VERSION="${IRONCLAW_SIMON_TELEGRAM_BUNDLE_VERSION:-1.12}"
CALENDAR_BUNDLE_VERSION="${IRONCLAW_SIMON_CALENDAR_BUNDLE_VERSION:-0.2.8}"
BRIEFING_BUNDLE_VERSION="${IRONCLAW_SIMON_DAILY_BRIEFING_BUNDLE_VERSION:-0.1.2}"
TELEGRAM_BUNDLE_DIR="$ROOT_DIR/bundles/simon_telegram_channel"
CALENDAR_BUNDLE_DIR="$ROOT_DIR/bundles/simon_google_calendar"
BRIEFING_BUNDLE_DIR="$ROOT_DIR/bundles/simon_daily_briefing"
WORK_DIR="$DIST_DIR/.work"

mkdir -p "$WORK_DIR"
rm -f "$DIST_DIR"/*.tar.gz

build_crate() {
  local crate_dir="$1"
  (
    cd "$ROOT_DIR/$crate_dir"
    cargo fmt --check
    cargo build --target wasm32-wasip2 --release
  )
}

package_extension() {
  local wasm_src="$1"
  local caps_src="$2"
  local canonical_name="$3"

  rm -rf "$WORK_DIR"
  mkdir -p "$WORK_DIR"
  cp "$wasm_src" "$WORK_DIR/$canonical_name.wasm"
  cp "$caps_src" "$WORK_DIR/$canonical_name.capabilities.json"
  tar -C "$WORK_DIR" -czf "$DIST_DIR/$canonical_name.tar.gz" \
    "$canonical_name.wasm" \
    "$canonical_name.capabilities.json"
}

build_crate "extensions/simon-telegram-channel"
build_crate "extensions/simon-google-calendar-tool"
build_crate "extensions/simon-daily-briefing"

package_extension \
  "$ROOT_DIR/extensions/simon-telegram-channel/target/wasm32-wasip2/release/simon_telegram_channel.wasm" \
  "$ROOT_DIR/extensions/simon-telegram-channel/simon-telegram.capabilities.json" \
  "simon_telegram_channel"

package_extension \
  "$ROOT_DIR/extensions/simon-google-calendar-tool/target/wasm32-wasip2/release/simon_google_calendar_tool.wasm" \
  "$ROOT_DIR/extensions/simon-google-calendar-tool/simon-google-calendar.capabilities.json" \
  "simon_google_calendar"

package_extension \
  "$ROOT_DIR/extensions/simon-daily-briefing/target/wasm32-wasip2/release/simon_daily_briefing.wasm" \
  "$ROOT_DIR/extensions/simon-daily-briefing/simon-daily-briefing.capabilities.json" \
  "simon_daily_briefing"

rm -rf "$WORK_DIR"

mkdir -p "$TELEGRAM_BUNDLE_DIR" "$CALENDAR_BUNDLE_DIR" "$BRIEFING_BUNDLE_DIR"
cp "$DIST_DIR/simon_telegram_channel.tar.gz" "$TELEGRAM_BUNDLE_DIR/$TELEGRAM_BUNDLE_VERSION.tar.gz"
cp "$DIST_DIR/simon_google_calendar.tar.gz" "$CALENDAR_BUNDLE_DIR/$CALENDAR_BUNDLE_VERSION.tar.gz"
cp "$DIST_DIR/simon_daily_briefing.tar.gz" "$BRIEFING_BUNDLE_DIR/$BRIEFING_BUNDLE_VERSION.tar.gz"

echo "Created upload bundles:"
ls -lh "$DIST_DIR"/*.tar.gz
echo "Tracked bundle copies:"
ls -lh \
  "$TELEGRAM_BUNDLE_DIR/$TELEGRAM_BUNDLE_VERSION.tar.gz" \
  "$CALENDAR_BUNDLE_DIR/$CALENDAR_BUNDLE_VERSION.tar.gz" \
  "$BRIEFING_BUNDLE_DIR/$BRIEFING_BUNDLE_VERSION.tar.gz"
