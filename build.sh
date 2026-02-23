#!/bin/bash
set -e

echo ">> Installing wasm-pack..."
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

echo ">> Building shell-wasm..."
cd crates/shell-wasm
wasm-pack build --target web --out-dir pkg

echo ">> Assembling Distribution..."
# Create a specific dist folder to serve
mkdir -p ../../dist
cp index.html ../../dist/index.html
cp -r pkg ../../dist/pkg

echo ">> Build Complete. Artifacts ready in /dist"
