#!/bin/bash
set -e

echo "Building ship game (Bevy WASM)..."
./scripts/build-ship.sh

echo "Building frontend..."
cd client
npm install
npm run build
cd ..

echo "Building complete. Frontend files are in client/dist/"
echo "Ready for deployment with: fly deploy"

