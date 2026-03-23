# Changelog

## 0.1.0

- First public release baseline for VHS Analyzer.
- Before the first public release, private development used internal milestone
  versions (`0.2.0` for the Rust workspace and `0.3.0` for the extension).
  Both version lines were normalized to `0.1.0` before the first public
  release so the project starts from a clean external release baseline.

- Added the initial VHS Analyzer VS Code extension with bundled language server
  activation, hover, completion, diagnostics, formatting, and no-server
  fallback behavior.
- Added side-by-side Preview and CodeLens workflows for running VHS tapes and
  rendering GIF, MP4, or WebM artifacts from the editor.
- Added platform-specific and universal VSIX packaging, release automation, and
  dual publishing for VS Code Marketplace and Open VSX.
