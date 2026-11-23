#!/usr/bin/env bash
set -euo pipefail

export PORT="${PORT:-6129}"
export BOHAT_STATIC_DIR="${BOHAT_STATIC_DIR:-/home/bohat/projects/rustflow-draw/apps/studio/dist}"

BIN="/home/bohat/projects/rustflow-draw/target/release/bohat-draw"
if [ ! -x "$BIN" ]; then
  echo "Building 100% Rust release binary..." >&2
  cargo build --release -p rustflow-studio --manifest-path /home/bohat/projects/rustflow-draw/Cargo.toml
fi

exec "$BIN" --server
