#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== Skills Hub Rust: local Linux validation =="
echo
rustc --version
cargo --version
rustup show active-toolchain || true
git --version

echo
echo "[1/8] rustfmt"
cargo fmt --all
cargo fmt --all -- --check

echo "[2/8] Cargo.lock"
cargo generate-lockfile
test -s Cargo.lock

echo "[3/8] cargo check"
cargo check --all-targets --locked

echo "[4/8] clippy"
cargo clippy --all-targets --locked -- -D warnings

echo "[5/8] tests"
cargo test --all-targets --locked

echo "[6/8] release build"
cargo build --release --locked

echo "[7/8] isolated smoke test"
./scripts/smoke-test.sh ./target/release/skills-hub-rs

echo "[8/8] package local Linux binary"
rm -rf dist/local-package
mkdir -p dist/local-package
cp target/release/skills-hub-rs dist/skills-hub-rs-linux-x86_64
chmod +x dist/skills-hub-rs-linux-x86_64
cp dist/skills-hub-rs-linux-x86_64 dist/local-package/skills-hub-rs
cp README.md LICENSE dist/local-package/
tar -C dist/local-package -czf dist/skills-hub-rs-linux-x86_64.tar.gz .
(
  cd dist
  sha256sum skills-hub-rs-linux-x86_64 > skills-hub-rs-linux-x86_64.sha256
  sha256sum skills-hub-rs-linux-x86_64.tar.gz > skills-hub-rs-linux-x86_64.tar.gz.sha256
)
rm -rf dist/local-package

echo
echo "VALIDATION OK"
echo "Artifacts:"
ls -lh dist/skills-hub-rs-linux-x86_64 dist/skills-hub-rs-linux-x86_64.tar.gz \
  dist/skills-hub-rs-linux-x86_64.sha256 dist/skills-hub-rs-linux-x86_64.tar.gz.sha256
