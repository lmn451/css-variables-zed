---
name: zed-extension-release
description: Automate Zed extension releases for this repo. Use when bumping the extension version, updating CHANGELOG/README/PUBLISHING, rebuilding extension.wasm, running tests, committing/tagging, and pushing a vX.Y.Z tag to trigger the Zed extensions registry update workflow.
---

# Zed Extension Release

## Overview

Automate version bumps and release chores for the Zed extension in this repo using the bundled script.

## Workflow

1. Pick the new version and write release notes.
2. Run the script to update `extension.toml`, `CHANGELOG.md`, `README.md` (Latest), and `PUBLISHING.md`.
3. Optionally build `extension.wasm` and run tests.
4. Review `git status` and diffs.
5. Commit, tag, and push to trigger the registry update workflow.
6. Verify the GitHub Action run and merge the PR in `zed-industries/extensions`.

## Script

Use `scripts/zed_extension_release.py` from the repo root.

### Common usage

Bump files only:

```bash
python3 /Users/applesucks/.codex/skills/zed-extension-release/scripts/zed_extension_release.py 0.0.9 \
  --note "Fix Vue file issue" \
  --note "Improve hover descriptions"
```

Full release (build + tests + commit + tag + push):

```bash
python3 /Users/applesucks/.codex/skills/zed-extension-release/scripts/zed_extension_release.py 0.0.9 \
  --notes-file release-notes.txt \
  --build \
  --run-tests \
  --commit \
  --tag \
  --push
```

### Behavior

- Updates `extension.toml` version.
- Inserts a new top section in `CHANGELOG.md` using provided notes.
- Updates README `### Latest: vX.Y.Z` if present.
- Updates release checklist/version fields in `PUBLISHING.md` if present.
- Updates hardcoded version checks in release tests when present (e.g., `test_extension.sh`, `.github/workflows/test.yml`).
- `--build` runs `cargo build --release --target wasm32-wasip1` and copies to `extension.wasm`.
- `--run-tests` runs `cargo test --lib`, `./test_extension.sh`, and `./test_clean_install.sh`.
- `--commit` fails on unrelated changes unless `--allow-dirty` is used.
- `--tag` creates `v<version>`; `--push` pushes commit + tag to `origin` (override with `--remote`).

