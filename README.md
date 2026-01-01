# CSS Variables (LSP) for Zed

Project-wide CSS custom properties (variables) support for Zed, powered by [`css-variable-lsp`](https://github.com/vunguyentuan/vscode-css-variables).

## Features

- 🎨 Project-wide indexing of `--var`s defined in `.css`, `.scss`, `.sass`, `.less`
- ✨ Autocomplete and color previews for `var(--...)`
- 🔍 Hover and go to definition for CSS variables
- 🚀 Works across HTML / JS / TS / Svelte / Vue etc. where `var(--...)` is used
- 🔧 Zero configuration required - works out of the box!

## Installation

1. Open Zed
2. Go to Extensions (Cmd+Shift+X or Ctrl+Shift+X)
3. Search for "CSS Variables"
4. Click Install

**That's it!** No Node.js or npm installation required. The extension automatically:
- Uses Zed's built-in Node.js runtime
- Installs `css-variable-lsp@1.0.5-beta.1` on first use
- Manages all dependencies automatically

## Development

### Prerequisites

- Rust with `wasm32-wasip1` target: `rustup target add wasm32-wasip1`
- Node.js and npm (for testing only)

### Building

```bash
# Build the extension
cargo build --release --target wasm32-wasip1

# Copy WASM to extension root
cp target/wasm32-wasip1/release/zed_css_variables.wasm extension.wasm
```

### Testing

The extension includes comprehensive automated tests:

```bash
# Run Rust unit tests
cargo test --lib

# Run integration tests
./test_extension.sh

# Run clean installation test (validates npm package installation)
./test_clean_install.sh
```

### Installing Dev Extension

1. Build the extension (see above)
2. Open Zed → Extensions → Install Dev Extension
3. Select this directory

## Known Limitations

- Only CSS/SCSS/LESS/SASS files are scanned; variables defined inside HTML `<style>` blocks are not indexed (this matches upstream behaviour)
- If a variable is defined in multiple files, the last scanned definition wins

## Version History

See [CHANGELOG.md](CHANGELOG.md) for detailed version history.

### Latest: v0.0.4

- Updated to `css-variable-lsp` v1.0.5-beta.1
- Added `npm:install` capability for proper package management
- Automated dependency installation
- Comprehensive test suite added
- Zero manual setup required
