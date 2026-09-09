#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
MANIFEST="desktop/src-tauri/Cargo.toml"

echo "== Skills Hub Rust Desktop validation =="
rustc --version
cargo --version

if ! pkg-config --exists webkit2gtk-4.1; then
  echo "error: webkit2gtk-4.1 development package is missing" >&2
  echo "Install Ubuntu prerequisites documented in docs/V0.2.0_TAURI.md" >&2
  exit 1
fi

echo "[1/5] desktop Cargo.lock"
cargo generate-lockfile --manifest-path "$MANIFEST"

echo "[2/5] rustfmt"
cargo fmt --manifest-path "$MANIFEST" --all -- --check

echo "[3/5] cargo check"
cargo check --manifest-path "$MANIFEST" --locked

echo "[4/5] clippy"
cargo clippy --manifest-path "$MANIFEST" --all-targets --locked -- -D warnings

echo "[5/5] build"
cargo build --manifest-path "$MANIFEST" --locked

echo "DESKTOP VALIDATION OK"
