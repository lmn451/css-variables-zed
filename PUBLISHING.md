# Publishing Guide for zed-css-variables

## Pre-Publishing Checklist

✅ **Version Updated**
- [x] `extension.toml` version: 0.0.9
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
- [x] `download_file` capability declared (primary: Rust binary)
- [x] `npm:install` fallback available
- [x] LSP version: css-variable-lsp v0.1.6 (Rust) / latest (npm fallback)
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
   - Open Extensions panel (Cmd+Shift+X)
   - Click on the installed dev extension
   - Click "Publish" (if available)

### Method 2: Via zed-industries/extensions Repository

1. **Fork the extensions repository**

   ```bash
   git clone https://github.com/zed-industries/extensions
   cd extensions
   ```

2. **Add your extension as a submodule**

   ```bash
   git submodule add https://github.com/lmn451/css-variables-zed extensions/css-variables
   ```

3. **Create a pull request**
   - Push your fork
   - Open PR against zed-industries/extensions

## Version Bumping Process

1. Update version in `extension.toml`
2. Update CHANGELOG.md with release notes
3. Rebuild WASM:
   ```bash
   cargo build --release --target wasm32-wasip1
   cp target/wasm32-wasip1/release/zed_css_variables.wasm extension.wasm
   ```
4. Run tests:
   ```bash
   cargo test --lib
   ./test_extension.sh
   ./test_clean_install.sh
   ```
5. Commit and tag:
   ```bash
   git add -A
   git commit -m "release: v0.0.X"
   git tag v0.0.X
   git push && git push --tags
   ```

## Testing Before Publish

```bash
# Run all tests
cargo test --lib
./test_extension.sh
./test_clean_install.sh

# Test in Zed
# 1. Open Zed
# 2. Extensions → Install Dev Extension
# 3. Select this directory
# 4. Open a project with CSS files
# 5. Verify completions, hover, go-to-definition work
```

## Rollback Process

If issues are discovered after publishing:

1. Revert to previous version in `extension.toml`
2. Rebuild and republish
3. Document issue in CHANGELOG.md
