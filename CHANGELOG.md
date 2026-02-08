# Changelog

All notable changes to this project will be documented in this file.

## 0.1.0

- **Major**: Extension now automatically downloads the latest `css-variable-lsp` release
- Changed `CSS_VARIABLES_RELEASE_TAG` from pinned version (v0.1.6) to "latest"
- GitHub's `/releases/latest/download/` endpoint automatically resolves to the newest release
- Extension fetches the most recent LSP version on each fresh install without extension updates
- Simplified implementation using hardcoded "latest" tag for reliability
- Updated extension version to 0.1.0

## 0.0.9

- Bump `css-variable-lsp` to v0.1.6
- Add Linux/Windows ARM64 release asset support
- Add undefinedVarFallback setting for var() fallback diagnostics
- Add npm fallback when Rust binary download fails

## 0.0.8

- fix issue with Vue files
- Use the latest `css-variable-lsp` on startup
- Add `npmVersion` setting to opt into beta npm package

## 0.0.7

- Pinned `css-variable-lsp` to v0.1.5
- Download prebuilt release assets instead of npm install

## 0.0.6

- Updated to `css-variable-lsp` v1.0.11

## 0.0.5

- Documentation and release metadata updated for v0.0.5

## 0.0.4

- Updated to `css-variable-lsp` v1.0.5-beta.1
- Added `npm:install` capability declaration in `extension.toml` for proper package installation
- Extension now automatically installs dependencies on fresh Zed installations
- No manual Node.js or npm setup required

## 0.0.3

- **Breaking Change**: Switched from `css-variables-language-server` to `css-variable-lsp` (v1.0.2)
- Fixed path resolution issue that caused "Cannot find module" errors
- Extension now properly uses the npm bin entry via `current_dir()` to locate the language server
- Updated package references in documentation

## 0.0.2

- Integrates the existing `css-variables-language-server` from the VS Code extension:
  - Indexes CSS custom properties from `*.css`, `*.scss`, `*.sass`, `*.less`.
  - Provides completions and color previews for `var(--...)`.
  - Supports hover and go-to-definition for CSS variables across files/languages.
- Bundles `css-variables-language-server` as a local npm dependency, preferring the
  workspace `node_modules/.bin/css-variables-language-server` and falling back to
  a globally installed binary if necessary.
- Known limitations (inherited from upstream server):
  - Does not index variables defined inside HTML `<style>` blocks.
  - If a variable is defined in multiple files, the last scanned definition wins.
