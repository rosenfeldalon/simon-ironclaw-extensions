#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist/ironclaw-upload"
TRACKED_BUNDLE_VERSION="${IRONCLAW_SIMON_BUNDLE_VERSION:-1.10}"
TRACKED_BUNDLE_DIR="$ROOT_DIR/bundles/simon_telegram_channel"
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

package_extension \
  "$ROOT_DIR/extensions/simon-telegram-channel/target/wasm32-wasip2/release/simon_telegram_channel.wasm" \
  "$ROOT_DIR/extensions/simon-telegram-channel/simon-telegram.capabilities.json" \
  "simon_telegram_channel"

package_extension \
  "$ROOT_DIR/extensions/simon-google-calendar-tool/target/wasm32-wasip2/release/simon_google_calendar_tool.wasm" \
  "$ROOT_DIR/extensions/simon-google-calendar-tool/simon-google-calendar.capabilities.json" \
  "simon_google_calendar"

rm -rf "$WORK_DIR"

mkdir -p "$TRACKED_BUNDLE_DIR"
cp "$DIST_DIR/simon_telegram_channel.tar.gz" "$TRACKED_BUNDLE_DIR/$TRACKED_BUNDLE_VERSION.tar.gz"

echo "Created upload bundles:"
ls -lh "$DIST_DIR"/*.tar.gz
echo "Tracked bundle copies:"
ls -lh "$TRACKED_BUNDLE_DIR/$TRACKED_BUNDLE_VERSION.tar.gz"
