# SPEC_TRACEABILITY.md — Requirement Traceability Matrix

**Phase:** 1 — LSP Foundation
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect
**Last Updated:** 2026-03-19
**Frozen By:** Architect (Claude) — Stage B

---

> **CONTRACT_FROZEN** — This traceability matrix is frozen as of 2026-03-18.
> All Freeze Candidates have been resolved. No changes without explicit user approval.

---

## 1. Purpose

Map every Phase 1 requirement ID to its specification source, implementation
location (TBD until Builder implements), test reference, and frozen design
decision (if applicable). This matrix ensures that all requirements are tracked
from spec through code to test.

## 2. Traceability Matrix

### 2.1 Lexer (WS-1) — SPEC_LEXER.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference | Design Decision |
| --- | --- | --- | --- | --- | --- |
| LEX-001 | Lossless tokenization (round-trip) | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-001, T-LEX-002, T-LEX-052 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| LEX-002 | Error resilience (no panic, no skip) | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-050 through T-LEX-053 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| LEX-003 | Whitespace preservation (WHITESPACE + NEWLINE) | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-003, T-LEX-004, T-LEX-006 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| LEX-004 | Comment tokens | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-005, T-LEX-006 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| LEX-005 | Keyword recognition (case-sensitive) | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-007, T-LEX-008, T-LEX-028 through T-LEX-034 + wait-scope coverage (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | FC-LEX-01: include ScrollUp/ScrollDown/Screenshot; FC-LEX-04: BOOLEAN kind |
| LEX-006 | Numeric literals (INTEGER, FLOAT) | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-009 through T-LEX-011 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| LEX-007 | String literals (quoted, unterminated) | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-012 through T-LEX-015 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | FC-LEX-03: single STRING token for unterminated |
| LEX-008 | Time literals (ms, s suffix) | P1 SHOULD | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-016 through T-LEX-018 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| LEX-009 | Regex literals | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-019 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| LEX-010 | JSON literals | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-020, T-LEX-021 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| LEX-011 | Path literals | P1 SHOULD | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-022 through T-LEX-024 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | FC-LEX-05: extension allowlist |
| LEX-012 | Punctuation tokens (@, +, %) | P0 MUST | `crates/vhs-analyzer-core/src/lexer.rs` | T-LEX-025 through T-LEX-027 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |

### 2.2 Parser (WS-2) — SPEC_PARSER.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference | Design Decision |
| --- | --- | --- | --- | --- | --- |
| PAR-001 | Unified SyntaxKind enum (token + node kinds) | P0 MUST | `crates/vhs-analyzer-core/src/syntax.rs` | Batch 1 rowan raw-kind round-trip + Phase 2 T-PAR-001 (`crates/vhs-analyzer-core/tests/lexer_tests.rs`) | — |
| PAR-002 | Lossless CST (round-trip via SyntaxNode.text()) | P0 MUST | `crates/vhs-analyzer-core/src/parser.rs` | T-PAR-050, T-PAR-052, T-PAR-057 (`crates/vhs-analyzer-core/tests/parser_tests.rs`) | — |
| PAR-003 | No panics on any input | P0 MUST | `crates/vhs-analyzer-core/src/parser.rs` | T-PAR-051, T-PAR-054 through T-PAR-058 (`crates/vhs-analyzer-core/tests/parser_tests.rs`) | FC-PAR-03: strict one-command-per-line |
| PAR-004 | Error localization (per-command isolation) | P0 MUST | `crates/vhs-analyzer-core/src/parser.rs` | T-PAR-053, T-PAR-055, T-PAR-059 (`crates/vhs-analyzer-core/tests/parser_tests.rs`) | — |
| PAR-005 | Fuel-based infinite loop protection | P0 MUST | `crates/vhs-analyzer-core/src/parser.rs` | T-PAR-056 (`crates/vhs-analyzer-core/tests/parser_tests.rs`) | — |
| PAR-006 | All VHS directives produce dedicated nodes | P0 MUST | `crates/vhs-analyzer-core/src/parser.rs` | T-PAR-001 through T-PAR-031 + T-PAR-004A (`crates/vhs-analyzer-core/tests/parser_tests.rs`) | FC-PAR-04: Copy with optional string |
| PAR-007 | Typed AST accessor layer | P1 SHOULD | `crates/vhs-analyzer-core/src/ast.rs` | T-PAR-070 through T-PAR-073 (`crates/vhs-analyzer-core/tests/ast_tests.rs`) | FC-PAR-02: hand-written for Phase 1 |

### 2.3 LSP Core (WS-3) — SPEC_LSP_CORE.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference | Design Decision |
| --- | --- | --- | --- | --- | --- |
| LSP-001 | Server bootstrap via stdio | P0 MUST | `crates/vhs-analyzer-lsp/src/main.rs` | T-LSP-001 (`crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`), T-INT-001 (`crates/vhs-analyzer-lsp/tests/integration_test.rs`) | — |
| LSP-002 | Initialize handshake with capabilities | P0 MUST | `crates/vhs-analyzer-lsp/src/server.rs` | T-LSP-002, T-LSP-003 (`crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`), T-INT-001 (`crates/vhs-analyzer-lsp/tests/integration_test.rs`) | — |
| LSP-003 | Full document sync (didOpen/didChange/didClose) | P0 MUST | `crates/vhs-analyzer-lsp/src/server.rs` | T-LSP-004 through T-LSP-006 (`crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`), T-INT-001 (`crates/vhs-analyzer-lsp/tests/integration_test.rs`) | — |
| LSP-004 | Concurrent document state store (DashMap) | P0 MUST | `crates/vhs-analyzer-lsp/src/server.rs` | T-LSP-007 (`crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`) | FC-LSP-01: MUST use DashMap |
| LSP-005 | Shutdown and exit protocol | P0 MUST | `crates/vhs-analyzer-lsp/src/server.rs`, `crates/vhs-analyzer-lsp/src/main.rs` | T-LSP-008 (`crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`), T-INT-001 (`crates/vhs-analyzer-lsp/tests/integration_test.rs`) | — |
| LSP-006 | Graceful error handling (no panics, LSP error responses) | P0 MUST | `crates/vhs-analyzer-lsp/src/server.rs` | T-LSP-009 (`crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`), T-INT-001 (`crates/vhs-analyzer-lsp/tests/integration_test.rs`) | — |
| LSP-007 | Logging via tracing to stderr + client | P1 SHOULD | `crates/vhs-analyzer-lsp/src/main.rs`, `crates/vhs-analyzer-lsp/src/server.rs` | T-LSP-013 (`crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`) | — |
| LSP-008 | Parse-error diagnostics via publishDiagnostics | P1 SHOULD | `crates/vhs-analyzer-lsp/src/server.rs` | T-LSP-010 through T-LSP-012 (`crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`), T-INT-001 (`crates/vhs-analyzer-lsp/tests/integration_test.rs`) | FC-LSP-03: SHOULD publish in Phase 1 |

### 2.4 Hover (WS-4) — SPEC_HOVER.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference | Design Decision |
| --- | --- | --- | --- | --- | --- |
| HOV-001 | Hover response format (Markdown, range) | P0 MUST | `crates/vhs-analyzer-lsp/src/hover.rs`, `crates/vhs-analyzer-lsp/src/server.rs` | T-HOV-010 through T-HOV-012 + Markdown markup coverage (`crates/vhs-analyzer-lsp/tests/hover_tests.rs`), T-INT-001 (`crates/vhs-analyzer-lsp/tests/integration_test.rs`) | — |
| HOV-002 | Command keyword hover documentation | P0 MUST | `crates/vhs-analyzer-lsp/src/hover.rs`, `crates/vhs-analyzer-lsp/src/server.rs` | T-HOV-001 through T-HOV-004, T-HOV-016 (`crates/vhs-analyzer-lsp/tests/hover_tests.rs`) | FC-HOV-01: embedded match expression |
| HOV-003 | Setting name hover documentation | P0 MUST | `crates/vhs-analyzer-lsp/src/hover.rs`, `crates/vhs-analyzer-lsp/src/server.rs` | T-HOV-005 through T-HOV-007 (`crates/vhs-analyzer-lsp/tests/hover_tests.rs`) | — |
| HOV-004 | Modifier key hover | P1 SHOULD | `crates/vhs-analyzer-lsp/src/hover.rs`, `crates/vhs-analyzer-lsp/src/server.rs` | T-HOV-008, T-HOV-009 (`crates/vhs-analyzer-lsp/tests/hover_tests.rs`) | — |
| HOV-005 | Literal value hover | P2 MAY | `crates/vhs-analyzer-lsp/src/hover.rs`, `crates/vhs-analyzer-lsp/src/server.rs` | T-HOV-015 (`crates/vhs-analyzer-lsp/tests/hover_tests.rs`) | — |
| HOV-006 | Hover resolution algorithm (token → ancestor → lookup) | P0 MUST | `crates/vhs-analyzer-lsp/src/hover.rs`, `crates/vhs-analyzer-lsp/src/server.rs` | T-HOV-013, T-HOV-014 (`crates/vhs-analyzer-lsp/tests/hover_tests.rs`) | FC-HOV-02: no links; FC-HOV-03: template + unique descriptions |

### 2.5 Formatting (WS-5) — SPEC_FORMATTING.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference | Design Decision |
| --- | --- | --- | --- | --- | --- |
| FMT-001 | LSP formatting response (minimal TextEdits) | P0 MUST | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-001, T-FMT-002, T-FMT-016, T-FMT-017 + formatter idempotence property (`crates/vhs-analyzer-core/tests/formatting_tests.rs`), T-INT-001 (`crates/vhs-analyzer-lsp/tests/integration_test.rs`) | FC-FMT-01: preserve order; FC-FMT-02: no sort |
| FMT-002 | No leading indentation (column 0) | P0 MUST | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-003, T-FMT-004, T-FMT-014 (`crates/vhs-analyzer-core/tests/formatting_tests.rs`) | — |
| FMT-003 | Single space between tokens | P0 MUST | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-005 (`crates/vhs-analyzer-core/tests/formatting_tests.rs`) | — |
| FMT-004 | No space around modifier/duration punctuation | P0 MUST | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-006, T-FMT-007, T-FMT-018 (`crates/vhs-analyzer-core/tests/formatting_tests.rs`) | FC-FMT-03: no space around @ |
| FMT-005 | Blank line normalization | P1 SHOULD | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-008, T-FMT-009 (`crates/vhs-analyzer-core/tests/formatting_tests.rs`) | FC-FMT-04: no auto-insert |
| FMT-006 | Trailing whitespace removal | P0 MUST | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-010 (`crates/vhs-analyzer-core/tests/formatting_tests.rs`) | — |
| FMT-007 | Final newline | P1 SHOULD | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-011, T-FMT-012 (`crates/vhs-analyzer-core/tests/formatting_tests.rs`) | — |
| FMT-008 | Comment preservation | P0 MUST | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-013, T-FMT-014 (`crates/vhs-analyzer-core/tests/formatting_tests.rs`) | — |
| FMT-009 | Error tolerance (skip ERROR nodes) | P0 MUST | `crates/vhs-analyzer-core/src/formatting.rs` | T-FMT-015 (`crates/vhs-analyzer-core/tests/formatting_tests.rs`) | — |

## 3. Requirement Statistics

| Category | P0 (MUST) | P1 (SHOULD) | P2 (MAY) | Total |
| --- | --- | --- | --- | --- |
| Lexer (LEX) | 10 | 2 | 0 | 12 |
| Parser (PAR) | 6 | 1 | 0 | 7 |
| LSP Core (LSP) | 6 | 2 | 0 | 8 |
| Hover (HOV) | 4 | 1 | 1 | 6 |
| Formatting (FMT) | 6 | 3 | 0 | 9 |
| **Total** | **32** | **9** | **1** | **42** |

## 4. Resolved Freeze Candidates Index

All 17 Freeze Candidates have been resolved in Stage B.

### From SPEC_LEXER.md

| ID | Item | Resolution |
| --- | --- | --- |
| FC-LEX-01 | Include ScrollUp/ScrollDown/Screenshot keywords? | **MUST include** — VHS README is behavioral truth |
| FC-LEX-02 | Copy command: with or without string argument? | **Parser concern** — resolved in FC-PAR-04 |
| FC-LEX-03 | Unterminated string: single STRING vs. split ERROR tokens? | **Single STRING** — parser reports error |
| FC-LEX-04 | Boolean: dedicated BOOLEAN kind vs. TRUE_KW/FALSE_KW? | **Dedicated BOOLEAN** — values, not commands |
| FC-LEX-05 | PATH vs. IDENT disambiguation rule? | **Extension allowlist** — gif/mp4/webm/tape/png/txt/ascii/svg/jpg/jpeg |

### From SPEC_PARSER.md

| ID | Item | Resolution |
| --- | --- | --- |
| FC-PAR-01 | Parse errors: side-channel Vec vs. in-tree storage? | **Side-channel Vec** — rust-analyzer pattern |
| FC-PAR-02 | Typed AST layer: hand-written vs. macro-generated? | **Hand-written** — grammar is small |
| FC-PAR-03 | Strict one-command-per-line enforcement? | **Strict** — NEWLINE is command terminator |
| FC-PAR-04 | COPY_COMMAND optional string argument? | **MUST include** — VHS README confirms `Copy "text"` |

### From SPEC_LSP_CORE.md

| ID | Item | Resolution |
| --- | --- | --- |
| FC-LSP-01 | DashMap vs. RwLock for document store? | **DashMap** — per-entry locking essential |
| FC-LSP-02 | Handle didSave in Phase 1? | **MUST NOT** — no-op; defer to Phase 2 |
| FC-LSP-03 | Publish parse-error diagnostics in Phase 1? | **SHOULD publish** — new requirement LSP-008 |
| FC-LSP-04 | MSRV policy? | **Pin MSRV 1.85** — tower-lsp-server requires it |

### From SPEC_HOVER.md

| ID | Item | Resolution |
| --- | --- | --- |
| FC-HOV-01 | Hover docs: embedded in code vs. external file? | **Embedded match** — zero deps, compiler-checked |
| FC-HOV-02 | Include links to VHS website in hover? | **MUST NOT** — cross-editor compatibility |
| FC-HOV-03 | Per-key unique hover content vs. shared template? | **Template + unique descriptions** |

### From SPEC_FORMATTING.md

| ID | Item | Resolution |
| --- | --- | --- |
| FC-FMT-01 | Enforce directive ordering? | **MUST NOT** — preserve user intent |
| FC-FMT-02 | Sort Set commands alphabetically? | **MUST NOT** — preserve user ordering |
| FC-FMT-03 | Space or no space around @ in Type@500ms? | **No space** — matches VHS convention |
| FC-FMT-04 | Auto-insert blank line between settings and commands? | **MUST NOT** — only normalize existing blanks |

## 5. Cross-Reference Index

| Spec File | Requirement IDs | Freeze Candidate IDs | Test IDs |
| --- | --- | --- | --- |
| SPEC_LEXER.md | LEX-001 through LEX-012 | FC-LEX-01 through FC-LEX-05 | T-LEX-001 through T-LEX-053 |
| SPEC_PARSER.md | PAR-001 through PAR-007 | FC-PAR-01 through FC-PAR-04 | T-PAR-001 through T-PAR-073 |
| SPEC_LSP_CORE.md | LSP-001 through LSP-008 | FC-LSP-01 through FC-LSP-04 | T-LSP-001 through T-LSP-013 |
| SPEC_HOVER.md | HOV-001 through HOV-006 | FC-HOV-01 through FC-HOV-03 | T-HOV-001 through T-HOV-016 |
| SPEC_FORMATTING.md | FMT-001 through FMT-009 | FC-FMT-01 through FC-FMT-04 | T-FMT-001 through T-FMT-018 |
| SPEC_TEST_MATRIX.md | — | — | T-INT-001 |
