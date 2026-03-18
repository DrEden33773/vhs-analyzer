# Phase 2: Intelligence & Diagnostics

## Status: Not Started (depends on Phase 1 freeze)

Phase 2 adds smart features on top of the LSP Foundation: context-aware
autocomplete, semantic diagnostics, and a safety check engine for `Type` directives.

## Work Streams

```txt
WS-1: Completion    (SPEC_COMPLETION.md)   — autocomplete for Set, themes, time units
WS-2: Diagnostics   (SPEC_DIAGNOSTICS.md)  — semantic validation, environment checks
WS-3: Safety        (SPEC_SAFETY.md)       — destructive command detection
```

## Dependency Graph

```txt
Phase 1 (frozen)
  ├──> WS-1 (Completion)  — uses AST for completion context
  ├──> WS-2 (Diagnostics) — uses AST + LSP for publishing diagnostics
  └──> WS-3 (Safety)      — uses AST to extract Type directive content
```

All three work streams depend on Phase 1 but are independent of each other.
They MAY be implemented in parallel once Phase 1 is frozen.

## Suggested Batch Progression

```txt
Batch 1: WS-1 — Completion provider
Batch 2: WS-2 — Semantic diagnostics
Batch 3: WS-3 — Safety check engine
```
