# One-Hour Checklist Before Making the Repository Public

Status: **completed** — repository is public.

## Goal

Before switching the `vhs-analyzer` repository to public, finish the smallest
set of high-value actions that close repository entrypoint, governance, and
status-consistency gaps.

## Done Criteria

- A new visitor can understand what the project is, how to install it, where
  the code lives, and how to report issues within one minute.
- Top-level documents no longer imply that Phase 3 is unfinished.
- The repository has the minimum open-source governance surface:
  license, contribution entrypoint, and security reporting entrypoint.
- Public visitors are not confused by internal directories or historical naming.

## P0: Must Finish Before Going Public

- [x] Add a root `README.md`
  - Explain the project shape: Rust LSP + VS Code/Cursor extension
  - Explain the relationship to the official `vhs` CLI
  - Link `editors/code/README.md`, `spec/README.md`, and `STATUS.yaml`
  - Provide the shortest install and development entrypoint
- [x] Add `CONTRIBUTING.md`
  - Document the basic `cargo` and `pnpm` commands
  - Document that extension development should use
    `vhs-analyzer.code-workspace`
  - Briefly explain the `spec/`-first change protocol
- [x] Add `SECURITY.md`
  - Define how to report vulnerabilities
  - Clarify whether public issues are appropriate for security reports
- [x] Sync top-level status documents
  - `ROADMAP.md`
  - `EXECUTION_TRACKER.md`
  - `spec/README.md`
- [x] Normalize extension-path references to `editors/code`
  - Update public-facing docs and frozen specs for path consistency
  - Treat this as a documentation correction, not a behavior-contract change
- [x] Decide whether the following directories should remain public as-is
  - `prompt/` kept and explained in the root `README.md`
  - `trace/` kept and explained in the root `README.md`
  - `errors/` cleaned up by removing the stale troubleshooting artifact

## P0: Quick Checks Before Flipping Visibility

- [x] Re-scan the repository for secrets, PATs, account details, or temporary
  logs
- [x] Confirm the root `LICENSE` is visible and correct
- [x] Confirm `STATUS.yaml` matches `trace/phase3/status.yaml`
- [x] Confirm outward-facing docs no longer describe Phase 2 or Phase 3 as
  unfinished
- [x] Confirm all public entrypoints and docs point to `editors/code/`

## P1: Add Soon After Going Public

- [ ] `CODE_OF_CONDUCT.md`
- [ ] Issue templates
- [ ] Pull request template
- [ ] `SUPPORT.md` or support guidance in the root README
- [ ] Example `.tape` files or an `examples/` directory
- [ ] Demo GIFs, screenshots, or short videos
- [ ] Dependabot and GitHub security feature setup

## P2: Safe to Intentionally Defer

- [ ] A full documentation site or long-form manual
- [ ] A broader community-governance document set
- [ ] Fine-grained maintainer ownership rules
- [ ] Heavier marketing or automation assets

## Reference Files

- `AGENTS.md`
- `STATUS.yaml`
- `EXECUTION_TRACKER.md`
- `ROADMAP.md`
- `spec/README.md`
- `trace/phase3/status.yaml`
- `trace/phase3/tracker.md`
- `editors/code/README.md`
- `vhs-analyzer.code-workspace`

## Guiding Principle

- Use the current repository state as the source of truth.
- Use [rust-analyzer](https://github.com/rust-lang/rust-analyzer) as a model
  for governance completeness and release discipline, not as a scale target to
  clone.
- Finish the work that makes outside users understand and trust the project
  first. Heavier process can come later.
