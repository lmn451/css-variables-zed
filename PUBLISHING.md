# Publishing Guide for zed-css-variables

## Pre-Publishing Checklist

✅ **Version Updated**
- [x] `extension.toml` version: 0.0.4
- [x] `package.json` version: 0.0.4
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
- [x] `npm:install` capability declared
- [x] LSP version: css-variable-lsp@1.0.5-beta.1
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

1. Update version in `extension.toml` and `package.json`
2. Update CHANGELOG.md
3. Run all tests
4. Build and update `extension.wasm`
5. Commit changes with descriptive message
6. Push to GitHub
7. Create a git tag: `git tag v0.0.X && git push origin v0.0.X`

## Release Checklist for v0.0.4

- [x] Updated to css-variable-lsp@1.0.5-beta.1
- [x] Added npm:install capability
- [x] Created comprehensive test suite
- [x] Updated documentation
- [x] All tests passing
- [x] Code committed
- [ ] Code pushed to GitHub
- [ ] Create git tag v0.0.4
- [ ] Submit to Zed extension marketplace

## Contact & Support

- **Repository**: https://github.com/lmn451/css-variables-zed
- **Issues**: https://github.com/lmn451/css-variables-zed/issues
- **License**: GPL-3.0
