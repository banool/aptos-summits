#!/bin/sh

set -e

cd "$(dirname "$0")"
cd ..

# Build the binary.
cargo build --release --target wasm32-unknown-unknown -p web

# Build the JS code to make it possible to load up the wasm.
wasm-bindgen ./target/wasm32-unknown-unknown/release/web.wasm --target web --out-name "summits" --out-dir /tmp/summits

# Optimize the size of the wasm.
# wasm-opt -Oz -o /tmp/summits/small.wasm /tmp/summits/summits_bg.wasm

# Move it in to place.
rm -rf ../frontend/src/summits
mv /tmp/summits/small.wasm /tmp/summits/summits_bg.wasm
mv /tmp/summits ../frontend/src/summits
