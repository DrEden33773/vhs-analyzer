# Phase 2 Execution Tracker

Phase: Intelligence & Diagnostics (Completion, Diagnostics, Safety)
Status: In Progress — Builder implementation pending
Started: 2026-03-19

## 1. Stage Progress

| Stage | Owner | Status | Completed | Deliverables |
| --- | --- | --- | --- | --- |
| Architect Stage A | Claude | Completed | 2026-03-19 | 3 exploratory specs, 30 reqs, 11 FC identified |
| Architect Stage B | Claude | Completed | 2026-03-19 | 5 frozen specs, 30 reqs, 67 test scenarios |
| Builder | Builder | Not started | — | 5 batches planned |

## 2. Builder Batch Plan (5 Batches)

| Batch | Name | WS | Crate | Requirements | Depends On | Milestone | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Lightweight Diagnostic Rules | WS-2 | lsp | DIA-001~007, DIA-013 | — | — | not started |
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
