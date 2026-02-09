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

On first use, the extension downloads a prebuilt `css-variable-lsp` release asset and caches it in
the extension working directory. If the download fails, it falls back to the npm package via Zed's
built-in Node.js runtime. No manual Node.js or npm setup is required.

## Configuration

You can override the lookup globs and folder blacklist via Zed settings. Open
the Settings JSON (Cmd+, then "Open Settings JSON") or a workspace
`.zed/settings.json`, and add:

```json
{
  "lsp": {
    "css-variables": {
      "settings": {
        "cssVariables": {
          "lookupFiles": ["**/*.css", "**/*.scss", "**/*.vue"],
          "blacklistFolders": ["**/dist/**", "**/node_modules/**"],
          "undefinedVarFallback": "info"
        }
      }
    }
  }
}
```

Settings must be nested under the `cssVariables` key.
Provided lists replace the defaults (include any defaults you still want).
`undefinedVarFallback` controls diagnostics when a `var(--name, fallback)` has an
undefined variable; supported values are `warning` (default), `info`, and `off`.

Binary resolution order (first match wins):
1) `lsp.css-variables.binary.path` (can point to a local dev build)
2) Download the pinned Rust release asset and cache it
3) `css-variable-lsp` in PATH
4) Fall back to npm package `css-variable-lsp`

Defaults:

- `lookupFiles`:
  - `**/*.less`
  - `**/*.scss`
  - `**/*.sass`
  - `**/*.css`
  - `**/*.html`
  - `**/*.vue`
  - `**/*.svelte`
  - `**/*.astro`
  - `**/*.ripple`
- `blacklistFolders`:
  - `**/.cache/**`
  - `**/.DS_Store`
  - `**/.git/**`
  - `**/.hg/**`
  - `**/.next/**`
  - `**/.svn/**`
  - `**/bower_components/**`
  - `**/CVS/**`
  - `**/dist/**`
  - `**/node_modules/**`
  - `**/tests/**`
  - `**/tmp/**`

Both settings accept standard glob patterns (including brace expansions like `**/*.{css,scss}`).
Note: these are glob patterns (not gitignore rules). To exclude files inside a directory,
include `/**` at the end (for example `**/dist/**`).

### NPM Package Version (Optional)

To opt into beta releases (used only when falling back to npm), set `npmVersion` in the same `settings` object:

```json
{
  "lsp": {
    "css-variables": {
      "settings": {
        "npmVersion": "beta"
      }
    }
  }
}
```

## LSP Flags & Environment

The extension launches `css-variable-lsp` with `--color-only-variables` and `--stdio`.

Supported LSP flags:

- `--no-color-preview`
- `--color-only-variables`
- `--lookup-files "<glob>,<glob>"`
- `--lookup-file "<glob>"` (repeatable)
- `--ignore-globs "<glob>,<glob>"`
- `--ignore-glob "<glob>"` (repeatable)
- `--path-display=relative|absolute|abbreviated`
- `--path-display-length=N`
- `--undefined-var-fallback=warning|info|off`

Supported environment variables:

- `CSS_LSP_COLOR_ONLY_VARIABLES=1`
- `CSS_LSP_LOOKUP_FILES` (comma-separated globs)
- `CSS_LSP_IGNORE_GLOBS` (comma-separated globs)
- `CSS_LSP_DEBUG=1`
- `CSS_LSP_PATH_DISPLAY=relative|absolute|abbreviated`
- `CSS_LSP_PATH_DISPLAY_LENGTH=1`
- `CSS_LSP_UNDEFINED_VAR_FALLBACK=warning|info|off`

Defaults:

- `path-display`: `relative`
- `path-display-length`: `1`
- `undefined-var-fallback`: `warning`
- LSP lookup globs:
  - `**/*.css`
  - `**/*.scss`
  - `**/*.sass`
  - `**/*.less`
  - `**/*.html`
  - `**/*.vue`
  - `**/*.svelte`
  - `**/*.astro`
  - `**/*.ripple`
- LSP ignore globs:
  - `**/node_modules/**`
  - `**/dist/**`
  - `**/out/**`
  - `**/.git/**`

Zed forwards `cssVariables.lookupFiles` as repeated `--lookup-file` flags and
`cssVariables.blacklistFolders` as repeated `--ignore-glob` flags.

### Completion Path Examples

Assume a variable is defined in `/Users/you/project/src/styles/theme.css` and your workspace root is `/Users/you/project`.

- `--path-display=relative` (default):
  - `Defined in src/styles/theme.css`
- `--path-display=absolute`:
  - `Defined in /Users/you/project/src/styles/theme.css`
- `--path-display=abbreviated --path-display-length=1`:
  - `Defined in s/s/theme.css`
- `--path-display=abbreviated --path-display-length=2`:
  - `Defined in sr/st/theme.css`
- `--path-display=abbreviated --path-display-length=0`:
  - `Defined in src/styles/theme.css`

## Development

### Prerequisites

- Rust with `wasm32-wasip1` target: `rustup target add wasm32-wasip1`

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

# Run clean installation test (validates download capability)
./test_clean_install.sh
```

### Installing Dev Extension

1. Build the extension (see above)
2. Open Zed -> Extensions -> Install Dev Extension
3. Select this directory

### Using a Local LSP Build

To test a local build of `css-lsp-rust`, set `binary.path` in your settings:

```json
{
  "lsp": {
    "css-variables": {
      "binary": {
        "path": "/path/to/css-lsp-rust/target/debug/css-variable-lsp"
      }
    }
  }
}
```

## Known Limitations

- Cascade resolution is best-effort; the LSP does not model DOM nesting or selector combinators.
- Rename operations replace full declarations/usages and may adjust formatting.

### Latest: v0.1.0

- Pins `css-variable-lsp` to v0.1.6
- Adds Linux/Windows ARM64 release asset support
- Adds `undefinedVarFallback` setting for var() fallback diagnostics
- Downloads a prebuilt release asset on first run, falls back to npm if needed
- Runs the server with `--color-only-variables` by default
