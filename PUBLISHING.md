# Publishing Guide for zed-css-variables

## Pre-Publishing Checklist

✅ **Version Updated**

- [x] `extension.toml` version: 0.0.7
- [x] CHANGELOG.md updated with release notes

✅ **Code Quality**

- [x] All tests passing (`cargo test --lib`)
- [x] Integration tests passing (`./test_extension.sh`)
- [x] Clean install test passing (`./test_clean_install.sh`)
- [x] Extension builds successfully
- [x] WASM file up to date

**Note:** All tests validate the extension works correctly without requiring Docker.

✅ **Documentation**

- [x] README.md updated with latest features
- [x] Installation instructions clear
- [x] Development setup documented
- [x] Testing instructions included
- [x] Known limitations documented

✅ **Extension Configuration**

- [x] `download_file` capability declared
- [x] LSP version: css-variable-lsp v0.1.5
- [x] Extension metadata complete (name, description, repository)
- [x] License specified (GPL-3.0)

✅ **Git & Repository**

- [x] All changes committed
- [ ] Changes pushed to GitHub
- [ ] Git tags created for version

## Publishing to Zed Extension Marketplace

### Method 1: Via Zed Extension Marketplace (Recommended)

1. **Ensure you're logged in to Zed**

   - Open Zed
   - Sign in with your GitHub account

2. **Publish the extension**

   - The Zed team reviews extensions submitted via GitHub
   - Extensions are typically published from the repository

3. **Repository Requirements**
   - Public GitHub repository ✅
   - Valid `extension.toml` ✅
   - Valid `extension.wasm` ✅
   - Clear README ✅

### Method 2: Manual Distribution

Users can install directly from the repository:

1. Clone the repository
2. Build the extension (see README.md)
3. In Zed: Extensions → Install Dev Extension → Select directory

## Post-Publishing

- [ ] Test installation from marketplace
- [ ] Verify LSP auto-installation works
- [ ] Check that all features work in fresh installation
- [ ] Monitor for user feedback and issues
- [ ] Update documentation if needed

## Updating the Extension

When releasing a new version:

1. Update version in `extension.toml`
2. Update CHANGELOG.md
3. Run all tests
4. Build and update `extension.wasm`
5. Commit changes with descriptive message
6. Push to GitHub
7. Create a git tag: `git tag v0.0.X && git push origin v0.0.X`

## Release Checklist for v0.0.7

- [x] Updated to css-variable-lsp@1.0.5-beta.1
- [x] Added download_file capability
- [x] Created comprehensive test suite
- [x] Updated documentation
- [x] All tests passing
- [x] Code committed
- [ ] Code pushed to GitHub
- [ ] Create git tag v0.0.7
- [ ] Submit to Zed extension marketplace

## Cross-Repo Release Process

This extension depends on **prebuilt binaries** from the LSP repo (`lmn451/css-lsp-rust`).

### Asset Naming Contract

The extension expects assets with these **exact names**:

| Platform        | Asset Name                                 |
| --------------- | ------------------------------------------ |
| macOS aarch64   | `css-variable-lsp-macos-aarch64.tar.gz`    |
| macOS x86_64    | `css-variable-lsp-macos-x86_64.tar.gz`     |
| Linux aarch64   | `css-variable-lsp-linux-aarch64.tar.gz`    |
| Linux x86_64    | `css-variable-lsp-linux-x86_64.tar.gz`     |
| Windows aarch64 | `css-variable-lsp-windows-aarch64.exe.zip` |
| Windows x86_64  | `css-variable-lsp-windows-x86_64.exe.zip`  |

> [!CAUTION]
> Asset names are defined in `src/lib.rs` → `asset_name_for_platform()`. Any changes require a coordinated update in both repos.

### Before Publishing the Extension

1. **Verify LSP release exists** at the tag specified by `CSS_VARIABLES_RELEASE_TAG`
2. **Verify all 6 assets** are available with correct names
3. Run the LSP smoke test (in rust-css-lsp repo):
   ```bash
   ./scripts/smoke-test-release.sh vX.Y.Z
   ```

### Updating to a New LSP Version

1. Update `CSS_VARIABLES_RELEASE_TAG` in `src/lib.rs`
2. Update `CHANGELOG.md` with LSP version
3. Bump version in `extension.toml`
4. Rebuild `extension.wasm`:
   ```bash
   cargo build --release --target wasm32-wasip1
   cp target/wasm32-wasip1/release/zed_css_variables.wasm extension.wasm
   ```
5. Run tests: `./test_extension.sh`
6. Create tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`

## Contact & Support

- **Repository**: https://github.com/lmn451/css-variables-zed
- **Issues**: https://github.com/lmn451/css-variables-zed/issues
- **License**: GPL-3.0
