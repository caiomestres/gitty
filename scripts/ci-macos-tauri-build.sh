#!/usr/bin/env bash
set -euo pipefail

npx tauri build --target universal-apple-darwin --bundles app "$@"

APP_PATH="$(find src-tauri/target -path '*/release/bundle/macos/*.app' | head -1)"
if [ -z "$APP_PATH" ]; then
  echo "error: macOS .app bundle not found after build" >&2
  exit 1
fi

echo "Ad-hoc codesigning: $APP_PATH"
codesign -s - --deep --force "$APP_PATH"

npx tauri bundle --target universal-apple-darwin --bundles dmg "$@"
