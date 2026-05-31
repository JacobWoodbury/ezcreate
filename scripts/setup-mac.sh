#!/usr/bin/env bash
# ezcreate - first-time setup (macOS)
# Run from repo root:  ./scripts/setup-mac.sh

set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script is for macOS only."
  echo "On Linux, use:  ./scripts/setup.sh"
  echo "On Windows, use:  .\\scripts\\setup.ps1"
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo ""
echo "ezcreate - first-time setup (macOS)"
echo "===================================="
echo ""

# --- 1. Xcode Command Line Tools (clang, linker) ---
echo "[1/3] Checking Xcode Command Line Tools..."
if ! xcode-select -p >/dev/null 2>&1; then
  echo "  Command Line Tools are not installed."
  echo ""
  echo "  Install them (GUI prompt will open):"
  echo "    xcode-select --install"
  echo ""
  echo "  Or install full Xcode from the App Store, then run:"
  echo "    sudo xcode-select -s /Applications/Xcode.app/Contents/Developer"
  echo ""
  echo "  When finished, run this script again."
  exit 1
fi

if ! xcrun --find clang >/dev/null 2>&1; then
  echo "  clang not found via xcrun. Reinstall Command Line Tools:"
  echo "    xcode-select --install"
  exit 1
fi

CLANG_VER="$(clang --version 2>/dev/null | head -n1 || true)"
echo "  xcode-select: $(xcode-select -p)"
if [[ -n "$CLANG_VER" ]]; then
  echo "  $CLANG_VER"
fi

# --- 2. Rust ---
echo ""
echo "[2/3] Checking Rust..."
if ! command -v rustc >/dev/null 2>&1; then
  echo "  Rust is not installed."
  echo ""
  echo "  Install rustup, then restart Terminal:"
  echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo ""
  echo "  Apple Silicon (M1/M2/M3): default host is usually aarch64-apple-darwin"
  echo "  Intel Mac: x86_64-apple-darwin"
  exit 1
fi

RUST_HOST="$(rustc -vV | awk '/^host: / { print $2 }')"
echo "  $(rustc --version)"
echo "  host: $RUST_HOST"

case "$RUST_HOST" in
  *-apple-darwin)
    ;;
  *)
    echo "  Warning: expected an Apple Darwin toolchain (e.g. aarch64-apple-darwin)."
    echo "  Install the macOS target:  rustup target add aarch64-apple-darwin"
    ;;
esac

# --- 3. Build check ---
echo ""
echo "[3/3] Running cargo check (first time may take 10-30+ min)..."
echo ""

cargo check

echo ""
echo "Setup complete. Run the game with:"
echo "  cargo run"
echo "  ./scripts/run.sh"
echo ""
echo "macOS walkthrough and shortcuts: docs/GETTING_STARTED.md#macos"
echo ""
