# SPEC_TRACEABILITY.md — Phase 3 Requirement Traceability

**Phase:** 3 — VSCode Extension Client
**Status:** CONTRACT_FROZEN
**Last Updated:** 2026-03-20

---

## 1. Purpose

Map every Phase 3 requirement (CLI-XXX, PRV-XXX, CLS-XXX, PKG-XXX) to its
planned implementation module, test reference(s), and related Phase 1+2
baseline requirement (if consuming one).

## 2. Client Requirements (CLI)

| Req ID | Description | Impl Module | Test IDs | Consumes |
| --- | --- | --- | --- | --- |
| CLI-001 | Binary discovery chain | `editors/code/src/server.ts` | T-CLI-002, T-CLI-003, T-CLI-004, T-CLI-005 (`editors/code/src/server.test.ts`) | LSP-001 (stdio) |
| CLI-002 | ServerOptions configuration | `editors/code/src/server.ts` | T-CLI-006 (`editors/code/src/server.test.ts`) | LSP-001, LSP-002 |
| CLI-003 | LanguageClientOptions configuration | `editors/code/src/extension.ts` | T-CLI-001, T-CLI-010 (`editors/code/src/extension.test.ts`) | LSP-003 (full sync) |
| CLI-004 | Extension activation lifecycle | `editors/code/src/extension.ts` | T-CLI-001, T-CLI-019 (`editors/code/src/extension.test.ts`) | LSP-005 (shutdown) |
| CLI-005 | Error recovery and auto-restart | `editors/code/src/server.ts`, `editors/code/src/extension.ts` | T-CLI-017, T-CLI-018 (`editors/code/src/server.test.ts`) | — |
| CLI-006 | Extension configuration schema | `editors/code/package.json` | T-CLI-007, T-CLI-008, T-CLI-009 (`editors/code/src/config.test.ts`) | — |
| CLI-007 | Runtime dependency detection | `editors/code/src/dependencies.ts` | T-CLI-011, T-CLI-012, T-CLI-021 (`editors/code/src/dependencies.test.ts`) | — |
| CLI-008 | Language contribution (TextMate grammar) | `editors/code/syntaxes/tape.tmLanguage.json`, `editors/code/package.json` | T-CLI-013 | LEX-001..LEX-006 (token kinds) |
| CLI-009 | No-server fallback mode | `editors/code/src/extension.ts`, `editors/code/src/server.ts` | T-CLI-005, T-CLI-014, T-CLI-015, T-CLI-016 (`editors/code/src/extension.test.ts`) | — |
| CLI-010 | Configuration change handling | `editors/code/src/config.ts` | T-CLI-008, T-CLI-009, T-CLI-010 (`editors/code/src/config.test.ts`) | — |
| CLI-011 | Status bar indicator | `editors/code/src/status.ts`, `editors/code/src/extension.ts` | T-CLI-020 (`editors/code/src/extension.test.ts`) | — |
| CLI-012 | Targeted suggest triggering | `editors/code/src/extension.ts` | T-CLI-022, T-CLI-023, T-CLI-024 (`editors/code/src/extension.test.ts`) | CMP-005, CMP-006, CMP-009 |

## 3. Preview Requirements (PRV)

| Req ID | Description | Impl Module | Test IDs | Consumes |
| --- | --- | --- | --- | --- |
| PRV-001 | Webview panel creation | `editors/code/src/preview.ts` | T-PRV-001, T-PRV-002 (`editors/code/src/preview.test.ts`) | — |
| PRV-002 | Webview messaging protocol | `editors/code/src/preview.ts` | T-PRV-020 (`editors/code/src/preview.test.ts`) | — |
| PRV-003 | VHS CLI invocation | `editors/code/src/execution.ts`, `editors/code/src/preview.ts` | T-PRV-003 (`editors/code/src/preview.test.ts`) | — |
| PRV-004 | Output artifact discovery | `editors/code/src/utils.ts`, `editors/code/src/execution.ts` | T-PRV-004, T-PRV-005 (`editors/code/src/utils.test.ts`), T-PRV-006, T-PRV-006A (`editors/code/src/execution.test.ts`) | PAR-001 (OutputCommand node) |
| PRV-005 | Auto-refresh on output file change | `editors/code/src/preview.ts` | T-PRV-007, T-PRV-008 (`editors/code/src/preview.test.ts`) | CLI-006 (autoRefresh setting) |
| PRV-006 | Execution cancellation | `editors/code/src/execution.ts`, `editors/code/src/preview.ts` | T-PRV-009, T-PRV-010 (`editors/code/src/preview.test.ts`), T-PRV-011, T-PRV-012 (`editors/code/src/execution.test.ts`) | — |
| PRV-007 | Content Security Policy | `editors/code/src/preview.ts` | T-PRV-013 (`editors/code/src/preview.test.ts`) | — |
| PRV-008 | Theme-aware Webview styling | `editors/code/src/preview.ts`, `editors/code/media/preview.css` | T-PRV-014 (`editors/code/src/preview.test.ts`) | — |
| PRV-009 | Loading and error states | `editors/code/src/preview.ts`, `editors/code/media/preview.css` | T-PRV-015, T-PRV-016, T-PRV-018, T-PRV-019 (`editors/code/src/preview.test.ts`) | — |
| PRV-010 | VHS missing graceful degradation | `editors/code/src/preview.ts` | T-PRV-017 (`editors/code/src/preview.test.ts`) | CLI-007 (dependency check) |

## 4. CodeLens Requirements (CLS)

| Req ID | Description | Impl Module | Test IDs | Consumes |
| --- | --- | --- | --- | --- |
| CLS-001 | CodeLens placement strategy | `editors/code/src/codelens.ts` | T-CLS-001, T-CLS-002, T-CLS-003, T-CLS-004 (`editors/code/src/codelens.test.ts`) | PAR-001 (OUTPUT_COMMAND) |
| CLS-002 | Command registry | `editors/code/src/codelens.ts`, `editors/code/src/extension.ts`, `editors/code/package.json` | T-CLS-005 (`editors/code/src/extension.test.ts`), T-CLS-006, T-CLS-007, T-CLS-008 (`editors/code/src/codelens.test.ts`) | — |
| CLS-003 | Execution state machine | `editors/code/src/execution.ts` | T-CLS-009, T-CLS-010, T-CLS-011, T-CLS-012 (`editors/code/src/execution.test.ts`) | — |
| CLS-004 | Status bar progress indicator | `editors/code/src/codelens.ts`, `editors/code/src/extension.ts`, `editors/code/src/status.ts` | T-CLS-013, T-CLS-014 (`editors/code/src/codelens.test.ts`, `editors/code/src/extension.test.ts`) | CLI-011 (status bar) |
| CLS-005 | CodeLens toggle via configuration | `editors/code/src/codelens.ts` | T-CLS-015, T-CLS-016 (`editors/code/src/codelens.test.ts`) | CLI-006 (codelens.enabled) |
| CLS-006 | CodeLens provider registration | `editors/code/src/codelens.ts`, `editors/code/src/extension.ts` | T-CLS-001, T-CLS-017 (`editors/code/src/codelens.test.ts`, `editors/code/src/extension.test.ts`) | — |
| CLS-007 | Execution output channel | `editors/code/src/execution.ts`, `editors/code/src/extension.ts` | T-CLS-006 (`editors/code/src/execution.test.ts`) | — |
| CLS-008 | Keyboard shortcut binding | `editors/code/package.json` (keybindings) | T-CLS-018 (E2E), manifest contribution asserted in `editors/code/src/codelens.test.ts` | — |
| CLS-009 | Editor context menu | `editors/code/package.json` (menus) | T-CLS-018, T-CLS-019 (E2E), manifest contribution asserted in `editors/code/src/codelens.test.ts` | — |
| CLS-010 | Explorer context menu | `editors/code/package.json` (menus) | Manifest contribution asserted in `editors/code/src/codelens.test.ts` | — |

## 5. Packaging Requirements (PKG)

| Req ID | Description | Impl Module | Test IDs | Consumes |
| --- | --- | --- | --- | --- |
| PKG-001 | Target platform matrix | `.github/workflows/release.yml` | T-PKG-001..T-PKG-007 | — |
| PKG-002 | Rust binary cross-compilation | `.github/workflows/release.yml` | T-PKG-008, T-PKG-009 | LSP-001 (binary) |
| PKG-003 | Extension manifest | `editors/code/package.json` | T-PKG-015 | CLI-006, CLI-008, CLS-002 |
| PKG-004 | esbuild bundle configuration | `editors/code/package.json`, `editors/code/scripts/build.mjs` | T-PKG-010, T-PKG-011 | — |
| PKG-005 | pnpm + vsce compatibility | `editors/code/.vscodeignore`, `editors/code/package.json`, `editors/code/scripts/build.mjs`, `editors/code/src/preview.ts`, `.github/workflows/release.yml` | T-PKG-015 | — |
| PKG-006 | GitHub Actions release workflow | `.github/workflows/release.yml` | T-PKG-016..T-PKG-019 | — |
| PKG-007 | TypeScript CI pipeline | `.github/workflows/extension-ci.yml` | T-PKG-012, T-PKG-013, T-PKG-014 | — |
| PKG-008 | Publishing strategy | `.github/workflows/release.yml` | T-PKG-018 | — |
| PKG-009 | No-server fallback VSIX | `.github/workflows/release.yml` | T-PKG-007 | CLI-009 |
| PKG-010 | Biome configuration | `editors/code/biome.json` | T-PKG-012 | — |
| PKG-011 | Vitest configuration | `editors/code/vitest.config.ts` | T-PKG-014 | — |
| PKG-012 | Rust release profile | `Cargo.toml` (workspace) | T-PKG-009 | — |

## 6. Batch 5 Integration Coverage

| Test ID | Scenario | Coverage | Status |
| --- | --- | --- | --- |
| T-INT3-001 | Full activation → LSP handshake → hover | CLI-001, CLI-002, CLI-003, CLI-004, CLI-011 | Completed |
| T-INT3-002 | CodeLens run → Preview shows result | CLS-001, CLS-002, CLS-003, PRV-001, PRV-002, PRV-003, PRV-004, PRV-009 | Completed |
| T-INT3-003 | No-server mode → CodeLens + Preview work without LSP | CLI-009, CLS-001, CLS-002, PRV-001, PRV-003, PRV-004 | Completed |
| T-INT3-004 | Platform VSIX install → bundled binary activates | CLI-001, PKG-001, PKG-002, PKG-003 | Deferred (MAY) |
| T-INT3-005 | Universal VSIX → no-server mode | CLI-009, PKG-009 | Deferred (MAY) |

## 7. Resolved Design Decisions Traceability

| Decision ID | Spec | Affects Requirements | Affects Tests |
| --- | --- | --- | --- |
| RD-CLI-01 | SPEC_CLIENT.md | CLI-001, CLI-002, CLI-009 | T-CLI-002..T-CLI-005 |
| RD-CLI-02 | SPEC_CLIENT.md | CLI-008 | T-CLI-013 |
| RD-CLI-03 | SPEC_CLIENT.md | CLI-010 | T-CLI-008, T-CLI-009 |
| RD-CLI-04 | SPEC_CLIENT.md | CLI-009 | T-CLI-015, T-CLI-016 |
| RD-CLI-05 | SPEC_CLIENT.md | CLI-007 | T-CLI-011, T-CLI-012, T-CLI-021 |
| RD-PRV-01 | SPEC_PREVIEW.md | PRV-001 | T-PRV-001, T-PRV-002 |
| RD-PRV-02 | SPEC_PREVIEW.md | PRV-003 | T-PRV-015 |
| RD-PRV-03 | SPEC_PREVIEW.md | PRV-004 | T-PRV-004, T-PRV-005 |
| RD-PRV-04 | SPEC_PREVIEW.md | PRV-009 | T-PRV-018, T-PRV-019 |
| RD-CLS-01 | SPEC_CODELENS.md | CLS-001 | T-CLS-001, T-CLS-002 |
| RD-CLS-02 | SPEC_CODELENS.md | CLS-001, CLS-002 | T-CLS-003, T-CLS-007 |
| RD-CLS-03 | SPEC_CODELENS.md | CLS-002, CLS-009 | T-CLS-020 |
| RD-CLS-04 | SPEC_CODELENS.md | CLS-001 | T-CLS-021 |
| RD-PKG-01 | SPEC_PACKAGING.md | PKG-001 | T-PKG-002, T-PKG-003 |
| RD-PKG-02 | SPEC_PACKAGING.md | PKG-012 | T-PKG-009 |
| RD-PKG-03 | SPEC_PACKAGING.md | PKG-005 | T-PKG-020 |
| RD-PKG-04 | SPEC_PACKAGING.md | PKG-003 | — |
| RD-PKG-05 | SPEC_PACKAGING.md | PKG-006, PKG-008 | T-PKG-016, T-PKG-017 |
| RD-PKG-06 | SPEC_PACKAGING.md | PKG-008 | T-PKG-018 |

## 8. Cross-Phase Dependency Index

| Phase 3 Req | Phase 1/2 Baseline | Dependency Type |
| --- | --- | --- |
| CLI-001, CLI-002 | LSP-001 (stdio transport) | Binary interface |
| CLI-002 | LSP-002 (Initialize handshake) | Protocol |
| CLI-003 | LSP-003 (Full sync) | Sync mode |
| CLI-004, CLI-019 | LSP-005 (Shutdown/exit) | Lifecycle |
| CLI-003 | CMP-001 (completionProvider) | Capability |
| CLI-003 | DIA-011 (Unified diagnostic pipeline) | Notification |
| CLI-003 | SAF-006 (Safety diagnostics) | Notification |
| CLI-008 | LEX-001..LEX-006 (Token kinds) | Scope mapping |
| PRV-004 | PAR-001 (OutputCommand node) | AST structure |
| CLS-001 | PAR-001 (OUTPUT_COMMAND kind) | Node position |
| PKG-002 | LSP-001 (Binary) | Artifact |
