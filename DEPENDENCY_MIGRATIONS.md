# Dependency migration notes

Baseline: `432ff5492ef783a9ba99431848ed21e69a8a986a` on the default branch.
The migration uses stable Rust 1.97.1. `Cargo.lock` records the complete resolved
graph; upstream constraints imposed by the published Zed API remain intact.

## Dependency decisions

| Dependency | Baseline | Result |
| --- | --- | --- |
| `zed_extension_api` | 0.7.0 | 0.7.0, latest published stable |
| direct `anyhow` | `1` (1.0.100 locked) | Removed unused declaration; transitive build dependency updated to 1.0.104 |
| direct `serde_json` | `1.0` (1.0.149 locked) | Removed unused declaration; Zed re-export updated transitively to 1.0.154 |
| direct `flate2` | `1.1.8` | Removed unused declaration; transitive build dependency updated to 1.1.11 |
| direct `tar` | `0.4.44` | Removed from manifest and lockfile |
| direct `zip` | `7.0.0`, deflate enabled | Removed from manifest and lockfile |
| `futures` family | 0.3.31 | 0.3.34; removes `pin-utils` 0.1.0 |
| `serde` family | 1.0.228 | 1.0.229 |
| ICU4X family | 2.1.1 / 2.1.2 | 2.4.0 |
| `syn` | 2.0.114 | 2.0.119 and 3.0.9, as required by each upstream macro crate |
| `actions/checkout` | v4 | v7 (released 7.0.1) |
| `actions/setup-node` | v4 | v7 (released 7.1.0) |
| `actions/cache` | v4 | v6 (released 6.1.0) |
| CI Node.js | 20 | 26 LTS |

All Rust sources, tests, and build scripts were checked for dependency use.
`src/lib.rs` only imports the standard library and `zed_extension_api`, including
`zed::serde_json`. Archive downloads and extraction already go through
`zed::download_file`, which runs in Zed's host. Removing unused archive crates
therefore requires no zip 7→8→9 extraction API rewrite. This project does not control
the host's archive dependencies.

`anyhow` and `flate2` remain in the lockfile through the WIT binding generation
toolchain. Their direct declarations were unnecessary; removing those declarations
alone would not remove the transitive copies. The lockfile refresh updates them.

## Upstream guidance read and applied

- [Zed extension development](https://zed.dev/docs/extensions/developing-extensions)
  recommends the latest compatible published API and `wasm32-wasip2`. The
  [crates.io registry](https://crates.io/crates/zed_extension_api) still publishes
  0.7.0. Upstream main's 0.8.0 is marked `publish = false`, so it is not adopted.
- Released [Zed 0.207.3's extension builder](https://github.com/zed-industries/zed/blob/v0.207.3/crates/extension/src/extension_builder.rs)
  already compiles `wasm32-wasip2` components; the same release's
  [API manifest](https://github.com/zed-industries/zed/blob/v0.207.3/crates/extension_api/Cargo.toml)
  uses API 0.7.0. Toolchain targets, build/test paths, release helpers (including
  the bundled `.skill` archive), CI, and contributor docs
  now match that released combination. The artifact comparator accepts both core
  modules and components, recursively ignoring only `name` metadata in nested
  binaries. Component bindings, API version metadata, code, and data remain
  significant. Tests cover differences in nested code and API metadata, section
  length changes, and malformed component nesting.
- [Zed capabilities](https://zed.dev/docs/extensions/capabilities) documents
  `npm:install` and `process:exec`. The manifest now declares the existing
  `css-variable-lsp` npm fallback and PATH version probes. Executable paths vary by
  worktree, so process declarations match any command but only `--version` or
  `-V` arguments. The GitHub download scope remains repository-specific.
- The upstream [Vue language configuration](https://github.com/zed-extensions/vue/blob/main/languages/vue/config.toml)
  names the language `Vue.js`. Its language-ID mapping now uses that exact key
  and the LSP ID `vue`. Manifest tests ensure every registered language has a
  corresponding language ID and the installation/probe capabilities are present.
- [Anyhow release notes](https://github.com/dtolnay/anyhow/releases) cover
  1.0.101–1.0.104, including the `Error::downcast_mut` UB fix in 1.0.103. No
  extension call sites use anyhow; the retained build dependency is updated.
- [Futures changelog](https://github.com/rust-lang/futures-rs/blob/master/CHANGELOG.md)
  covers the 1.71 MSRV increase, standard pinning macros, removal of `pin-utils`,
  soundness fixes, and the syn 3 update. Stable Rust 1.97.1 satisfies the MSRV;
  this extension has no futures API call sites to migrate.
- [Serde 1.0.229](https://github.com/serde-rs/serde/releases/tag/v1.0.229) updates
  its macro dependency to syn 3. [Serde JSON releases](https://github.com/serde-rs/json/releases)
  1.0.150–1.0.154 change invalid enum-key handling, UTF-8 error positions, and
  optional arbitrary-precision/no-std behavior. The extension uses ordinary JSON
  objects, arrays, booleans, and strings via the Zed re-export. Existing settings
  merge and CLI-generation tests exercise those paths.
- [Flate2 changelog](https://github.com/rust-lang/flate2-rs/blob/main/CHANGELOG.md)
  describes decoder/reset/error-progress and flush fixes in 1.1.11. No extension
  code calls flate2 directly; host-side archive extraction is unchanged.
- [ICU4X changelog](https://github.com/unicode-org/icu4x/blob/main/CHANGELOG.md)
  describes Unicode/CLDR data changes, normalizer soundness fixes, and internal
  API changes. These libraries are used transitively by URL parsing in the WIT
  build toolchain; no ICU APIs or persisted collation keys exist in this extension.
- [Checkout migration notes](https://github.com/actions/checkout#whats-new)
  document the Node 24 runtime, runner minimum 2.327.1, changed credential storage,
  and fork protections for `pull_request_target`/`workflow_run`. These workflows
  use hosted runners and ordinary `pull_request`, push, and manual triggers;
  they need no unsafe-checkout opt-in or credential-storage changes.
- [Setup-node migration notes](https://github.com/actions/setup-node#whats-new-in-v7)
  document Node 24, npm cache detection, removal of `always-auth`, and removal of
  the dummy token fallback. This project uses neither registry auth input nor
  `always-auth`; automatic package-manager caching is explicitly disabled because
  the extension repository has no npm lockfile.
- [Cache migration notes](https://github.com/actions/cache#whats-new) document
  Node 24/runner requirements and ESM changes; existing inputs remain supported.
  Hosted Ubuntu/macOS runners satisfy the runtime requirements.
- [Node 26 release notes](https://github.com/nodejs/node/blob/main/doc/changelogs/CHANGELOG_V26.md)
  identify 26 as LTS and note the Linux `libatomic` requirement since Node 25.
  CI uses maintained hosted images. The clean-install test now propagates npm
  failures, requires the fallback executable, defaults its cleanup option safely,
  and uses an isolated ignored temporary directory.

`dtolnay/rust-toolchain@stable` and `huacnlee/zed-extension-action@v2` remain
current for their existing purpose; the latter's latest release is 2.0.0.
The publishing action is not executed as part of validation.

## Advisory verification

An exact-version OSV batch query checked both lockfiles. Baseline: 103 registry
packages, with matches in `anyhow` 1.0.100 (`RUSTSEC-2026-0190`), `pin-utils`
0.1.0 (`RUSTSEC-2026-0441`), `tar` 0.4.44, and `zip` 7.0.0. Migrated: 88 registry
packages, zero advisory matches at verification time.

The tar/zip matches describe dependency versions, not demonstrated extension
exploitation: their archive APIs were unused. `pin-utils` is an unmaintained-crate
advisory. This check is an OSV lookup, not a `cargo audit` invocation or a proof
that Zed's separately built host has no vulnerabilities.

The clean-install test resolved the separately published npm fallback to
`css-variable-lsp` 1.0.22. An isolated lockfile audit found three moderately
affected dependency nodes (`css-variable-lsp`, `glob`, `minimatch`) caused by two
advisories: [glob cancellation bypass](https://github.com/advisories/GHSA-3g3m-2fjr-6pw4)
and [minimatch globparts amplification](https://github.com/advisories/GHSA-g3hv-75wg-m249).
This latest published fallback still needs an upstream dependency update and
release. Zed's npm installation API accepts a package and version, with no
dependency-override parameter; this Rust extension has no npm dependency manifest.
The extension therefore retains the existing latest-version fallback, and the
Rust lockfile's zero-match result does not apply to that separately installed npm
graph. The audit establishes affected versions, not runtime exploitability.

## Validation results

- Stable Rust 1.97.1: formatting, `cargo check --locked --all-targets`, and Clippy
  with `--all-targets -- -D warnings` passed.
- All 29 Rust unit tests, 8 artifact-comparator tests, and 2 manifest tests passed.
- A real optimized `wasm32-wasip2` build produced a 411,524-byte component;
  `test_extension.sh` passed, including the rebuilt artifact comparison.
- Additional validation with wasmparser 0.227.1 accepted both the raw component
  and the 344,790-byte canonicalized component and confirmed the embedded Zed
  API version bytes encode 0.7.0. Raw SHA-256:
  `2ab7ee38038a5316b11f5b174b99ce32146470bd23f24f5d8202f7a02975cde0`.
- Clean installation passed with Node 26.2.0/npm 12.0.2 and the published
  `css-variable-lsp` 1.0.22; its binary and `out/server.js` entrypoint existed,
  and `node --check` accepted the entrypoint. npm advisory results are above.
- Shell syntax, workflow YAML syntax, and `git diff --check` passed.
- The standalone and bundled release helper `build_wasm` functions both rebuilt
  and copied the correct component; no release, commit, tag, or push functions
  were invoked by those helper checks.

## Reproduction

```bash
rustup target add --toolchain 1.97.1 wasm32-wasip2
rustup component add --toolchain 1.97.1 rustfmt clippy
cargo fmt -- --check
CARGO_BUILD_JOBS=2 cargo check --locked --all-targets
CARGO_BUILD_JOBS=2 cargo clippy --locked --all-targets -- -D warnings
CARGO_BUILD_JOBS=2 cargo test --locked --lib
CARGO_BUILD_JOBS=2 ./scripts/build_wasm.sh
cp target/wasm32-wasip2/release/zed_css_variables.wasm extension.wasm
CARGO_BUILD_JOBS=2 ./test_extension.sh
./test_clean_install.sh
```

The checked-in artifact must be regenerated after dependency or target changes;
CI compares the rebuilt component against it without overwriting it first.
Integration scripts verify build/artifact/manifest contracts and npm installation;
they do not install the extension into a running Zed session or exercise a live
GitHub asset download. No desktop settings or published releases are modified.