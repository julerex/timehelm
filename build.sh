#!/bin/bash
set -e

echo "Building ship game (Bevy WASM)..."
./scripts/build-ship.sh

echo "Building server..."
cd server && cargo build --release && cd ..

echo "Building complete. Run server with: cd server && cargo run"
echo "Ready for deployment with: fly deploy"
