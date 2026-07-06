#!/usr/bin/env bash
# Regenerate the WASM engine bindings consumed by apps/desktop-ui.
# Requires: rustup target add wasm32-unknown-unknown
#           cargo install wasm-bindgen-cli --version 0.2.126
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown -p flowly-wasm
wasm-bindgen --target web \
  --out-dir apps/desktop-ui/src/wasm \
  target/wasm32-unknown-unknown/release/flowly_wasm.wasm
