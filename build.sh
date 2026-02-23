#!/bin/bash
set -e

echo ">> CLEANING OLD ARTIFACTS..."
rm -rf dist
mkdir -p dist

echo ">> INSTALLING WASM-PACK..."
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
export PATH=$PATH:$HOME/.cargo/bin

echo ">> COMPILING WASM (RELEASE MODE)..."
cd crates/shell-wasm
# Force a clean build of the wasm crate
cargo clean
wasm-pack build --target web --out-dir pkg --release

echo ">> ASSEMBLING DISTRO..."
cp index.html ../../dist/index.html
cp -r pkg ../../dist/pkg

# Add a build timestamp to index.html for verification
sed -i "s/Sovereign Shell/Sovereign Shell (Built: $(date +'%T'))/" ../../dist/index.html

echo ">> DONE. Ready for Netlify."
