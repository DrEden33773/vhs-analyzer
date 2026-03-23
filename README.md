# vhs-analyzer

`vhs-analyzer` is a language tooling project for
[VHS](https://github.com/charmbracelet/vhs) `.tape` files. It combines a Rust language server with a VS Code / Cursor extension so that authoring tape files feels closer to working with a modern programming language.

## What It Includes

- A Rust LSP server for hover, completion, diagnostics, formatting, and safety checks.
- A VS Code / Cursor extension for editor activation, CodeLens actions, live preview, and packaged VSIX distribution.
- Cross-platform packaging and CI workflows for bundled and universal extension builds.

## Relationship to the Official VHS CLI

This project complements the official `vhs` CLI. `vhs` remains the renderer and runtime for `.tape` scripts. `vhs-analyzer` adds authoring-time tooling, validation, and editor workflows around that runtime.

Preview and Run features still depend on local `vhs`, `ttyd`, and `ffmpeg` installations.

## Project Status

All planned implementation phases are complete.

- Machine-readable status: [`STATUS.yaml`](STATUS.yaml)
- Specification index: [`spec/README.md`](spec/README.md)
- Extension usage and packaging details: [`editors/code/README.md`](editors/code/README.md)

## Repository Layout

- [`crates/vhs-analyzer-core`](crates/vhs-analyzer-core): lexer, parser, AST, formatting, and shared language logic
- [`crates/vhs-analyzer-lsp`](crates/vhs-analyzer-lsp): `tower-lsp-server` binary for editor integration
- [`editors/code`](editors/code): VS Code / Cursor extension
- [`spec`](spec): frozen behavior and packaging contracts
- [`trace`](trace): per-phase execution records and closeout history
- [`publish-helper`](publish-helper): public-release and release-day checklists

## Quick Start

If you want to understand the project quickly, start here:

1. Read this file.
2. Open [`editors/code/README.md`](editors/code/README.md) for the extension feature set and packaging model.
3. Open [`spec/README.md`](spec/README.md) if you want the contract-level view.

If you want to develop locally:

```bash
cargo build --release -p vhs-analyzer-lsp --locked
pnpm --dir editors/code install --frozen-lockfile
pnpm --dir editors/code build
```

For extension development, open the repository with [`vhs-analyzer.code-workspace`](vhs-analyzer.code-workspace) instead of opening the repo root folder directly.

## Development Checks

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

## Contributing and Security

- Contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- Security policy: [`SECURITY.md`](SECURITY.md)
- Non-security bugs and feature requests should go through GitHub issues once the repository is public.

## License

MIT. See [`LICENSE`](LICENSE).
