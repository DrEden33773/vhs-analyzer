# EXECUTION_TRACKER.md

Execution tracker linked to `ROADMAP.md`, `README.md`, `STATUS.yaml`, and `AGENTS.md`.
This file quantifies implementation progress and enforces model responsibility boundaries.

## 1. Snapshot

- Date: 2026-03-18
- Workspace: `vhs-analyzer`
- Current Phase: **Phase 1 — Architect Stage B completed (specs frozen)** → ready for Builder

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

- [x] **(Architect Stage A)** Draft exploratory specs for Lexer, Parser, LSP Core, Hover, Formatting
  - Deliverables: SPEC_LEXER.md, SPEC_PARSER.md, SPEC_LSP_CORE.md, SPEC_HOVER.md, SPEC_FORMATTING.md, SPEC_TRACEABILITY.md
  - 41 requirements defined (32 P0 MUST, 8 P1 SHOULD, 1 P2 MAY)
  - 17 Freeze Candidates identified for Stage B resolution
- [x] **(Architect Stage B)** Freeze all specs with MUST/SHOULD/MAY contracts and test matrix
  - All 17 Freeze Candidates resolved with definitive decisions
  - 42 requirements frozen (32 P0, 9 P1, 1 P2) — added LSP-008 (parse-error diagnostics)
  - SPEC_TEST_MATRIX.md created with 105 test scenarios (84 P0, 20 P1, 1 P2)
  - All 7 spec files carry CONTRACT_FROZEN marker
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

Progress score (all phases): `4 / 15 = 27%`

### 3.5 Verification and Testing

- [ ] CI gate setup for Linux + macOS + Windows
- [ ] Phase 1 test matrix verified

Current automated tests: `0`

## 4. Pending Tasks with Planned LLM Ownership

### 4.1 Architect-Owned (Claude)

- [x] Draft Phase 1 Stage A specs (Lexer token set, AST node definitions, LSP protocol subset)
- [x] Freeze Phase 1 Stage B specs (close all 17 Freeze Candidates, produce MUST/SHOULD/MAY contracts, finalize SPEC_TEST_MATRIX.md)
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
| Phase 1 | In Progress (Stage B done, Builder next) | 2026-03-18 | — | — |
| Phase 2 | Not Started | — | — | — |
| Phase 3 | Not Started | — | — | — |

## 6. Stage A Design Decisions Log

Key architectural decisions made during Phase 1 Stage A (2026-03-18):

| Decision | Choice | Rationale | Spec Reference |
| --- | --- | --- | --- |
| Lexer strategy | Hand-written (no logos) | Max control over error recovery; small token set | SPEC_LEXER.md §5.1 |
| Keyword handling | Always-keyword (stateless lexer) | Simple; parser handles value-position keywords | SPEC_LEXER.md §5.2 |
| Time literal | Single TIME token | Trivial lookahead; cleaner for hover | SPEC_LEXER.md §5.3 |
| Parser builder | Direct GreenNodeBuilder (not event list) | VHS has no left-assoc binary ops; simpler one-pass | SPEC_PARSER.md §6.1 |
| Error recovery | Newline-delimited | VHS is line-oriented; natural boundary | SPEC_PARSER.md §6.2 |
| Key command nodes | Unified KEY_COMMAND | All share same shape; fewer node kinds | SPEC_PARSER.md §6.3 |
| Modifier decomposition | Multi-token | Enables per-modifier hover/completion | SPEC_PARSER.md §6.4 |
| Document store | DashMap | Per-entry locking; non-blocking concurrent access | SPEC_LSP_CORE.md §4.1 |
| Parse-on-change | Synchronous in did_change | VHS files small; <1ms parse time expected | SPEC_LSP_CORE.md §4.2 |
| Document sync | Full sync (TextDocumentSyncKind::Full) | Simple; no incremental patching bugs | SPEC_LSP_CORE.md §4.4 |
| Hover docs storage | match expression | Small entry count; zero deps | SPEC_HOVER.md §4.1 |
| Hover context | Token + parent node | Context-sensitive docs with minimal complexity | SPEC_HOVER.md §4.2 |
| Formatter strategy | Token-stream transform | Minimal edits; preserves structure; error-tolerant | SPEC_FORMATTING.md §4.1 |

## 7. Stage B Freeze Decisions Log

Key architectural decisions resolved during Phase 1 Stage B (2026-03-18):

| FC ID | Decision | Rationale | Spec Reference |
| --- | --- | --- | --- |
| FC-LEX-01 | Include ScrollUp/ScrollDown/Screenshot | VHS README is behavioral truth; grammar.js lags | SPEC_LEXER.md §7 |
| FC-LEX-02 | Copy argument is parser concern | Lexer is stateless; Copy always lexes as COPY_KW | SPEC_LEXER.md §7 |
| FC-LEX-03 | Single STRING for unterminated strings | Maintains lossless property; parser reports error | SPEC_LEXER.md §7 |
| FC-LEX-04 | Dedicated BOOLEAN token kind | Booleans are values, not commands; cleaner semantics | SPEC_LEXER.md §7 |
| FC-LEX-05 | Extension allowlist for PATH | Finite VHS format set; avoids false positives | SPEC_LEXER.md §7 |
| FC-PAR-01 | Side-channel Vec for parse errors | rust-analyzer pattern; GreenNode stays immutable/internable | SPEC_PARSER.md §9 |
| FC-PAR-02 | Hand-written typed AST layer | Grammar is small (~20 types); codegen is overkill | SPEC_PARSER.md §9 |
| FC-PAR-03 | Strict one-command-per-line | VHS has no line continuation; NEWLINE = terminator | SPEC_PARSER.md §9 |
| FC-PAR-04 | Copy with optional string argument | VHS README documents `Copy "text"` | SPEC_PARSER.md §9 |
| FC-LSP-01 | DashMap for document store | Per-entry locking; non-blocking concurrent access | SPEC_LSP_CORE.md §8 |
| FC-LSP-02 | No didSave handler in Phase 1 | No work to do on save; defer to Phase 2 | SPEC_LSP_CORE.md §8 |
| FC-LSP-03 | SHOULD publish parse-error diagnostics | Zero-cost output of parser; immediate user value | SPEC_LSP_CORE.md §8 |
| FC-LSP-04 | Pin MSRV 1.85 | tower-lsp-server v0.23 requires it | SPEC_LSP_CORE.md §8 |
| FC-HOV-01 | Embedded match expression for hover docs | Zero-dep; compiler-checked; ~46 entries | SPEC_HOVER.md §7 |
| FC-HOV-02 | No links in Phase 1 hover | Cross-editor rendering inconsistency | SPEC_HOVER.md §7 |
| FC-HOV-03 | Template + unique descriptions for keys | 13 keys share syntax; template avoids duplication | SPEC_HOVER.md §7 |
| FC-FMT-01 | Preserve directive ordering | Formatter scope is whitespace, not structure | SPEC_FORMATTING.md §6 |
| FC-FMT-02 | No sorting of Set commands | Preserve user intent and grouping | SPEC_FORMATTING.md §6 |
| FC-FMT-03 | No space around @ in durations | Matches VHS documentation convention | SPEC_FORMATTING.md §6 |
| FC-FMT-04 | No auto-insert blank lines | Only normalize existing; don't add structure | SPEC_FORMATTING.md §6 |
