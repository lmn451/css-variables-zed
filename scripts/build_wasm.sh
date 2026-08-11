#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Keep compiler paths stable across local machines and CI runners while
# preserving any flags supplied by the caller.
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=${REPO_ROOT}=/workspace --remap-path-prefix=${CARGO_HOME}=/cargo"

cargo build --locked --release --target wasm32-wasip1
