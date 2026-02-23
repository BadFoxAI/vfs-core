#!/bin/bash
set -e

echo "--- STARTING ENGINE BUILD ---"
mkdir -p dist

# Install wasm-pack to a local, reachable bin
echo ">> Installing wasm-pack locally..."
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh -s -- --to $(pwd)
export PATH=$(pwd):$PATH

echo ">> Verifying wasm-pack..."
./wasm-pack --version

echo ">> Building shell-wasm crate..."
# Run wasm-pack from the root, pointing to the crate
./wasm-pack build crates/shell-wasm --target web --out-dir ../../dist/pkg --release

echo ">> Copying web assets..."
cp crates/shell-wasm/index.html dist/index.html

echo ">> Creating build manifest..."
date > dist/BUILT_AT.txt

echo "--- BUILD SUCCESSFUL ---"
ls -R dist
