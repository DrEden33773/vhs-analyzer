# SPEC_TRACEABILITY.md — Requirement Traceability Matrix

**Phase:** 1 — LSP Foundation
**Status:** Stage A (Exploratory Design)
**Owner:** Architect
**Last Updated:** 2026-03-18

---

## 1. Purpose

Map every Phase 1 requirement ID to its specification source, implementation
location (TBD until Builder implements), and test reference. This matrix
ensures that all requirements are tracked from spec through code to test.

## 2. Traceability Matrix

### 2.1 Lexer (WS-1) — SPEC_LEXER.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference |
| --- | --- | --- | --- | --- |
| LEX-001 | Lossless tokenization (round-trip) | P0 MUST | `core::lexer` | TBD — round-trip property test |
| LEX-002 | Error resilience (no panic, no skip) | P0 MUST | `core::lexer` | TBD — fuzz test |
| LEX-003 | Whitespace preservation (WHITESPACE + NEWLINE) | P0 MUST | `core::lexer` | TBD — mixed line endings test |
| LEX-004 | Comment tokens | P0 MUST | `core::lexer` | TBD — comment unit test |
| LEX-005 | Keyword recognition (case-sensitive) | P0 MUST | `core::lexer` | TBD — keyword mapping unit tests |
| LEX-006 | Numeric literals (INTEGER, FLOAT) | P0 MUST | `core::lexer` | TBD — numeric literal tests |
| LEX-007 | String literals (quoted, unterminated) | P0 MUST | `core::lexer` | TBD — string literal tests |
| LEX-008 | Time literals (ms, s suffix) | P1 SHOULD | `core::lexer` | TBD — time literal tests |
| LEX-009 | Regex literals | P0 MUST | `core::lexer` | TBD — regex literal tests |
| LEX-010 | JSON literals | P0 MUST | `core::lexer` | TBD — JSON literal tests |
| LEX-011 | Path literals | P1 SHOULD | `core::lexer` | TBD — path literal tests |
| LEX-012 | Punctuation tokens (@, +, %) | P0 MUST | `core::lexer` | TBD — punctuation unit tests |

### 2.2 Parser (WS-2) — SPEC_PARSER.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference |
| --- | --- | --- | --- | --- |
| PAR-001 | Unified SyntaxKind enum (token + node kinds) | P0 MUST | `core::syntax` | TBD — compilation + rowan Language impl |
| PAR-002 | Lossless CST (round-trip via SyntaxNode.text()) | P0 MUST | `core::parser` | TBD — round-trip test |
| PAR-003 | No panics on any input | P0 MUST | `core::parser` | TBD — fuzz test |
| PAR-004 | Error localization (per-command isolation) | P0 MUST | `core::parser` | TBD — inter-command error test |
| PAR-005 | Fuel-based infinite loop protection | P0 MUST | `core::parser` | TBD — stuck-parser test |
| PAR-006 | All VHS directives produce dedicated nodes | P0 MUST | `core::parser` | TBD — full-coverage integration test |
| PAR-007 | Typed AST accessor layer | P1 SHOULD | `core::ast` | TBD — typed accessor tests |

### 2.3 LSP Core (WS-3) — SPEC_LSP_CORE.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference |
| --- | --- | --- | --- | --- |
| LSP-001 | Server bootstrap via stdio | P0 MUST | `lsp::main` | TBD — startup integration test |
| LSP-002 | Initialize handshake with capabilities | P0 MUST | `lsp::server` | TBD — initialize request/response test |
| LSP-003 | Full document sync (didOpen/didChange/didClose) | P0 MUST | `lsp::server` | TBD — document lifecycle test |
| LSP-004 | Concurrent document state store (DashMap) | P0 MUST | `lsp::server` | TBD — concurrent access test |
| LSP-005 | Shutdown and exit protocol | P0 MUST | `lsp::server` | TBD — shutdown sequence test |
| LSP-006 | Graceful error handling (no panics, LSP error responses) | P0 MUST | `lsp::server` | TBD — forced error test |
| LSP-007 | Logging via tracing to stderr + client | P1 SHOULD | `lsp::main` | TBD — log output verification |

### 2.4 Hover (WS-4) — SPEC_HOVER.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference |
| --- | --- | --- | --- | --- |
| HOV-001 | Hover response format (Markdown, range) | P0 MUST | `lsp::hover` | TBD — response format test |
| HOV-002 | Command keyword hover documentation | P0 MUST | `lsp::hover` | TBD — keyword hover tests (all commands) |
| HOV-003 | Setting name hover documentation | P0 MUST | `lsp::hover` | TBD — setting hover tests (all settings) |
| HOV-004 | Modifier key hover | P1 SHOULD | `lsp::hover` | TBD — Ctrl/Alt/Shift hover tests |
| HOV-005 | Literal value hover | P2 MAY | `lsp::hover` | TBD — optional |
| HOV-006 | Hover resolution algorithm (token → ancestor → lookup) | P0 MUST | `lsp::hover` | TBD — multi-position hover tests |

### 2.5 Formatting (WS-5) — SPEC_FORMATTING.md

| Req ID | Statement Summary | Priority | Impl Module | Test Reference |
| --- | --- | --- | --- | --- |
| FMT-001 | LSP formatting response (minimal TextEdits) | P0 MUST | `core::formatting` | TBD — formatting integration test |
| FMT-002 | No leading indentation (column 0) | P0 MUST | `core::formatting` | TBD — indent removal test |
| FMT-003 | Single space between tokens | P0 MUST | `core::formatting` | TBD — space normalization test |
| FMT-004 | No space around modifier/duration punctuation | P0 MUST | `core::formatting` | TBD — punctuation spacing test |
| FMT-005 | Blank line normalization | P1 SHOULD | `core::formatting` | TBD — blank line collapse test |
| FMT-006 | Trailing whitespace removal | P0 MUST | `core::formatting` | TBD — trailing whitespace test |
| FMT-007 | Final newline | P1 SHOULD | `core::formatting` | TBD — final newline test |
| FMT-008 | Comment preservation | P0 MUST | `core::formatting` | TBD — comment preservation test |
| FMT-009 | Error tolerance (skip ERROR nodes) | P0 MUST | `core::formatting` | TBD — error-tolerant formatting test |

## 3. Requirement Statistics

| Category | P0 (MUST) | P1 (SHOULD) | P2 (MAY) | Total |
| --- | --- | --- | --- | --- |
| Lexer (LEX) | 10 | 2 | 0 | 12 |
| Parser (PAR) | 6 | 1 | 0 | 7 |
| LSP Core (LSP) | 6 | 1 | 0 | 7 |
| Hover (HOV) | 4 | 1 | 1 | 6 |
| Formatting (FMT) | 6 | 3 | 0 | 9 |
| **Total** | **32** | **8** | **1** | **41** |

## 4. Freeze Candidates Summary

All Freeze Candidates across Phase 1 specs, collected for Stage B resolution:

### From SPEC_LEXER.md

| ID | Item |
| --- | --- |
| FC-LEX-01 | Include ScrollUp/ScrollDown/Screenshot keywords? |
| FC-LEX-02 | Copy command: with or without string argument? |
| FC-LEX-03 | Unterminated string: single STRING vs. split ERROR tokens? |
| FC-LEX-04 | Boolean: dedicated BOOLEAN kind vs. TRUE_KW/FALSE_KW? |
| FC-LEX-05 | PATH vs. IDENT disambiguation rule? |

### From SPEC_PARSER.md

| ID | Item |
| --- | --- |
| FC-PAR-01 | Parse errors: side-channel Vec vs. in-tree storage? |
| FC-PAR-02 | Typed AST layer: hand-written vs. macro-generated? |
| FC-PAR-03 | Strict one-command-per-line enforcement? |
| FC-PAR-04 | COPY_COMMAND optional string argument? (linked to FC-LEX-02) |

### From SPEC_LSP_CORE.md

| ID | Item |
| --- | --- |
| FC-LSP-01 | `DashMap` vs. `RwLock<HashMap>` for document store? |
| FC-LSP-02 | Handle didSave in Phase 1? |
| FC-LSP-03 | Publish parse-error diagnostics in Phase 1? |
| FC-LSP-04 | MSRV policy (pin 1.85 vs. track stable)? |

### From SPEC_HOVER.md

| ID | Item |
| --- | --- |
| FC-HOV-01 | Hover docs: embedded in code vs. external file? |
| FC-HOV-02 | Include links to VHS website in hover? |
| FC-HOV-03 | Per-key unique hover content vs. shared template? |

### From SPEC_FORMATTING.md

| ID | Item |
| --- | --- |
| FC-FMT-01 | Enforce directive ordering? |
| FC-FMT-02 | Sort Set commands alphabetically? |
| FC-FMT-03 | Space or no space around @ in Type@500ms? |
| FC-FMT-04 | Auto-insert blank line between settings and commands? |

**Total Freeze Candidates: 17** <!-- markdownlint-disable-line -->

## 5. Cross-Reference Index

| Spec File | Requirement IDs | Freeze Candidate IDs |
| --- | --- | --- |
| SPEC_LEXER.md | LEX-001 through LEX-012 | FC-LEX-01 through FC-LEX-05 |
| SPEC_PARSER.md | PAR-001 through PAR-007 | FC-PAR-01 through FC-PAR-04 |
| SPEC_LSP_CORE.md | LSP-001 through LSP-007 | FC-LSP-01 through FC-LSP-04 |
| SPEC_HOVER.md | HOV-001 through HOV-006 | FC-HOV-01 through FC-HOV-03 |
| SPEC_FORMATTING.md | FMT-001 through FMT-009 | FC-FMT-01 through FC-FMT-04 |
