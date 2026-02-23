#!/bin/bash
# Exit on any error
set -e

echo "--- ENGINE BUILD START ---"

# Clear everything
rm -rf dist pkg target/wasm32-unknown-unknown
mkdir -p dist

# Install wasm-pack to the current directory
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh -s -- --to $(pwd)
export PATH=$(pwd):$PATH

echo ">> VERIFYING WASM-PACK..."
./wasm-pack --version

echo ">> COMPILING WASM CRATE..."
# We run wasm-pack from the root and point it to the crate
./wasm-pack build crates/shell-wasm --target web --out-dir ../../dist/pkg --release

echo ">> DEPLOYING HTML..."
# Copy the index.html directly to dist
cp crates/shell-wasm/index.html dist/index.html

# Create a build marker
echo "BUILD_TIME: $(date)" > dist/version.txt

echo "--- DIRECTORY STRUCTURE ---"
ls -R dist
echo "--- ENGINE BUILD COMPLETE ---"
