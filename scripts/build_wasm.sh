#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Keep compiler paths stable across local machines and CI runners while
# preserving any flags supplied by the caller.
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
# When rust-src is installed, inlined standard-library code can embed its
# local path instead of rustc's virtual /rustc/<commit> source path.
SYSROOT="$(rustc --print sysroot)"
RUSTC_COMMIT="$(rustc -vV | awk '/^commit-hash:/ {print $2}')"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=${REPO_ROOT}=/workspace --remap-path-prefix=${CARGO_HOME}=/cargo --remap-path-prefix=${SYSROOT}/lib/rustlib/src/rust=/rustc/${RUSTC_COMMIT}"

cargo build --locked --release --target wasm32-wasip2
