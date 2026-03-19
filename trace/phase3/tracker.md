# Phase 3 Execution Tracker

Phase: VSCode Extension Client (Client, Preview, CodeLens, Packaging)
Status: In Progress (Architect Stage A complete)
Started: 2026-03-19

## 1. Stage Progress

| Stage | Owner | Status | Completed | Deliverables |
| --- | --- | --- | --- | --- |
| Architect Stage A | Claude | Completed | 2026-03-19 | 4 exploratory specs + updated README (43 reqs, 19 FCs) |
| Architect Stage B | Claude | Completed | 2026-03-20 | 6 frozen specs + test matrix + traceability |
| Builder | Builder | In Progress | — | 5 batches (scaffold + client, preview, codelens, packaging, integration) |

## 2. Builder Batch Plan (5 Batches)

| Batch | Name | WS | Location | Requirements | Depends On | Milestone | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | LSP Client + Project Scaffold | WS-1 + WS-4 | editors/code | CLI-* + PKG scaffold | — | Extension activates | completed |
| 2 | Live Preview Webview | WS-2 | editors/code | PRV-* | B1 | Preview renders GIF | not started |
| 3 | CodeLens & Commands | WS-3 | editors/code | CLS-* | B1 | Run button works | not started |
| 4 | Platform Packaging & CI/CD | WS-4 | editors/code + .github | PKG-* | B2, B3 | VSIX builds for all targets | not started |
| 5 | Integration + Closeout | — | editors/code | T-INT3 | B4 | Phase 3 complete | not started |

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
