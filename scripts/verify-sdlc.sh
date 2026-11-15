#!/usr/bin/env bash
set -euo pipefail

echo "==> [1/4] Checking Rust formatting (cargo fmt --check)..."
cargo fmt --all -- --check

echo "==> [2/4] Running strict Clippy lints (cargo clippy -- -D warnings)..."
cargo clippy --workspace --all-targets -- -D warnings

echo "==> [3/4] Running Workspace Unit, VRT & Benchmark Tests..."
cargo test --workspace

echo "==> [4/4] Verifying Web Application syntax..."
node --check apps/web/server.mjs
node --check apps/web/src/app.js

echo "==> All SDLC quality gates passed!"
