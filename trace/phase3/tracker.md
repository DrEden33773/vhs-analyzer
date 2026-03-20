# Phase 3 Execution Tracker

Phase: VSCode Extension Client (Client, Preview, CodeLens, Packaging)
Status: Completed — all 5 batches completed
Started: 2026-03-19
Completed: 2026-03-20

## 1. Stage Progress

| Stage | Owner | Status | Completed | Deliverables |
| --- | --- | --- | --- | --- |
| Architect Stage A | Claude | Completed | 2026-03-19 | 4 exploratory specs + updated README (43 reqs, 19 FCs) |
| Architect Stage B | Claude | Completed | 2026-03-20 | 6 frozen specs + test matrix + traceability |
| Builder | Builder | Completed | 2026-03-20 | 5 batches completed, integration closeout done |

## 2. Builder Batch Plan (5 Batches)

| Batch | Name | WS | Location | Requirements | Depends On | Milestone | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | LSP Client + Project Scaffold | WS-1 + WS-4 | editors/code | CLI-* + PKG scaffold | — | Extension activates | completed |
| 2 | Live Preview Webview | WS-2 | editors/code | PRV-* | B1 | Preview renders GIF | completed |
| 3 | CodeLens & Commands | WS-3 | editors/code | CLS-* | B1 | Run button works | completed |
| 4 | Platform Packaging & CI/CD | WS-4 | editors/code + .github | PKG-* | B2, B3 | VSIX builds for all targets | completed |
| 5 | Integration + Closeout | — | editors/code | T-INT3 | B4 | Phase 3 complete | completed |

## 3. Dependency Constraints

- B1 MUST complete before B2 and B3 (WS-1 Client is the foundation).
- B2 and B3 are independent of each other (MAY run in parallel).
- B4 depends on B2 and B3 (packaging requires all features).
- B5 MUST be last (integration test + closeout).

## 4. Technology Stack

| Layer | Tool | Version |
| --- | --- | --- |
| Package manager | pnpm | latest |
| Bundler | esbuild | latest |
| Lint + Format | Biome | latest |
| Test framework | Vitest | latest |
| Type checking | tsc --noEmit | strict mode |
| Extension packager | @vscode/vsce | latest |
| LSP client | vscode-languageclient | 9.x |
| VSCode engine | ^1.85.0 | minimum |
| Node.js | >=18.x | LTS |

## 5. Builder Batch Records

Builder appends one record per completed batch below this line.

### Batch 1 — Completed (2026-03-20)

- Scope: WS-1 client foundation plus WS-4 scaffold items required for local development.
- Requirements: CLI-001 through CLI-011, PKG-003, PKG-004, PKG-005, PKG-007, PKG-010, PKG-011.
- Implementation:
  - Created the `editors/code/` TypeScript extension project with `pnpm`, `esbuild`, `Biome`, `Vitest`, strict `tsconfig`, `.vscodeignore`, language configuration, and a JSON-authored TextMate grammar.
  - Implemented binary discovery, stdio server options, activation and deactivation lifecycle, no-server mode with globalState suppression, configuration change routing, runtime dependency prompts, and a status bar controller.
  - Renamed the Rust binary target to `vhs-analyzer`, accepted `--stdio` as a no-op, and updated Rust integration tests to the new binary name.
- Tests:
  - TypeScript: `pnpm run test` → 22/22 passing (`server.test.ts`, `config.test.ts`, `dependencies.test.ts`, `extension.test.ts`).
  - Rust baseline: `cargo test --workspace --all-targets --locked` → passing after binary rename updates.
- Quality gate:
  - `pnpm run lint` passed.
  - `pnpm run typecheck` passed.
  - `pnpm run test` passed.
  - `pnpm run build` passed.
  - `cargo test --workspace --all-targets --locked` passed.
- Notes:
  - The extension build currently produces a single `dist/extension.js` bundle at 358.6 KB, within the packaging target budget.
  - `vscode-languageclient/node` runtime loading is deferred behind a dynamic import so Vitest can unit-test the controller without a real VSCode host.

### Batch 2 — Completed (2026-03-20)

- Scope: WS-2 live preview plus the shared execution and output-path utilities required by the preview pipeline.
- Requirements: PRV-001 through PRV-010.
- Implementation:
  - Added `editors/code/src/utils.ts` with the shared quote-aware `Output` directive regex and output-path extraction helper.
  - Added `editors/code/src/execution.ts` with per-file execution state tracking, workspace-aware working-directory resolution, single-execution enforcement, and SIGTERM/SIGKILL cancellation.
  - Added `editors/code/src/preview.ts` with `PreviewManager`, `PreviewPanel`, typed Webview messaging, auto-refresh debounce, theme forwarding, VHS missing handling, and a CSP-protected HTML template.
  - Added `editors/code/media/preview.css` with theme-aware preview styling for prompt, loading, complete, and error states.
- Tests:
  - TypeScript: `pnpm run test` → 42/42 passing (`server.test.ts`, `config.test.ts`, `dependencies.test.ts`, `extension.test.ts`, `utils.test.ts`, `execution.test.ts`, `preview.test.ts`).
  - Rust regression: `cargo test --workspace --all-targets --locked` → 225/225 passing.
- Quality gate:
  - `pnpm run lint` passed.
  - `pnpm run typecheck` passed.
  - `pnpm run test` passed.
  - `pnpm run build` passed.
  - `cargo fmt --all -- --check` passed.
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` passed.
  - `cargo test --workspace --all-targets --locked` passed.
- Notes:
  - Preview panel reuse is keyed by tape file path, matching the one-panel-per-file requirement.
  - Auto-refresh uses a 500ms debounce and cache-busting query strings to avoid flicker while VHS writes output incrementally.
  - The shared execution layer is ready for Batch 3 CodeLens command reuse without reopening Batch 2 design decisions.

### Batch 3 — Completed (2026-03-20)

- Scope: WS-3 CodeLens provider, command registry, execution feedback wiring, and editor manifest contributions.
- Requirements: CLS-001 through CLS-010.
- Implementation:
  - Added `editors/code/src/codelens.ts` with file-level and Output-level CodeLens computation, dynamic run titles, configuration and document refresh triggers, command registration, and execution-to-status-bar binding.
  - Extended `editors/code/src/execution.ts` with timestamped run output logging and reveal-on-error behavior for a dedicated `VHS Analyzer: Run` output channel.
  - Extended `editors/code/src/preview.ts` so Output-level CodeLens actions can target a specific artifact path while keeping one preview panel per tape file.
  - Updated `editors/code/src/extension.ts` to register the CodeLens provider and commands in both server and no-server modes, and updated `editors/code/package.json` with commands, menus, and the tape-only run shortcut.
- Tests:
  - TypeScript: `pnpm run test` → 59/59 passing (`server.test.ts`, `config.test.ts`, `dependencies.test.ts`, `extension.test.ts`, `utils.test.ts`, `execution.test.ts`, `preview.test.ts`, `codelens.test.ts`).
  - Rust regression: `cargo test --workspace --all-targets --locked` → 225/225 passing.
- Quality gate:
  - `pnpm run lint` passed.
  - `pnpm run typecheck` passed.
  - `pnpm run test` passed.
  - `pnpm run build` passed.
  - `cargo fmt --all -- --check` passed.
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` passed.
  - `cargo test --workspace --all-targets --locked` passed.
- Notes:
  - File-level CodeLens placement now skips leading blank and comment lines, matching RD-CLS-01 without requiring the LSP server.
  - Output-level CodeLens actions can preview a specific artifact path while file-level "Run & Preview" continues to default to the first `Output`.
  - The extension bundle size is now 377.9 KB after Batch 3 wiring, still within the packaging target budget.

### Batch 4 — Completed (2026-03-20)

- Scope: WS-4 packaging completion, release automation, and final VSIX asset shaping for the existing extension features.
- Requirements: PKG-001, PKG-002, PKG-003, PKG-004, PKG-005, PKG-006, PKG-008, PKG-009, PKG-012.
- Implementation:
  - Added workspace-level Rust release optimizations in `Cargo.toml` and created `.github/workflows/release.yml` with the lint-and-test → build-rust → package-vsix → publish pipeline, six Rust targets, seven VSIX outputs, pre-release detection, dual publishing, and GitHub Release asset upload.
  - Replaced the Preview panel's runtime stylesheet file dependency with a bundled raw CSS import backed by `editors/code/scripts/build.mjs`, keeping `media/preview.css` as source while removing it from the shipped VSIX.
  - Tightened the extension packaging boundary via `editors/code/.vscodeignore`, added `editors/code/LICENSE` and `editors/code/CHANGELOG.md`, and updated `.github/workflows/extension-ci.yml` to install pnpm before `actions/setup-node` cache resolution while still verifying the pinned pnpm version.
- Tests:
  - TypeScript: `pnpm run test` → 59/59 passing (`server.test.ts`, `config.test.ts`, `dependencies.test.ts`, `extension.test.ts`, `utils.test.ts`, `execution.test.ts`, `preview.test.ts`, `codelens.test.ts`).
  - Rust regression: `cargo test --workspace --all-targets --locked` → 225/225 passing.
  - Packaging smoke checks: `pnpm exec vsce ls --no-dependencies`, `pnpm exec vsce package --no-dependencies`, and `cargo build --release -p vhs-analyzer-lsp --locked` → passed.
- Quality gate:
  - `pnpm run lint` passed.
  - `pnpm run typecheck` passed.
  - `pnpm run test` passed.
  - `pnpm run build` passed.
  - `pnpm exec vsce ls --no-dependencies` passed.
  - `pnpm exec vsce package --no-dependencies` passed.
  - `cargo fmt --all -- --check` passed.
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` passed.
  - `cargo test --workspace --all-targets --locked` passed.
  - `cargo build --release -p vhs-analyzer-lsp --locked` passed.
- Notes:
  - The final bundled extension size is 379.41 KB, still below the 500 KB packaging target.
  - Local VSIX smoke checks now contain only `package.json`, `language-configuration.json`, `README.md`, `LICENSE`, `CHANGELOG.md`, `syntaxes/tape.tmLanguage.json`, and `dist/extension.js`, with no shipped `media/preview.css`.

### Batch 5 — Completed (2026-03-20)

- Scope: integration coverage for the frozen extension feature set plus final Phase 3 closeout records.
- Requirements: cross-cutting `T-INT3-*` coverage for activation/LSP, CodeLens/Preview, and no-server fallback.
- Files:
  - `.github/workflows/extension-ci.yml`
  - `editors/code/src/integration.test.ts`
  - `editors/code/package.json`
  - `editors/code/README.md`
  - `editors/code/CHANGELOG.md`
  - `editors/code/icon.png`
  - `spec/phase3/SPEC_TRACEABILITY.md`
  - `trace/phase3/status.yaml`
  - `trace/phase3/tracker.md`
  - `STATUS.yaml`
  - `EXECUTION_TRACKER.md`
- Deliverables:
  - Added `editors/code/src/integration.test.ts` with three process-level integration tests covering `T-INT3-001` through `T-INT3-003`: bundled activation + hover, CodeLens-driven Run & Preview, and no-server mode retaining CodeLens + Preview.
  - Introduced a minimal stdio LSP harness and fake `vhs` command fixture so the tests exercise the real activation flow, bundled binary discovery, `ExecutionManager`, `PreviewManager`, and command wiring without relying on a VS Code host process.
  - Updated `.github/workflows/extension-ci.yml` so extension CI builds a debug `vhs-analyzer` binary before `pnpm run test` and passes it through `VHS_ANALYZER_LSP_BINARY`, satisfying the new Batch 5 integration-test precondition in CI.
  - Replaced the placeholder extension README with marketplace-ready installation and feature documentation, expanded the `0.3.0` changelog entry, and added an extension icon wired through `editors/code/package.json`.
  - Updated Phase 3 traceability and status artifacts, and marked the root indexes as completed while explicitly deferring the optional MAY E2E scenarios `T-INT3-004` and `T-INT3-005`.
- Quality gate:
  - `pnpm run lint`
  - `pnpm run typecheck`
  - `pnpm run test`
  - `pnpm run build`
  - `pnpm exec vsce ls --no-dependencies`
  - `pnpm exec vsce package --no-dependencies`
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - `cargo test --workspace --all-targets --locked`
  - `cargo build --release -p vhs-analyzer-lsp --locked`
- Notes:
  - Batch 5 closes Phase 3 with 62 TypeScript tests and 225 Rust tests green locally.
  - The optional E2E install-path scenarios remain deferred because they are MAY coverage in the frozen matrix and would require a full VS Code host harness beyond the current Vitest-based integration boundary.
