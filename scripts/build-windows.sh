#!/usr/bin/env bash
# Cross-compile the Windows app + NSIS installer from Linux.
# Requires: rustup target add x86_64-pc-windows-msvc; cargo install cargo-xwin;
#           apt install nsis llvm; vendor/ populated (libvosk + model).
set -euo pipefail
cd "$(dirname "$0")/.."
npm --prefix apps/desktop-ui run build
cd apps/desktop-shell
export RUSTFLAGS="-L native=$(pwd)/vendor"
export XWIN_ACCEPT_LICENSE=yes
exec ../desktop-ui/node_modules/.bin/tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc "$@"
