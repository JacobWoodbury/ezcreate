#!/usr/bin/env bash
# ezcreate - first-time setup (Linux; macOS delegates to setup-mac.sh)
# Run from repo root:  ./scripts/setup.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "$(uname -s)" == "Darwin" ]]; then
  exec "$SCRIPT_DIR/setup-mac.sh" "$@"
fi

ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

echo ""
echo "ezcreate - first-time setup (Linux)"
echo "==================================="
echo ""

echo "[1/2] Checking Rust..."
if ! command -v rustc >/dev/null 2>&1; then
  echo "  Rust is not installed."
  echo "  Install: https://rustup.rs"
  echo "  Then run:  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 1
fi
echo "  $(rustc --version)"
echo "  host: $(rustc -vV | awk '/^host: / { print $2 }')"

echo ""
echo "[2/2] Checking C compiler..."
if ! command -v cc >/dev/null 2>&1; then
  echo "  cc not found. Install build tools, for example:"
  echo "    Debian/Ubuntu:  sudo apt install build-essential"
  echo "    Fedora:         sudo dnf groupinstall 'Development Tools'"
  echo "    Arch:           sudo pacman -S base-devel"
  exit 1
fi
echo "  $(cc --version | head -n1)"

echo ""
echo "Running cargo check (first compile can take a while)..."
echo ""

cargo check

echo ""
echo "Setup complete. Run the game with:"
echo "  cargo run"
echo "  ./scripts/run.sh"
echo ""
echo "Walkthrough: docs/GETTING_STARTED.md"
echo ""
