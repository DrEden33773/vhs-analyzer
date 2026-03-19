# Phase 2 Execution Tracker

Phase: Intelligence & Diagnostics (Completion, Diagnostics, Safety)
Status: In Progress — Batch 1 completed, Batch 2 pending
Started: 2026-03-19

## 1. Stage Progress

| Stage | Owner | Status | Completed | Deliverables |
| --- | --- | --- | --- | --- |
| Architect Stage A | Claude | Completed | 2026-03-19 | 3 exploratory specs, 30 reqs, 11 FC identified |
| Architect Stage B | Claude | Completed | 2026-03-19 | 5 frozen specs, 30 reqs, 67 test scenarios |
| Builder | Builder | In Progress | — | 5 batches planned |

## 2. Builder Batch Plan (5 Batches)

| Batch | Name | WS | Crate | Requirements | Depends On | Milestone | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Lightweight Diagnostic Rules | WS-2 | lsp | DIA-001~007, DIA-013 | — | — | completed |
| 2 | Safety Engine | WS-3 | lsp | SAF-001~007 | B1 | — | not started |
| 3 | Heavyweight Diagnostics + Pipeline | WS-2 | lsp | DIA-008~012 | B2 | Pipeline complete | not started |
| 4 | Completion Provider | WS-1 | lsp | CMP-001~010 | B3 | All features complete | not started |
| 5 | Integration + Closeout | — | both | T-INT2 + property | B4 | Phase 2 complete | not started |

## 3. Dependency Constraints

- B1 → B2 → B3 → B4 → B5 (recommended sequential order).
- B1-B2 are purely synchronous (no async). B3 introduces async.
- B4 is independent of B1-B3 (could theoretically run in parallel).
- B5 MUST be last (integration test + closeout).

## 4. Stage B Design Decisions Log

Key architectural decisions resolved during Phase 2 Stage B (2026-03-19):

| FC ID | Decision | Rationale | Spec Reference |
| --- | --- | --- | --- |
| FC-CMP-01 | Empty trigger characters `[]` | Client word-boundary triggers sufficient | SPEC_COMPLETION.md §11 |
| FC-CMP-02 | 4 WindowBar styles | VHS README confirms complete list | SPEC_COMPLETION.md §11 |
| FC-CMP-03 | External themes.txt + include_str! + LazyLock | 318 entries too large for inline; near-zero overhead | SPEC_COMPLETION.md §11 |
| FC-DIA-01 | No Output directory check | VHS v0.3.0+ auto-creates directories | SPEC_DIAGNOSTICS.md §11 |
| FC-DIA-02 | No LoopOffset range validation | VHS docs unspecified | SPEC_DIAGNOSTICS.md §11 |
| FC-DIA-03 | Validate Screenshot .png only (DIA-013) | VHS README confirms png-only | SPEC_DIAGNOSTICS.md §11 |
| FC-DIA-04 | Cancellation-and-restart only | PATH lookups ~5ms | SPEC_DIAGNOSTICS.md §11 |
| FC-SAF-01 | No cross-Type sequence detection | High false-positive rate | SPEC_SAFETY.md §12 |
| FC-SAF-02 | Use regex crate (standard) | Needs RegexSet | SPEC_SAFETY.md §12 |
| FC-SAF-03 | No workspace config in Phase 2 | Defer to Phase 3 | SPEC_SAFETY.md §12 |
| FC-SAF-04 | No Env directive scanning | Niche vector; high FP risk | SPEC_SAFETY.md §12 |

## 5. Builder Batch Records

Builder appends one record per completed batch below this line.

### Batch 1 — Lightweight Diagnostic Rules

- Date: 2026-03-19
- Status: Completed
- Requirements: `DIA-001`, `DIA-002`, `DIA-003`, `DIA-004`, `DIA-005`, `DIA-006`, `DIA-007`, `DIA-013`
- Files:
  - `crates/vhs-analyzer-lsp/src/server.rs`
  - `crates/vhs-analyzer-lsp/src/diagnostics.rs`
  - `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs`
  - `crates/vhs-analyzer-lsp/tests/diagnostics_tests.rs`
  - `crates/vhs-analyzer-lsp/tests/integration_test.rs`
  - `crates/vhs-analyzer-lsp/tests/lsp_integration_tests.rs`
  - `spec/phase2/SPEC_TRACEABILITY.md`
  - `trace/phase2/status.yaml`
  - `trace/phase2/tracker.md`
- Deliverables:
  - Added a dedicated lightweight diagnostic layer that publishes parse diagnostics and semantic diagnostics together on `didOpen` and `didChange`.
  - Implemented missing `Output`, invalid `Output` extension, invalid `Screenshot` extension, duplicate `Set`, invalid `MarginFill` hex, and numeric range validation rules.
  - Added 28 passing acceptance tests in `crates/vhs-analyzer-lsp/tests/diagnostics_tests.rs` covering the Batch 1 `T-DIA-001` through `T-DIA-065` scope.
  - Updated existing LSP integration tests so Phase 2's lightweight diagnostics remain visible after syntax errors are fixed but semantic issues still exist.
- Quality gate:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  - `cargo test --workspace --all-targets --locked`
- Notes:
  - Added a compatibility shim in `crates/vhs-analyzer-lsp/src/diagnostics/semantic.rs` that suppresses specific Phase 1 parse diagnostics for recoverable `Output` and `Screenshot` path variants plus signed numeric setting values, allowing the frozen Phase 2 semantic contracts to hold without modifying the frozen Phase 1 lexer/parser behavior.
  - `DIA-010` and `DIA-011` are now partially wired for the `didChange` lightweight path and remain in progress until Batch 3 completes the heavyweight and save-time pipeline.
