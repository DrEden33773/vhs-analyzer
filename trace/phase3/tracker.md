# Phase 3 Execution Tracker

Phase: VSCode Extension Client (Client, Preview, CodeLens, Packaging)
Status: In Progress (Architect Stage A complete)
Started: 2026-03-19

## 1. Stage Progress

| Stage | Owner | Status | Completed | Deliverables |
| --- | --- | --- | --- | --- |
| Architect Stage A | Claude | Completed | 2026-03-19 | 4 exploratory specs + updated README (43 reqs, 19 FCs) |
| Architect Stage B | Claude | Not Started | — | 6 frozen specs + test matrix + traceability |
| Builder | Builder | Not Started | — | 5 batches (scaffold + client, preview, codelens, packaging, integration) |

## 2. Builder Batch Plan (5 Batches)

| Batch | Name | WS | Location | Requirements | Depends On | Milestone | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | LSP Client + Project Scaffold | WS-1 + WS-4 | editors/vscode | CLI-* | — | Extension activates | not started |
| 2 | Live Preview Webview | WS-2 | editors/vscode | PRV-* | B1 | Preview renders GIF | not started |
| 3 | CodeLens & Commands | WS-3 | editors/vscode | CLS-* | B1 | Run button works | not started |
| 4 | Platform Packaging & CI/CD | WS-4 | editors/vscode + .github | PKG-* | B2, B3 | VSIX builds for all targets | not started |
| 5 | Integration + Closeout | — | editors/vscode | T-INT3 | B4 | Phase 3 complete | not started |

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
