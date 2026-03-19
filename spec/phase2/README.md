# Phase 2: Intelligence & Diagnostics

## Status: Stage B (CONTRACT_FROZEN — 2026-03-19)

Phase 2 adds smart features on top of the LSP Foundation: context-aware
autocomplete, semantic diagnostics, and a safety check engine for `Type`
directives. All three work streams depend on Phase 1 frozen contracts but
are independent of each other.

## Spec Files

| File | Work Stream | Status | Description |
| --- | --- | --- | --- |
| `SPEC_COMPLETION.md` | WS-1 | Frozen | Context-aware autocomplete for commands, settings, themes, values |
| `SPEC_DIAGNOSTICS.md` | WS-2 | Frozen | Semantic validation, environment checks, unified diagnostic pipeline |
| `SPEC_SAFETY.md` | WS-3 | Frozen | Destructive command detection in Type directives |
| `SPEC_TEST_MATRIX.md` | — | Frozen | Phase 2 acceptance test matrix |
| `SPEC_TRACEABILITY.md` | — | Frozen | Requirement-to-implementation mapping (maintained by Builder) |

## Work Streams

```text
WS-1: Completion    (SPEC_COMPLETION.md)   — autocomplete for commands, Set settings,
                                             theme names, setting values, snippets
WS-2: Diagnostics   (SPEC_DIAGNOSTICS.md)  — semantic validation, environment checks,
                                             unified pipeline with Phase 1 parse errors
WS-3: Safety        (SPEC_SAFETY.md)       — Type directive content scanning against
                                             dangerous command pattern database
```

## Dependency Graph

```text
Phase 1 (frozen)
  │
  ├── SPEC_PARSER.md ──┬──> WS-1 (Completion)   — AST for context resolution
  │                     ├──> WS-2 (Diagnostics)  — AST for semantic analysis
  │                     └──> WS-3 (Safety)       — TYPE_COMMAND string extraction
  │
  └── SPEC_LSP_CORE.md ──> WS-2 (Diagnostics)   — server lifecycle, didSave handler,
                                                    publish_diagnostics, DocumentState

Phase 2 internal:
  WS-3 (Safety) ──> WS-2 (Diagnostics)  — safety diagnostics merge into unified pipeline
  WS-1 (Completion) is fully independent of WS-2 and WS-3
```

## Builder Batch Plan

The three work streams are independent and MAY be implemented in parallel
or sequentially. The recommended ordering accounts for the WS-3 → WS-2
soft dependency (safety diagnostics integrate into the diagnostic pipeline).

```text
Batch 1: WS-2 — Diagnostic pipeline infrastructure
         (lightweight semantic checks + heavyweight framework + didSave handler)
         Establishes the unified pipeline that WS-3 plugs into.
         Deliverables: DIA-001 through DIA-013, DocumentState extension,
         didSave handler, cancellation tokens.

Batch 2: WS-3 — Safety check engine
         (pattern database + detection algorithm + suppression + pipeline integration)
         Leverages the diagnostic pipeline from Batch 1.
         Deliverables: SAF-001 through SAF-007, regex pattern database,
         LazyLock<RegexSet>, suppression comment scanning.

Batch 3: WS-1 — Completion provider
         (context resolution + item registries + snippets + theme data file)
         Fully independent; can be parallelized with Batch 1-2 if desired.
         Deliverables: CMP-001 through CMP-010, data/themes.txt,
         include_str! + LazyLock theme registry.

Batch 4: Integration test + closeout
         (cross-WS integration tests, property-based tests, Phase 1 regression)
         Deliverables: T-INT2-001 through T-INT2-004, all property-based tests.
```

**Parallel execution note:** Batch 3 (Completion) has zero dependency on
Batch 1 or 2 and MAY run in parallel from the start. Batch 2 (Safety) has
a soft dependency on Batch 1 (Diagnostics) via the unified pipeline
integration point (DIA-011 / SAF-006). If running in parallel, the Builder
should implement the safety detection logic first and defer pipeline
integration to a sync point after Batch 1 completes.

## Cross-Phase Capability Extension

Phase 2 extends the Phase 1 `InitializeResult` capabilities. The
complete Phase 2 capability set is defined in SPEC_COMPLETION.md §10:

| Capability | Phase 1 | Phase 2 |
| --- | --- | --- |
| `textDocumentSync.change = 1` (Full) | Yes | Yes |
| `textDocumentSync.save` | No | Yes (`{ includeText: false }`) |
| `hoverProvider` | Yes | Yes |
| `documentFormattingProvider` | Yes | Yes |
| `completionProvider` | No | Yes (`triggerCharacters: [], resolveProvider: false`) |

## Key Resolved Decisions (Stage B Summary)

| FC ID | Decision | Rationale |
| --- | --- | --- |
| FC-CMP-01 | Empty trigger characters `[]` | Client word-boundary triggers sufficient; space trigger too noisy |
| FC-CMP-02 | 4 WindowBar styles | VHS README confirms: Colorful, ColorfulRight, Rings, RingsRight |
| FC-CMP-03 | External `data/themes.txt` + `include_str!` + `LazyLock` | 318 entries too large for inline array; near-zero overhead |
| FC-DIA-01 | No Output directory check | VHS v0.3.0+ auto-creates directories |
| FC-DIA-02 | No LoopOffset range validation | VHS docs do not specify valid ranges |
| FC-DIA-03 | Validate Screenshot `.png` only (DIA-013) | VHS README confirms png-only |
| FC-DIA-04 | Cancellation-and-restart only | PATH lookups ~5ms; debounce not needed |
| FC-SAF-01 | No cross-Type sequence detection | High false-positive rate; Phase 3+ scope |
| FC-SAF-02 | Use `regex` crate (standard) | Needs `RegexSet`; `regex-lite` lacks it |
| FC-SAF-03 | No workspace config in Phase 2 | Defer to Phase 3 extension settings |
| FC-SAF-04 | No Env directive scanning | Niche attack vector; high false-positive risk |
