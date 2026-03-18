# Phase 1 Execution Tracker

Phase: LSP Foundation (Lexer, Parser, tower-lsp-server, Hover, Formatting)
Status: In Progress — Batch 3 complete, core crate complete, Builder in progress
Started: 2026-03-18
Completed: —

## 1. Stage Progress

| Stage | Owner | Status | Completed | Deliverables |
| --- | --- | --- | --- | --- |
| Architect Stage A | Claude | Completed | 2026-03-18 | 6 exploratory specs, 41 reqs, 17 FC identified |
| Architect Stage B | Claude | Completed | 2026-03-18 | 7 frozen specs, 42 reqs, 105 test scenarios |
| Builder | Builder | In Progress | — | Batch 3 complete — Formatting done, `vhs-analyzer-core` complete; LSP Core and Hover remain |

## 2. Builder Batch Plan (Crate-Aligned, 6 Batches)

| Batch | Name | WP | Crate | Requirements | Depends On | Milestone | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | SyntaxKind + Lexer | WS-1 | core | PAR-001, LEX-001~012 | — | — | completed |
| 2 | Parser + Typed AST | WS-2 | core | PAR-002~007 | B1 | — | completed |
| 3 | Formatting | WS-5 | core | FMT-001~009 | B2 | core crate complete | completed |
| 4 | LSP Core + Diagnostics | WS-3 | lsp | LSP-001~008 | B3 | — | not started |
| 5 | Hover | WS-4 | lsp | HOV-001~006 | B4 | lsp crate complete | not started |
| 6 | Integration + Closeout | — | both | T-INT-001 | B5 | Phase 1 complete | not started |

## 3. Dependency Constraints

- B1 → B2 → B3 → B4 → B5 → B6 (strictly sequential).
- B1-B3 complete `vhs-analyzer-core` crate. B4-B5 complete `vhs-analyzer-lsp` crate.
- B6 MUST be last (integration test + closeout).

## 4. Stage A Design Decisions Log

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

## 5. Stage B Freeze Decisions Log

All 17 Freeze Candidates resolved during Phase 1 Stage B (2026-03-18):

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

## 6. Requirement Statistics (Post-Freeze)

| Category | P0 (MUST) | P1 (SHOULD) | P2 (MAY) | Total |
| --- | --- | --- | --- | --- |
| Lexer (LEX) | 10 | 2 | 0 | 12 |
| Parser (PAR) | 6 | 1 | 0 | 7 |
| LSP Core (LSP) | 6 | 2 | 0 | 8 |
| Hover (HOV) | 4 | 1 | 1 | 6 |
| Formatting (FMT) | 6 | 3 | 0 | 9 |
| **Total** | **32** | **9** | **1** | **42** |

## 7. Test Matrix Summary

| Category | P0 (MUST) | P1 (SHOULD) | P2 (MAY) | Total |
| --- | --- | --- | --- | --- |
| Lexer (T-LEX) | 30 | 6 | 0 | 36 |
| Parser (T-PAR) | 21 | 4 | 0 | 25 |
| LSP Core (T-LSP) | 9 | 4 | 0 | 13 |
| Hover (T-HOV) | 11 | 2 | 1 | 14 |
| Formatting (T-FMT) | 13 | 4 | 0 | 17 |
| **Total** | **84** | **20** | **1** | **105** |

## 8. Completion Records

### Batch 1 — SyntaxKind + Lexer

- Date: 2026-03-19
- Status: Completed
- Requirements: `PAR-001`, `LEX-001` through `LEX-012`
- Files:
  - `crates/vhs-analyzer-core/src/lib.rs`
  - `crates/vhs-analyzer-core/src/syntax.rs`
  - `crates/vhs-analyzer-core/src/lexer.rs`
  - `crates/vhs-analyzer-core/tests/lexer_tests.rs`
  - `crates/vhs-analyzer-core/Cargo.toml`
- Deliverables:
  - Added the unified `SyntaxKind` enum with `#[repr(u16)]` and the `rowan::Language` bridge.
  - Implemented a hand-written lexer covering trivia, comments, keywords, literals, punctuation, PATH allowlist rules, and error tokens.
  - Added 37 passing integration and property tests for lexer behavior and the rowan raw-kind round-trip.
- Quality gate:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- Notes:
  - Added `proptest` to `crates/vhs-analyzer-core/Cargo.toml` for lossless and no-panic property coverage.
  - Added an explicit wait-scope keyword test for `Screen` and `Line` because the frozen lexer matrix does not enumerate them individually even though the frozen lexer spec requires dedicated token kinds.

### Batch 2 — Parser + Typed AST

- Date: 2026-03-19
- Status: Completed
- Requirements: `PAR-002` through `PAR-007`
- Files:
  - `crates/vhs-analyzer-core/src/lib.rs`
  - `crates/vhs-analyzer-core/src/parser.rs`
  - `crates/vhs-analyzer-core/src/ast.rs`
  - `crates/vhs-analyzer-core/tests/parser_tests.rs`
  - `crates/vhs-analyzer-core/tests/ast_tests.rs`
- Deliverables:
  - Implemented a direct `rowan::GreenNodeBuilder` parser with dedicated command nodes, newline-delimited recovery, strict one-command-per-line handling, and side-channel `ParseError` collection.
  - Added hand-written typed AST wrappers and accessors for Phase 1 parser nodes, including `TypeCommand`, `SetCommand`, `KeyCommand`, `Duration`, and related command wrappers.
  - Added 41 passing parser tests plus 4 typed AST tests covering directive parsing, error localization, lossless round-trips, no-panic property behavior, and typed accessors.
- Quality gate:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- Notes:
  - Parser coverage intentionally follows the frozen explicit `SyntaxKind` enumeration and does not attempt to resolve unrelated spec count mismatches.
  - The parser accepts whitespace-tolerant `@` and `+` forms so later formatting work can normalize lines without losing CST structure.

### Batch 3 — Formatting

- Date: 2026-03-19
- Status: Completed
- Requirements: `FMT-001` through `FMT-009`
- Files:
  - `crates/vhs-analyzer-core/src/lib.rs`
  - `crates/vhs-analyzer-core/src/formatting.rs`
  - `crates/vhs-analyzer-core/tests/formatting_tests.rs`
- Deliverables:
  - Implemented a rowan-only formatting module that emits byte-range `TextEdit` operations from the parsed syntax tree.
  - Normalized leading indentation, inter-token spacing, blank lines, trailing whitespace, final newlines, and line-start comment indentation while preserving directive order.
  - Preserved malformed lines containing `ERROR` nodes verbatim so formatting remains lossless and error-tolerant.
  - Added 18 passing formatting tests covering `T-FMT-001` through `T-FMT-018`, including idempotence, comment preservation, and mixed valid/error files.
- Quality gate:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
- Notes:
  - Batch 3 completes the `vhs-analyzer-core` crate milestone.
  - The frozen summary tables still report 17 formatting scenarios, but implementation follows the explicit `T-FMT-001` through `T-FMT-018` enumeration without resolving the spec mismatch.
