#!/usr/bin/env bash
# CI wrapper: routes `tauri build` through the macOS ad-hoc signing flow when enabled.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "${GITTY_MACOS_ADHOC_SIGN:-}" = "1" ] && [ "$1" = "build" ]; then
  shift
  exec bash "$SCRIPT_DIR/ci-macos-tauri-build.sh" "$@"
fi

exec npx tauri "$@"
