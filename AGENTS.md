# AGENTS.md

Agent coordination guide for `vhs-analyzer`.
This file is designed for fast recovery after context compression.

## 1. Read Order (Compression-Safe)

1. `ROADMAP.md`
2. `spec/README.md`
3. `spec/phase1/SPEC_*.md` (Phase 1 LSP Foundation contracts)
4. `spec/phase2/SPEC_*.md` (Phase 2 Diagnostics & Autocomplete contracts)
5. `spec/phase3/SPEC_*.md` (Phase 3 VSCode Extension Client contracts)
6. Current phase's `SPEC_TRACEABILITY.md`
7. `STATUS.yaml`
8. `EXECUTION_TRACKER.md`
9. `README.md`
10. `trace/` (archived phase records — read only when historical context is needed)

## 2. Authority Order

When files disagree, follow:

1. `spec/**/*.md`
2. `STATUS.yaml`
3. `EXECUTION_TRACKER.md`
4. `ROADMAP.md`
5. `README.md`

## 3. Role Boundaries

- `Builder` owns implementation, tests, refactors, and non-strategic doc sync.
- `Architect (Claude)` owns AST design, LSP protocol subset selection, validation matrix, and Extension API design.
- `Scout (Gemini)` owns market research, competitive analysis, and roadmap drafting.
- Builder MUST NOT finalize Architect-owned design outputs without explicit user instruction.
- Architect MUST NOT write implementation code in `crates/` or `editors/`.

## 4. Change Protocol

1. Update `spec/` first for behavior changes.
2. Implement code to match spec.
3. Update tests per the current phase's `SPEC_TEST_MATRIX.md`.
4. Update the current phase's `SPEC_TRACEABILITY.md` links for changed requirements.
5. Update `trace/<current-phase>/status.yaml` and `trace/<current-phase>/tracker.md` with batch progress.
6. Update root `STATUS.yaml` only when phase-level status changes (e.g., `not_started` → `in_progress` → `completed`).

## 5. Quick Start Task Routing

### Phase 1 (LSP Foundation)

- If task is lexer tokenization or token mapping: start from `spec/phase1/SPEC_LEXER.md`.
- If task is parser or AST node definition: start from `spec/phase1/SPEC_PARSER.md`.
- If task is `tower-lsp-server` wiring or LSP lifecycle: start from `spec/phase1/SPEC_LSP_CORE.md`.
- If task is hover documentation: start from `spec/phase1/SPEC_HOVER.md`.
- If task is document formatting: start from `spec/phase1/SPEC_FORMATTING.md`.
- If task is verification scope: start from `spec/phase1/SPEC_TEST_MATRIX.md`.

### Phase 2 (Intelligence & Diagnostics)

- If task is autocomplete or completion provider: start from `spec/phase2/SPEC_COMPLETION.md`.
- If task is semantic validation or diagnostics: start from `spec/phase2/SPEC_DIAGNOSTICS.md`.
- If task is safety check engine: start from `spec/phase2/SPEC_SAFETY.md`.

### Phase 3 (VSCode Extension Client)

- If task is LSP client bootstrapping: start from `spec/phase3/SPEC_CLIENT.md`.
- If task is live preview or Webview: start from `spec/phase3/SPEC_PREVIEW.md`.
- If task is CodeLens or inline commands: start from `spec/phase3/SPEC_CODELENS.md`.
- If task is CI/CD or VSIX packaging: start from `spec/phase3/SPEC_PACKAGING.md`.

### General

- If task is progress planning: use `STATUS.yaml` first, then `EXECUTION_TRACKER.md`.
- If task needs historical phase context: check `trace/<phase>/` archives.

## 6. Guardrails

- Preserve `rowan`-based lossless AST as the single source of truth for syntax analysis.
- Maintain `tower-lsp-server` as the sole LSP framework — do not introduce alternative LSP crates.
- Keep parser error-resilient: partial/invalid `.tape` files MUST produce a usable AST, never panic.
- Do not introduce Phase 3 extension client implementation into Phase 1 or Phase 2 specs.
- Phase specs are frozen after Stage B completion; changes require explicit user approval.
- Use `tree-sitter-vhs` grammar.js as the ground-truth reference for VHS tape language syntax.
- **Root files are index-only.** Root `STATUS.yaml` and `EXECUTION_TRACKER.md` are thin routing files — they contain project metadata, phase-level status pointers, and responsibility boundaries only. All phase-specific execution detail (stages, deliverables, stats, decision logs, batch progress, builder checkpoints) MUST live in `trace/<phase>/status.yaml` and `trace/<phase>/tracker.md`. Agents MUST NOT bloat root files with phase-internal data. See `trace/README.md` for the full convention.
- All `.md` files produced by any agent MUST pass `markdownlint-cli2`. See `.cursor/rules/markdown-output.mdc` for compliance details and `.markdownlint-cli2.jsonc` for the rule configuration.
- Rust comment strategy: prefer self-documenting code first. Library crates must document public APIs with `///` and keep `#![warn(missing_docs)]`; binary crates should prefer brief `//!` module docs plus concise `//` why-comments for non-obvious protocol, concurrency, UTF-16/range conversion, and context-resolution logic rather than blanket `///` coverage.
- **Phase 3 workspace convention:** When working on the VSCode extension in `editors/code/`, open the repository via `vhs-analyzer.code-workspace` instead of opening the repo root folder directly. The multi-root workspace intentionally disables the root Biome instance and enables Biome only for `editors/code/`. Root `/.vscode/settings.json` may exist for local editor preferences, but it is not the shared source of truth for Biome behavior.
