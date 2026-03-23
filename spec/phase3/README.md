# Phase 3: VSCode Extension Client

## Status: Stage B (CONTRACT_FROZEN)

> Frozen on 2026-03-20. All Freeze Candidates resolved.

Phase 3 builds the TypeScript VSCode/Cursor extension that consumes the Rust
LSP binary (Phase 1+2), provides live preview, CodeLens run buttons, and
handles cross-platform packaging for distribution.

## Technology Stack (Locked)

| Tool | Purpose |
| --- | --- |
| pnpm 10.32.1 | Package manager (pinned via `packageManager` field, `--no-dependencies` for vsce) |
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

## Builder Batch Plan

Respecting the dependency graph (WS-1 before WS-2/WS-3, WS-4 independent):

### Batch 1: WS-1 (Client) + WS-4 Scaffold

**Goal:** Establish the extension foundation and project infrastructure.

- Extension project scaffold: `editors/code/` with `package.json`,
  `tsconfig.json`, `biome.json`, `vitest.config.ts`, `esbuild` build script,
  `pnpm` workspace setup, `.vscodeignore`.
- `packageManager: "pnpm@10.32.1"` in `package.json`.
- LSP client bootstrap (`src/extension.ts`, `src/server.ts`): binary discovery
  chain (user → bundled → PATH → no-server), `LanguageClient` configuration,
  activation/deactivation lifecycle.
- Configuration schema (`src/config.ts`): 5 settings, change handling with
  auto-restart (RD-CLI-03).
- TextMate grammar (`syntaxes/tape.tmLanguage.json`): full token mapping.
- Language configuration (`language-configuration.json`).
- Status bar indicator (`src/status.ts`).
- Runtime dependency detection (`src/dependencies.ts`) using `which` package.
- No-server mode with "Don't show again" notification (RD-CLI-04).
- Error recovery with exponential backoff.
- Rename Cargo binary target from `vhs-analyzer-lsp` to `vhs-analyzer`
  (RD-CLI-01).
- WS-4 scaffold: CI skeleton (`.github/workflows/extension-ci.yml`), Biome
  config, Vitest config, esbuild config.
- Unit tests for binary discovery, config handling, dependency detection.

### Batch 2: WS-2 (Preview) — can parallel with Batch 3

**Goal:** Live preview Webview with VHS CLI execution.

- Shared ExecutionManager (`src/execution.ts`): per-file state machine,
  process lifecycle, SIGTERM/SIGKILL cancellation.
- Preview panel (`src/preview.ts`): Webview creation, HTML template with CSP,
  theme-aware styling, loading/complete/error states.
- Messaging protocol: discriminated union types for Extension ↔ Webview
  communication.
- VHS CLI invocation via `child_process.spawn`.
- Output artifact discovery: shared quote-aware regex (RD-PRV-03).
- Auto-refresh via `FileSystemWatcher` with debounce.
- VHS missing graceful degradation.
- Webview stylesheet (`media/preview.css`).
- Unit tests for messaging protocol, output regex, execution state machine,
  cancellation logic.

### Batch 3: WS-3 (CodeLens) — can parallel with Batch 2

**Goal:** Inline run buttons and command registry.

- CodeLens provider (`src/codelens.ts`): file-level at first non-trivial line
  (RD-CLS-01), Output-level with "Run & Preview" (RD-CLS-02).
- Command registration: `runTape`, `previewTape`, `stopRunning`.
- Dynamic CodeLens titles reflecting execution state.
- Context variable `vhs-analyzer.isRunning` (RD-CLS-03).
- Editor and Explorer context menus.
- Keyboard shortcut `Ctrl+Shift+R` / `Cmd+Shift+R`.
- Execution output channel ("VHS Analyzer: Run").
- Unit tests for CodeLens placement, command execution, state transitions,
  toggle behavior.

### Batch 4: WS-4 Completion (Platform VSIX Matrix)

**Goal:** Cross-platform packaging and CI/CD.

- Rust release profile: `opt-level = "s"`, `lto = true`, `strip = true`,
  `codegen-units = 1` (RD-PKG-02).
- Release workflow (`.github/workflows/release.yml`): lint-and-test →
  build-rust (6 targets) → package-vsix (6 platform + 1 universal) → publish.
- `vsce package --target {platform} --no-dependencies` for each target.
- Universal VSIX (no binary, no-server mode).
- Dual publishing: `vsce publish` + `npx ovsx publish` (RD-PKG-06).
- GitHub Release asset upload (7 VSIX files).
- Pre-release detection from tag format (RD-PKG-05).
- `.vscodeignore` verification.

### Batch 5: Integration Test + Closeout

**Goal:** End-to-end verification and documentation.

- Integration tests: full activation → LSP handshake → hover; CodeLens run →
  Preview result; no-server mode CodeLens + Preview.
- E2E tests (MAY): `@vscode/test-electron` for context menus, settings UI.
- CHANGELOG.md for v0.3.0.
- Extension icon and README for marketplace.
- Verify all T-INT3 scenarios pass.
- Close out Phase 3 in `trace/phase3/` and root `STATUS.yaml`.

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
editors/code/
├── package.json            (manifest, contributes, scripts, packageManager)
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
│   └── vhs-analyzer        (bundled binary, platform-specific VSIX only)
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

## Frozen Deliverables

- [x] SPEC_CLIENT.md — 11 requirements, 5 resolved design decisions (CONTRACT_FROZEN)
- [x] SPEC_PREVIEW.md — 10 requirements, 4 resolved design decisions (CONTRACT_FROZEN)
- [x] SPEC_CODELENS.md — 10 requirements, 4 resolved design decisions (CONTRACT_FROZEN)
- [x] SPEC_PACKAGING.md — 12 requirements, 6 resolved design decisions (CONTRACT_FROZEN)
- [x] SPEC_TEST_MATRIX.md — 87 test scenarios across 5 domains
- [x] SPEC_TRACEABILITY.md — full requirement-to-implementation mapping
