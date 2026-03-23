# SPEC_TRACEABILITY.md — Phase 2 Requirement Traceability

**Phase:** 2 — Intelligence & Diagnostics
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect → Builder (maintained by Builder during implementation)
**Last Updated:** 2026-03-19

---

> **CONTRACT_FROZEN** — This specification is frozen as of 2026-03-19.
> No changes without explicit user approval.

---

## 1. Overview

This document maps every Phase 2 requirement to its planned implementation
module and test references. The Builder MUST update the "Impl Module" and
"Status" columns during implementation.

**Requirement ID Prefixes:**

- `CMP-NNN` — Completion (SPEC_COMPLETION.md)
- `DIA-NNN` — Diagnostics (SPEC_DIAGNOSTICS.md)
- `SAF-NNN` — Safety (SPEC_SAFETY.md)

## 2. WS-1: Completion Traceability

| Req ID | Description | Priority | Planned Impl Module | Test IDs | Phase 1 Baseline | Status |
| --- | --- | --- | --- | --- | --- | --- |
| CMP-001 | completionProvider capability | P0 | `crates/vhs-analyzer-lsp/src/server.rs` | T-CMP-001, T-CMP-002 | LSP-002 (InitializeResult) | Completed |
| CMP-002 | Completion context resolution algorithm | P0 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-010–T-CMP-083, T-INT2-002 | PAR-001, PAR-007, HOV-006 | Completed |
| CMP-003 | Command keyword completions | P0 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-010–T-CMP-015, T-INT2-002 | PAR-001 (SyntaxKind) | Completed |
| CMP-004 | Setting name completions | P0 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-020–T-CMP-022 | PAR-007 (SetCommand) | Completed |
| CMP-005 | Theme name completions | P0 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-030–T-CMP-035 + T-CMP-034A + T-CMP-034B | — | Completed |
| CMP-006 | Setting value completions | P1 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-040–T-CMP-042 + T-CMP-042A | PAR-007 (SetCommand) | Completed |
| CMP-007 | Snippet templates | P1 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-050–T-CMP-052 | — | Completed |
| CMP-008 | Output extension completions | P1 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-060 | PAR-007 (OutputCommand) | Completed |
| CMP-009 | Time unit completions | P2 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-090–T-CMP-092 | — | Completed |
| CMP-010 | Modifier key target completions | P1 | `crates/vhs-analyzer-lsp/src/completion.rs` | T-CMP-070–T-CMP-072 | PAR-007 (CtrlCommand) | Completed |

## 3. WS-2: Diagnostics Traceability

| Req ID | Description | Priority | Planned Impl Module | Test IDs | Phase 1 Baseline | Status |
| --- | --- | --- | --- | --- | --- | --- |
| DIA-001 | Diagnostic source tag | P0 | `crates/vhs-analyzer-lsp/src/diagnostics.rs`; `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-001, T-DIA-002 | LSP-008 (parse diagnostics) | Completed |
| DIA-002 | Severity mapping | P0 | `crates/vhs-analyzer-lsp/src/diagnostics.rs`; `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-001–T-DIA-093 | — | Completed |
| DIA-003 | Missing Output directive | P1 | `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-010–T-DIA-012 | PAR-001 (SOURCE_FILE) | Completed |
| DIA-004 | Invalid Output path extension | P0 | `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-020–T-DIA-025 | PAR-007 (OutputCommand) | Completed |
| DIA-005 | Duplicate Set for same setting | P1 | `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-040–T-DIA-041 | PAR-007 (SetCommand) | Completed |
| DIA-006 | Invalid hex color in MarginFill | P0 | `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-050–T-DIA-055 | PAR-007 (SetCommand) | Completed |
| DIA-007 | Numeric value out of range | P0 | `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-060–T-DIA-065, T-INT2-002 | PAR-007 (SetCommand) | Completed |
| DIA-008 | Require program not found | P1 | `crates/vhs-analyzer-lsp/src/diagnostics/heavyweight.rs`; `crates/vhs-analyzer-lsp/src/server.rs` | T-DIA-070–T-DIA-072 | PAR-007 (RequireCommand) | Completed |
| DIA-009 | Source file not found | P1 | `crates/vhs-analyzer-lsp/src/diagnostics/heavyweight.rs`; `crates/vhs-analyzer-lsp/src/server.rs` | T-DIA-080–T-DIA-081 | PAR-007 (SourceCommand) | Completed |
| DIA-010 | Diagnostic timing classification | P0 | `crates/vhs-analyzer-lsp/src/server.rs`; `crates/vhs-analyzer-lsp/src/diagnostics.rs`; `crates/vhs-analyzer-lsp/src/diagnostics/heavyweight.rs` | T-DIA-072, T-DIA-090–T-DIA-093 | LSP-003 (didChange) | Completed |
| DIA-011 | Unified diagnostic pipeline | P0 | `crates/vhs-analyzer-lsp/src/server.rs`; `crates/vhs-analyzer-lsp/src/diagnostics.rs`; `crates/vhs-analyzer-lsp/src/diagnostics/heavyweight.rs` | T-DIA-090–T-DIA-092, T-INT2-001, T-INT2-002 | LSP-008 (publish_diagnostics) | Completed |
| DIA-012 | Heavyweight check cancellation | P1 | `crates/vhs-analyzer-lsp/src/server.rs`; `crates/vhs-analyzer-lsp/src/diagnostics/heavyweight.rs` | T-DIA-091 | — | Completed |
| DIA-013 | Invalid Screenshot extension | P0 | `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-030–T-DIA-032 | PAR-007 (ScreenshotCommand) | Completed |
| DIA-014 | Unknown built-in theme name | P1 | `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` | T-DIA-066–T-DIA-067 | PAR-007 (SetCommand) | Completed |

## 4. WS-3: Safety Traceability

| Req ID | Description | Priority | Planned Impl Module | Test IDs | Phase 1 Baseline | Status |
| --- | --- | --- | --- | --- | --- | --- |
| SAF-001 | Type directive content extraction | P0 | `crates/vhs-analyzer-lsp/src/safety.rs` | T-SAF-061 | PAR-007 (TypeCommand), LEX-007 | Completed |
| SAF-002 | Dangerous command pattern database | P0 | `crates/vhs-analyzer-lsp/src/safety/patterns.rs` | T-SAF-001–T-SAF-031 | — | Completed |
| SAF-003 | Risk severity levels | P0 | `crates/vhs-analyzer-lsp/src/safety.rs`; `crates/vhs-analyzer-lsp/src/safety/patterns.rs` | T-SAF-001–T-SAF-043 | — | Completed |
| SAF-004 | Detection algorithm | P0 | `crates/vhs-analyzer-lsp/src/safety.rs`; `crates/vhs-analyzer-lsp/src/safety/patterns.rs` | T-SAF-060 | PAR-001 (TYPE_COMMAND) | Completed |
| SAF-005 | Inline suppression mechanism | P1 | `crates/vhs-analyzer-lsp/src/safety.rs` | T-SAF-050–T-SAF-052 | — | Completed |
| SAF-006 | Integration with diagnostic pipeline | P0 | `crates/vhs-analyzer-lsp/src/diagnostics.rs`; `crates/vhs-analyzer-lsp/src/server.rs` | T-INT2-001 | DIA-011 (unified pipeline) | Completed |
| SAF-007 | No false positives on benign commands | P0 | `crates/vhs-analyzer-lsp/src/safety.rs`; `crates/vhs-analyzer-lsp/src/safety/patterns.rs` | T-SAF-040–T-SAF-043 | — | Completed |

## 5. Property-Based Testing Requirements

| Property | Scope | Req Reference | Test ID |
| --- | --- | --- | --- |
| No panics on arbitrary cursor positions | Completion handler | CMP-002 | T-CMP-083 |
| No panics on arbitrary AST inputs | Lightweight diagnostic collectors | DIA-010 | T-DIA-093 |
| No panics on arbitrary string content in Type | Safety detection algorithm | SAF-004 | T-SAF-070 |

## 6. Batch 5 Integration Coverage

| Test ID | Scenario | Coverage | Status |
| --- | --- | --- | --- |
| T-INT2-001 | Combined diagnostics | DIA-011, SAF-006, LSP-008 | Completed |
| T-INT2-002 | Completion + diagnostics coexist | CMP-002, CMP-003, DIA-007, DIA-011 | Completed |
| T-INT2-003 | Server version 0.1.0 | Batch 4 release contract (`serverInfo.version`) | Completed |
| T-INT2-004 | Phase 1 hover + formatting preserved | HOV-001, HOV-002, FMT-001, LSP-005 | Completed |

## 7. Cross-Phase Dependency Summary

| Phase 2 Requirement | Phase 1 Dependency | Extension Type |
| --- | --- | --- |
| CMP-001 (completionProvider) | LSP-002 (InitializeResult) | Capability addition |
| CMP-002 (context resolution) | PAR-001, PAR-007, HOV-006 | Reuse of cursor→AST pattern |
| DIA-010 (didSave handler) | FC-LSP-02 (didSave deferred) | Phase 1 deferred → Phase 2 implements |
| DIA-011 (unified pipeline) | LSP-008 (parse diagnostics) | Extension: parse + semantic + safety |
| SAF-001 (Type extraction) | PAR-007 (TypeCommand), LEX-007 | AST accessor + string token |
