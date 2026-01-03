# CSS Variables (LSP) for Zed

Project-wide CSS custom properties (variables) support for Zed, powered by `css-variable-lsp`.

## Features

- Workspace indexing of CSS variables across `.css`, `.scss`, `.sass`, `.less`, and HTML `<style>` blocks / inline styles.
- Context-aware completion for `var(--...)` and CSS property values.
- Hover that shows cascade-ordered definitions (`!important`, specificity, source order).
- Go to definition and find references for CSS variables.
- Color decorations on `var(--...)` usages (the extension runs the LSP with `--color-only-variables`).
- Works in CSS, SCSS, Sass, Less, HTML, JavaScript/TypeScript (JSX/TSX), Svelte, Vue, Astro, and PostCSS.

## Installation

1. Open Zed
2. Go to Extensions (Cmd+Shift+X or Ctrl+Shift+X)
3. Search for "CSS Variables"
4. Click Install

On first use, the extension installs `css-variable-lsp@1.0.7` via Zed's `npm:install` capability using Zed's built-in Node.js runtime. No manual Node.js or npm setup is required.

## Configuration

You can override the lookup globs and folder blacklist via Zed settings:

```json
{
  "lsp": {
    "css_variables": {
      "settings": {
        "cssVariables": {
          "lookupFiles": ["**/*.css", "**/*.scss", "**/*.vue"],
          "blacklistFolders": ["**/dist", "**/node_modules"]
        }
      }
    }
  }
}
```

For convenience, you can also place `lookupFiles` and `blacklistFolders` at the top level of `settings` and the extension will nest them under `cssVariables`.

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
2. Open Zed -> Extensions -> Install Dev Extension
3. Select this directory

## Known Limitations

- Cascade resolution is best-effort; the LSP does not model DOM nesting or selector combinators.
- Rename operations replace full declarations/usages and may adjust formatting.

### Latest: v0.0.4

- Pins `css-variable-lsp` to v1.0.7
- Uses `npm:install` for automatic dependency setup
- Runs the server with `--color-only-variables` by default
