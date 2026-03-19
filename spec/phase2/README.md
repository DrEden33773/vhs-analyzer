# Phase 2: Intelligence & Diagnostics

## Status: Stage A (Exploratory Design — 2026-03-19)

Phase 2 adds smart features on top of the LSP Foundation: context-aware
autocomplete, semantic diagnostics, and a safety check engine for `Type`
directives. All three work streams depend on Phase 1 frozen contracts but
are independent of each other.

## Spec Files

| File | Work Stream | Status | Description |
| --- | --- | --- | --- |
| `SPEC_COMPLETION.md` | WS-1 | Stage A | Context-aware autocomplete for commands, settings, themes, values |
| `SPEC_DIAGNOSTICS.md` | WS-2 | Stage A | Semantic validation, environment checks, unified diagnostic pipeline |
| `SPEC_SAFETY.md` | WS-3 | Stage A | Destructive command detection in Type directives |
| `SPEC_TEST_MATRIX.md` | — | Not started | Phase 2 acceptance test matrix (created during Stage B) |
| `SPEC_TRACEABILITY.md` | — | Not started | Requirement-to-implementation mapping (maintained by Builder) |

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

### Key Design Discovery (Stage A)

- **WS-3 → WS-2 integration**: Safety diagnostics are collected as a
  lightweight check (pure string pattern matching) and merged into the
  unified diagnostic pipeline defined in SPEC_DIAGNOSTICS.md DIA-011.
  This means WS-3 has a soft dependency on WS-2's pipeline architecture,
  though the safety detection logic itself is independent.

- **didSave handler**: Phase 2 extends Phase 1 by adding a `didSave`
  handler for heavyweight diagnostics (DIA-010). This is a cross-cutting
  concern that affects the server lifecycle design in WS-2.

- **Server capabilities**: Phase 2 adds `completionProvider` and
  `textDocumentSync.save` to the Phase 1 capability set (see
  SPEC_COMPLETION.md §10).

## Suggested Batch Progression

```text
Batch 1: WS-2 — Diagnostic pipeline infrastructure
         (lightweight semantic checks + heavyweight framework + didSave handler)
         This establishes the pipeline that WS-3 plugs into.

Batch 2: WS-3 — Safety check engine
         (pattern database + detection algorithm + suppression + pipeline integration)
         Leverages the diagnostic pipeline from Batch 1.

Batch 3: WS-1 — Completion provider
         (context resolution + item registries + snippets)
         Fully independent; can be parallelized with Batch 1-2 if desired.
```

**Rationale for reordering (vs. original):** The original ordering
(Completion → Diagnostics → Safety) was based on perceived user value
priority. The revised ordering (Diagnostics → Safety → Completion) is
based on implementation dependency: WS-2 creates the diagnostic pipeline
infrastructure that WS-3 consumes. WS-1 is independent and can be
scheduled in parallel.

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
