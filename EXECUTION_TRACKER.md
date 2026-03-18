# EXECUTION_TRACKER.md

Execution tracker linked to `ROADMAP.md`, `README.md`, `STATUS.yaml`, and `AGENTS.md`.
This file quantifies implementation progress and enforces model responsibility boundaries.

## 1. Snapshot

- Date: 2026-03-18
- Workspace: `vhs-analyzer`
- Current Phase: **Pre-Phase 1 completed** → ready for Phase 1 Architect Stage A

## 2. Responsibility Boundaries

- Builder MUST focus on executable implementation, tests, refactors, and non-strategic docs sync.
- Architect (Claude) SHOULD own AST design, LSP protocol subset, validation matrix, extension API design, and Webview messaging protocol.
- Scout (Gemini) SHOULD own market research, competitive analysis, and roadmap drafting.
- Builder MUST NOT finalize Architect-owned design outputs without explicit user instruction.
- Cross-model edits SHOULD happen by contract-first handoff through `spec/` and this tracker.

## 3. Roadmap Progress (Quantified)

Legend: `[x]` completed, `[~]` in progress, `[ ]` not started

### 3.1 Pre-Phase 1: Project Initialization

- [x] Initialize monorepo (`crates/vhs-analyzer-core`, `crates/vhs-analyzer-lsp`, `editors/vscode`)
- [x] Workflow setup (port `prompt/`, `spec/`, `trace/`, and coordination files from `eden-skills`)

### 3.2 Phase 1: LSP Foundation

- [ ] **(Architect Stage A)** Draft exploratory specs for Lexer, Parser, LSP Core, Hover, Formatting
- [ ] **(Architect Stage B)** Freeze all specs with MUST/SHOULD/MAY contracts and test matrix
- [ ] **(Builder)** Handcraft Lexer and map VHS tokens
- [ ] **(Builder)** Implement Recursive Descent Parser (rowan-based)
- [ ] **(Builder)** Wire up `tower-lsp-server` and implement `initialize` / `textDocument/didChange`
- [ ] **(Builder)** Implement `textDocument/hover` provider

### 3.3 Phase 2: Intelligence & Diagnostics

- [ ] Implement semantic validation (syntax errors, invalid paths)
- [ ] Implement Safety Check Engine (warn on destructive commands)
- [ ] Implement `textDocument/completion` provider

### 3.4 Phase 3: VSCode Extension Client

- [ ] Develop TypeScript client using `vscode-languageclient`
- [ ] Build Live Preview Webview
- [ ] Implement runtime dependency detection (warn if `vhs`, `ttyd`, `ffmpeg` are missing)
- [ ] Setup multi-target CI/CD via `vsce`

Progress score (all phases): `2 / 15 = 13%`

### 3.5 Verification and Testing

- [ ] CI gate setup for Linux + macOS + Windows
- [ ] Phase 1 test matrix verified

Current automated tests: `0`

## 4. Pending Tasks with Planned LLM Ownership

### 4.1 Architect-Owned (Claude)

- [ ] Draft Phase 1 Stage A specs (Lexer token set, AST node definitions, LSP protocol subset)
- [ ] Freeze Phase 1 Stage B specs (close all Freeze Candidates, produce MUST/SHOULD/MAY contracts)
- [ ] Design Phase 2 validation matrix and autocomplete heuristics
- [ ] Define Phase 3 Extension API, Webview messaging protocol, and CI packaging spec

### 4.2 Builder-Owned

- [ ] Implement Phase 1 from frozen specs (Lexer, Parser, tower-lsp-server, Hover)
- [ ] Implement Phase 2 from frozen specs (Completion, Diagnostics, Safety)
- [ ] Implement Phase 3 from frozen specs (VSCode client, Preview, CodeLens, CI/CD)

### 4.3 Shared with Boundary Control

- Any change that mutates LSP behavior MUST be spec-first (`spec/` update before code).
- Any Architect decision consumed by Builder MUST be recorded as explicit contract items before implementation.

## 5. Phase Records

All phase execution records (both active and frozen) will live in `trace/<phase>/`.
Each directory will contain `status.yaml` (machine-readable) and `tracker.md`
(human-readable batch progress).

| Phase | Status | Started | Completed | Archive |
| ----- | ------ | ------- | --------- | ------- |
| Pre-Phase 1 | Completed | 2026-03-18 | 2026-03-18 | — |
| Phase 1 | Not Started | — | — | — |
| Phase 2 | Not Started | — | — | — |
| Phase 3 | Not Started | — | — | — |
