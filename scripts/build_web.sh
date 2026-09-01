#!/usr/bin/env bash
set -euo pipefail

# Build script for Laser Potato static web deployment.
# Outputs static HTML, JS, and WASM bundles to the ./dist directory.

echo "=== Building Laser Potato Static Web Deployment ==="

# Check if trunk is installed
if command -v trunk &> /dev/null; then
    echo "[+] Building with Trunk..."
    trunk build --release
    echo "[✓] Web build complete in ./dist/"
    echo "    To test locally: trunk serve --port 8080"
    exit 0
fi

# Fallback: Build using cargo and wasm-bindgen CLI
echo "[!] 'trunk' CLI not found. Falling back to cargo + wasm-bindgen-cli..."

# Ensure wasm target is installed
rustup target add wasm32-unknown-unknown

# Build release wasm binary for play target
cargo build --release --target wasm32-unknown-unknown --bin play

# Ensure dist directory exists
mkdir -p dist

# Check for wasm-bindgen
if command -v wasm-bindgen &> /dev/null; then
    echo "[+] Running wasm-bindgen..."
    wasm-bindgen --target web \
        --out-dir dist \
        --out-name laserpotato \
        --no-typescript \
        target/wasm32-unknown-unknown/release/play.wasm

    # Copy HTML shell
    cp index.html dist/index.html
    echo "[✓] Static web build generated in ./dist/"
else
    echo "[!] Tip: Install 'trunk' for easy one-command builds and local preview:"
    echo "    cargo install --locked trunk"
fi
