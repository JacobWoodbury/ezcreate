#!/usr/bin/env bash
# ezcreate — first-time setup (macOS / Linux / Git Bash)
# Run from repo root:  ./scripts/setup.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo ""
echo "ezcreate — first-time setup"
echo "========================="
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
echo "[2/2] Running cargo check (first compile can take a while)..."
echo ""

cargo check

echo ""
echo "Setup complete. Run the game with:"
echo "  cargo run"
echo "  ./scripts/run.sh"
echo ""
echo "Walkthrough: docs/GETTING_STARTED.md"
echo ""
