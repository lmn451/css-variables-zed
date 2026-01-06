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

On first use, the extension installs `css-variable-lsp@1.0.11` via Zed's `npm:install` capability using Zed's built-in Node.js runtime. No manual Node.js or npm setup is required.

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
          "blacklistFolders": ["**/dist/**", "**/node_modules/**"]
        }
      }
    }
  }
}
```

Settings must be nested under the `cssVariables` key.
Provided lists replace the defaults (include any defaults you still want).

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

Supported environment variables:

- `CSS_LSP_COLOR_ONLY_VARIABLES=1`
- `CSS_LSP_LOOKUP_FILES` (comma-separated globs)
- `CSS_LSP_IGNORE_GLOBS` (comma-separated globs)
- `CSS_LSP_DEBUG=1`
- `CSS_LSP_PATH_DISPLAY=relative|absolute|abbreviated`
- `CSS_LSP_PATH_DISPLAY_LENGTH=1`

Defaults:

- `path-display`: `relative`
- `path-display-length`: `1`
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

### Latest: v0.0.6

- Pins `css-variable-lsp` to v1.0.11
- Uses `npm:install` for automatic dependency setup
- Runs the server with `--color-only-variables` by default
