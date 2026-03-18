# spec/

Implementation contracts for `vhs-analyzer` across all phases.

## Purpose

This directory defines executable specifications for LSP behavior and extension architecture.
`ROADMAP.md` explains strategy. `spec/` defines what must be built.

## Directory Structure

```txt
spec/
├── README.md              (this file — master index)
├── phase1/                (Phase 1: LSP Foundation — Lexer, Parser, LSP Core)
│   ├── README.md          (work stream index, dependency graph)
│   ├── SPEC_LEXER.md      (token definitions, lexer behavior)
│   ├── SPEC_PARSER.md     (AST node definitions, rowan integration, error resilience)
│   ├── SPEC_LSP_CORE.md   (tower-lsp-server wiring, initialize, didChange)
│   ├── SPEC_HOVER.md      (hover documentation provider)
│   ├── SPEC_FORMATTING.md (document formatting provider)
│   ├── SPEC_TEST_MATRIX.md
│   └── SPEC_TRACEABILITY.md
├── phase2/                (Phase 2: Intelligence & Diagnostics)
│   ├── README.md
│   ├── SPEC_COMPLETION.md (context-aware autocomplete)
│   ├── SPEC_DIAGNOSTICS.md(semantic validation, environment checks)
│   ├── SPEC_SAFETY.md     (destructive command detection in Type directives)
│   ├── SPEC_TEST_MATRIX.md
│   └── SPEC_TRACEABILITY.md
└── phase3/                (Phase 3: VSCode Extension Client)
    ├── README.md
    ├── SPEC_CLIENT.md     (LSP client bootstrapping, binary launch)
    ├── SPEC_PREVIEW.md    (side-by-side live preview Webview)
    ├── SPEC_CODELENS.md   (inline execution buttons)
    ├── SPEC_PACKAGING.md  (platform-specific VSIX, CI/CD matrix)
    ├── SPEC_TEST_MATRIX.md
    └── SPEC_TRACEABILITY.md
```

## Phase 1: LSP Foundation (Not Started)

Phase 1 specs will be drafted by the Architect in Stage A (exploratory) and
frozen in Stage B (contract). The Builder implements from frozen contracts only.

Planned spec files:

- `phase1/SPEC_LEXER.md`: VHS token set derived from `tree-sitter-vhs` grammar.js, lexer resilience, error tokens
- `phase1/SPEC_PARSER.md`: `rowan`-based lossless AST, recursive descent parser, node kinds, error recovery strategy
- `phase1/SPEC_LSP_CORE.md`: `tower-lsp-server` lifecycle, `initialize` capabilities, `textDocument/didOpen`, `textDocument/didChange`, incremental sync
- `phase1/SPEC_HOVER.md`: hover provider mapping AST nodes to VHS documentation strings
- `phase1/SPEC_FORMATTING.md`: document formatting rules (indentation, alignment, spacing)
- `phase1/SPEC_TEST_MATRIX.md`: minimum acceptance test matrix for Phase 1
- `phase1/SPEC_TRACEABILITY.md`: requirement IDs mapped to code and tests

## Phase 2: Intelligence & Diagnostics (Not Started)

Phase 2 extends the LSP Foundation with smart features. Depends on Phase 1 frozen contracts.

Planned spec files:

- `phase2/SPEC_COMPLETION.md`: context-aware autocomplete for `Set` settings, theme names, time units, file extensions
- `phase2/SPEC_DIAGNOSTICS.md`: semantic validation — syntax errors, invalid `Output` paths, invalid hex colors, missing `Require` dependencies
- `phase2/SPEC_SAFETY.md`: AST-based regex scanning of `Type` directives for high-risk bash commands (e.g., `rm -rf`, `mkfs`, `dd`)
- `phase2/SPEC_TEST_MATRIX.md`: Phase 2 acceptance test matrix
- `phase2/SPEC_TRACEABILITY.md`: Phase 2 requirement-to-implementation mapping

## Phase 3: VSCode Extension Client (Not Started)

Phase 3 builds the TypeScript VSCode/Cursor extension and CI/CD pipeline.

Planned spec files:

- `phase3/SPEC_CLIENT.md`: `vscode-languageclient` integration, LSP binary discovery and launch, configuration schema
- `phase3/SPEC_PREVIEW.md`: side-by-side Webview for GIF/MP4 preview, file watcher, auto-refresh
- `phase3/SPEC_CODELENS.md`: `▶ Run this tape` CodeLens, command registration, VHS CLI invocation
- `phase3/SPEC_PACKAGING.md`: platform-specific VSIX matrix build, `vsce package --target`, GitHub Actions CI, no-server fallback
- `phase3/SPEC_TEST_MATRIX.md`: Phase 3 acceptance test matrix
- `phase3/SPEC_TRACEABILITY.md`: Phase 3 requirement-to-implementation mapping

## Rule of Authority

When documents disagree, follow this order:

1. `spec/**/*.md` (normative behavior)
2. `STATUS.yaml` (machine-readable execution status)
3. `EXECUTION_TRACKER.md` (quantified progress and ownership)
4. `ROADMAP.md` (product strategy and milestones)
5. `README.md` (project summary)

## Normative Language

Keywords are interpreted as:

- `MUST`: mandatory behavior
- `SHOULD`: recommended behavior
- `MAY`: optional behavior

## Contributor Workflow

1. Identify which phase the change belongs to (`phase1/`, `phase2/`, or `phase3/`).
2. Update the relevant spec file first.
3. Implement code to match the spec.
4. Add or update tests from the corresponding `SPEC_TEST_MATRIX.md`.
5. Run `cargo fmt --all`, `cargo clippy --workspace`, and `cargo test --workspace`.
6. Update the corresponding `SPEC_TRACEABILITY.md` mappings.
7. If behavior changed, update `STATUS.yaml`, `EXECUTION_TRACKER.md`, `README.md`, and `ROADMAP.md`.

## Cross-Phase Extension Convention

Phase 2 spec files extend Phase 1 base contracts:

- `SPEC_COMPLETION.md` extends `phase1/SPEC_PARSER.md` (uses AST nodes for completion context)
- `SPEC_DIAGNOSTICS.md` extends `phase1/SPEC_PARSER.md` and `phase1/SPEC_LSP_CORE.md`

Phase 3 spec files consume Phase 1 and Phase 2 outputs:

- `SPEC_CLIENT.md` consumes the LSP binary produced by Phase 1 + Phase 2
- `SPEC_CODELENS.md` depends on `phase1/SPEC_PARSER.md` AST (to locate directive positions)

When reading an extension file, always read the corresponding base file first.
