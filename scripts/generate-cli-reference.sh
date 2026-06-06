#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT="$ROOT/docs/cli-reference.md"
BINARY="$ROOT/target/release/gitty"

SUBCOMMANDS=(
  scan
  list
  status
  fetch
  pull
  checkout
  group
  tag
  filter
  macro
  health
  scheduler
)

cd "$ROOT"

echo "Building gitty CLI..."
cargo build -p gitty-cli --release

capture_help() {
  local title="$1"
  shift
  {
    echo "## $title"
    echo
    echo '```text'
    "$BINARY" "$@" 2>&1 || true
    echo '```'
    echo
  }
}

{
  echo "# CLI Reference"
  echo
  echo "Auto-generated from \`gitty --help\`. Regenerate with:"
  echo
  echo '```bash'
  echo "./scripts/generate-cli-reference.sh"
  echo '```'
  echo
  capture_help "gitty" --help

  for cmd in "${SUBCOMMANDS[@]}"; do
    capture_help "gitty $cmd" "$cmd" --help
  done
} >"$OUTPUT"

echo "Wrote $OUTPUT"
