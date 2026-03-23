# Contributing to vhs-analyzer

Thanks for contributing.

## Before You Start

- For behavior changes, treat [`spec/`](spec/) as the source of truth.
- Follow the authority order from [`AGENTS.md`](AGENTS.md): `spec/` first, then [`STATUS.yaml`](STATUS.yaml), [`EXECUTION_TRACKER.md`](EXECUTION_TRACKER.md), [`ROADMAP.md`](ROADMAP.md), and finally [`README.md`](README.md).
- Use [`vhs-analyzer.code-workspace`](vhs-analyzer.code-workspace) when working on the extension in [`editors/code`](editors/code).

## Repository Areas

- [`crates/vhs-analyzer-core`](crates/vhs-analyzer-core): core language logic
- [`crates/vhs-analyzer-lsp`](crates/vhs-analyzer-lsp): LSP binary
- [`editors/code`](editors/code): VS Code / Cursor extension
- [`spec`](spec): frozen contracts and test matrices
- [`trace`](trace): per-phase execution records

## Change Protocol

This repository uses a spec-first workflow for behavior changes.

1. Identify the relevant phase and spec file.
2. Update the relevant spec before changing behavior.
3. Implement code to match the spec.
4. Add or update tests from the matching `SPEC_TEST_MATRIX.md`.
5. Update the matching `SPEC_TRACEABILITY.md` links.
6. Update `trace/<phase>/` records when phase execution state changes.
7. Update root status indexes only when the phase-level status changes.

For current task routing and guardrails, see [`AGENTS.md`](AGENTS.md).

## Contributing with AI Agents

If you plan to contribute with agentic engineering, read [`docs/agentic-workflow.md`](docs/agentic-workflow.md) first.

If you are orchestrating your own AI agent, it is also reasonable to make the agent read that document and summarize the workflow back to you before it starts making changes.

This is strongly recommended for larger or multi-step changes because the document explains:

- The Scout / Architect / Builder role split
- Why behavior changes are spec-first
- How batch handoffs and human review are expected to work
- How the `prompt/` and `trace/` archives fit into the repository history

This recommendation is not a prerequisite for small human-only fixes. For concise operational rules, start with [`AGENTS.md`](AGENTS.md).

## Local Commands

Rust workspace:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
```

Extension:

```bash
pnpm --dir editors/code install --frozen-lockfile
pnpm --dir editors/code typecheck
pnpm --dir editors/code lint
pnpm --dir editors/code test
pnpm --dir editors/code build
```

Release-oriented local check:

```bash
cargo build --release -p vhs-analyzer-lsp --locked
```

## Pull Requests

- Keep changes scoped and explain the user-visible or contract-level impact.
- Include spec and test updates when behavior changes.
- Keep Markdown files compliant with the repository's markdownlint rules.
- Do not use public issues for security reports. Follow [`SECURITY.md`](SECURITY.md).

## Questions

- Use GitHub issues for normal bugs, feature requests, and contributor questions once the repository is public.
- Use private reporting for security issues.
