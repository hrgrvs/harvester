#!/usr/bin/env bash
# Install Harvester as the `harvester` command (macOS, Linux, Omarchy/Arch).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

need_rust() {
  if command -v cargo >/dev/null 2>&1; then
    return 1
  fi
  return 0
}

if need_rust; then
  echo "cargo not found. Installing Rust via rustup (user install)…"
  if ! command -v curl >/dev/null 2>&1; then
    echo "Install curl, then re-run ./install.sh" >&2
    exit 1
  fi
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

echo "Building Harvester (release)…"
cargo build --release

BIN="$ROOT/target/release/harvester"
if [[ ! -x "$BIN" ]]; then
  echo "build failed: $BIN missing" >&2
  exit 1
fi

DEST="${HARVESTER_PREFIX:-$HOME/.local/bin}"
mkdir -p "$DEST"
install -m 0755 "$BIN" "$DEST/harvester"

echo
echo "Installed $DEST/harvester"
if ! command -v harvester >/dev/null 2>&1; then
  echo "Add this to your shell rc if needed:"
  echo "  export PATH=\"$DEST:\$PATH\""
fi
echo "Run:  harvester"
echo
echo "Development continues on Origin. This public GitHub clone is the playable tree."
