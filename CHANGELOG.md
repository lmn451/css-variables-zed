# Changelog

All notable changes to this project will be documented in this file.

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
