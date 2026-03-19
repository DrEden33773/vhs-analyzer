# Phase 3: VSCode Extension Client

## Status: Stage A (Exploratory Design)

Phase 3 builds the TypeScript VSCode/Cursor extension that consumes the Rust
LSP binary (Phase 1+2), provides live preview, CodeLens run buttons, and
handles cross-platform packaging for distribution.

## Technology Stack (Locked)

| Tool | Purpose |
| --- | --- |
| pnpm | Package manager (strict isolation, `--no-dependencies` for vsce) |
| esbuild | Bundler (single-file CJS bundle, `--external:vscode`) |
| Biome | Lint + Format (single `biome.json`, Rust-native speed) |
| Vitest | Test framework (native ESM/TS, esbuild-aligned) |
| tsc --noEmit | Type checking (strict mode, all strict flags) |
| @vscode/vsce | Extension packager (`--target` for platform VSIX) |
| vscode-languageclient v9 | LSP client library |
| VSCode engine ^1.85.0 | Minimum VSCode version |
| Node.js >=18 | Runtime target |

## Work Streams

```text
WS-1: Client       (SPEC_CLIENT.md)    — LSP client bootstrap, binary discovery, config schema, TextMate grammar
WS-2: Preview      (SPEC_PREVIEW.md)   — Webview panel, VHS CLI invocation, messaging protocol, auto-refresh
WS-3: CodeLens     (SPEC_CODELENS.md)  — Inline ▶ Run buttons, command registry, execution state machine
WS-4: Packaging    (SPEC_PACKAGING.md) — Platform VSIX matrix, cross-compile, GitHub Actions CI/CD, publishing
```

## Dependency Graph

```text
Phase 1+2 (Frozen)
  │
  ├──▶ WS-1 (Client) ← consumes LSP binary, stdio transport, server capabilities
  │      │
  │      ├──▶ WS-2 (Preview) ← depends on activation lifecycle, config schema
  │      │       │
  │      │       └──── shares ExecutionManager ────┐
  │      │                                         │
  │      └──▶ WS-3 (CodeLens) ← depends on activation, ◀─┘ shares VHS execution engine
  │
  └──▶ WS-4 (Packaging) ← packages Rust binary into platform VSIX
              │
              └── MAY run in parallel with WS-2/WS-3
```

**Ordering constraints:**

- WS-1 MUST complete before WS-2 and WS-3 (they depend on extension activation
  lifecycle, configuration schema, and no-server mode definition).
- WS-2 and WS-3 share an `ExecutionManager` for VHS CLI process coordination.
  They MAY be developed in parallel but MUST agree on the ExecutionManager
  interface.
- WS-4 MAY start as soon as the Rust binary is buildable (Phase 1 completion)
  and the extension manifest (WS-1) is defined. WS-4 is independent of WS-2/WS-3
  feature completion.

## Cross-Phase Consumption

| Phase 3 Spec | Consumes From | What |
| --- | --- | --- |
| SPEC_CLIENT.md | phase1/SPEC_LSP_CORE.md | stdio transport, InitializeResult capabilities, shutdown/exit lifecycle |
| SPEC_CLIENT.md | phase2/SPEC_COMPLETION.md | completionProvider capability (client receives, does not re-implement) |
| SPEC_CLIENT.md | phase2/SPEC_DIAGNOSTICS.md | publishDiagnostics notifications |
| SPEC_CLIENT.md | phase2/SPEC_SAFETY.md | Safety diagnostics with `safety/*` codes |
| SPEC_CLIENT.md | phase1/SPEC_LEXER.md | Token kinds for TextMate grammar scope mapping |
| SPEC_CODELENS.md | phase1/SPEC_PARSER.md | OUTPUT_COMMAND node kind for CodeLens placement |
| SPEC_PREVIEW.md | VHS CLI | Native rendering via `vhs` child process |
| SPEC_PACKAGING.md | Phase 1+2 Rust binary | Cross-compiled LSP server binary bundled in VSIX |

## Code Location

```text
editors/vscode/
├── package.json            (manifest, contributes, scripts)
├── tsconfig.json           (strict mode, noEmit)
├── biome.json              (lint + format config)
├── vitest.config.ts        (test config)
├── language-configuration.json  (bracket matching, comments)
├── .vscodeignore           (exclude src/, node_modules/ from VSIX)
├── syntaxes/
│   └── tape.tmLanguage.json    (TextMate grammar)
├── src/
│   ├── extension.ts        (WS-1: activation, LSP client bootstrap)
│   ├── server.ts           (WS-1: binary discovery, server options)
│   ├── config.ts           (WS-1: configuration schema, change handling)
│   ├── preview.ts          (WS-2: Webview panel, messaging, VHS invocation)
│   ├── codelens.ts         (WS-3: CodeLens provider, command registration)
│   ├── execution.ts        (WS-2+3: shared ExecutionManager)
│   ├── status.ts           (WS-1: status bar indicator)
│   └── dependencies.ts     (WS-1: runtime dependency detection)
├── media/
│   └── preview.css         (WS-2: Webview stylesheet)
├── server/
│   └── vhs-analyzer-lsp    (bundled binary, platform-specific VSIX only)
└── dist/
    └── extension.js        (esbuild bundle output)
```

## Requirement ID Prefixes

| Prefix | Spec File | Domain |
| --- | --- | --- |
| CLI-XXX | SPEC_CLIENT.md | LSP Client Bootstrapping |
| PRV-XXX | SPEC_PREVIEW.md | Live Preview Webview |
| CLS-XXX | SPEC_CODELENS.md | CodeLens & Commands |
| PKG-XXX | SPEC_PACKAGING.md | Platform Packaging & CI/CD |

## Stage A Deliverables

- [x] SPEC_CLIENT.md — 11 requirements, 4 design options, 5 freeze candidates
- [x] SPEC_PREVIEW.md — 10 requirements, 4 design options, 4 freeze candidates
- [x] SPEC_CODELENS.md — 10 requirements, 4 design options, 4 freeze candidates
- [x] SPEC_PACKAGING.md — 12 requirements, 4 design options, 6 freeze candidates

## Stage B (Next)

Freeze all specs: resolve Freeze Candidates, assign MUST/SHOULD/MAY contracts,
produce SPEC_TEST_MATRIX.md and SPEC_TRACEABILITY.md, then hand off to Builder.
