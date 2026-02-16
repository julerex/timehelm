#!/bin/bash
# Build the Bevy ship game for WebAssembly and output to client/dist/ship/
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Ensure wasm target is installed
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# Build ship game for WASM (getrandom needs wasm_js cfg for wasm32)
echo "Building ship game for WASM..."
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --manifest-path ship-game/Cargo.toml --target wasm32-unknown-unknown --release

# Install wasm-bindgen-cli if needed
if ! command -v wasm-bindgen &>/dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Create output directory for ship WASM (server serves from client/public/)
OUT_DIR="$PROJECT_ROOT/client/public/ship"
mkdir -p "$OUT_DIR"

# Run wasm-bindgen to generate JS glue (output is in ship-game/target when using --manifest-path)
WASM_PATH="$PROJECT_ROOT/ship-game/target/wasm32-unknown-unknown/release/ship_game.wasm"
wasm-bindgen --no-typescript --target web \
    --out-dir "$OUT_DIR" \
    --out-name "ship" \
    "$WASM_PATH"

echo "Ship game built to $OUT_DIR"
echo "  ship.js"
echo "  ship_bg.wasm"
