# Phase 3: VSCode Extension Client

## Status: Not Started (depends on Phase 1 + Phase 2 freeze)

Phase 3 builds the TypeScript VSCode/Cursor extension that consumes the Rust
LSP binary, provides live preview, CodeLens, and handles cross-platform packaging.

## Work Streams

```txt
WS-1: Client       (SPEC_CLIENT.md)    — vscode-languageclient, binary discovery/launch
WS-2: Preview      (SPEC_PREVIEW.md)   — side-by-side Webview, file watcher, auto-refresh
WS-3: CodeLens     (SPEC_CODELENS.md)  — inline ▶ Run buttons, VHS CLI invocation
WS-4: Packaging    (SPEC_PACKAGING.md) — platform VSIX, GitHub Actions matrix, no-server fallback
```

## Dependency Graph

```txt
WS-1 (Client)
  ├──> WS-2 (Preview)
  └──> WS-3 (CodeLens)

WS-4 (Packaging) — independent, MAY run in parallel with WS-2/WS-3
```

WS-1 MUST complete before WS-2 and WS-3.
WS-4 MAY start once the LSP binary is buildable (after Phase 1).

## Code Location

```txt
editors/vscode/
├── package.json
├── tsconfig.json
├── src/
│   ├── extension.ts      (WS-1: activation, LSP client bootstrap)
│   ├── preview.ts         (WS-2: Webview panel, file watcher)
│   └── codelens.ts        (WS-3: CodeLens provider)
└── .github/
    └── workflows/
        └── release.yml    (WS-4: matrix build, vsce package --target)
```
