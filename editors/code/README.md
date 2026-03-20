# VHS Analyzer

VHS Analyzer adds language tooling and execution workflows for VHS `.tape`
files in VS Code, Cursor, and other editors that consume VS Code extensions.

## Features

- Language server features when a `vhs-analyzer` binary is available: hover
  documentation, completions, diagnostics, and formatting.
- Inline CodeLens actions to run the current tape or run and open the preview
  in one step.
- Side-by-side preview for GIF, MP4, and WebM outputs with progress updates,
  retry support, and optional auto-refresh.
- No-server fallback for the universal package: syntax highlighting, CodeLens,
  and Preview stay available even without the bundled LSP binary.

## Installation

- Prefer the platform-specific marketplace package for full language server
  support. Those packages bundle `server/vhs-analyzer[.exe]` inside the VSIX.
- The universal package is intended for unsupported platforms or custom builds.
  Universal version — install the platform-specific version for full language
  server support.
- Preview and Run features require `vhs`, `ttyd`, and `ffmpeg` on your
  `PATH`.

## Commands

- `VHS: Run Tape` runs the current tape file without opening the preview.
- `VHS: Run & Preview` runs the tape and opens or refreshes the side-by-side
  preview.
- `VHS: Stop` cancels the active VHS execution for the current file.

## Settings

- `vhs-analyzer.server.path` points the extension at a custom
  `vhs-analyzer` binary.
- `vhs-analyzer.server.args` adds extra command-line flags when the LSP server
  starts.
- `vhs-analyzer.trace.server` controls protocol tracing in the dedicated trace
  output channel.
- `vhs-analyzer.preview.autoRefresh` toggles preview auto-refresh when the
  rendered artifact changes on disk.
- `vhs-analyzer.codelens.enabled` shows or hides inline run buttons.

## Development

- `pnpm install`
- `pnpm run build`
- `pnpm run test`
- `cargo build --release -p vhs-analyzer-lsp --locked`

## License

MIT
