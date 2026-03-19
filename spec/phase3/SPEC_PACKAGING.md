# SPEC_PACKAGING.md — Platform Packaging & CI/CD

**Phase:** 3 — VSCode Extension Client
**Work Stream:** WS-4 (Packaging)
**Status:** Stage A (Exploratory Design)
**Owner:** Architect
**Depends On:** WS-1 (SPEC_CLIENT.md — extension manifest, activation), Phase 1+2 Rust binary
**Last Updated:** 2026-03-19

---

## 1. Purpose

Define the platform-specific VSIX packaging strategy, Rust cross-compilation
approach, GitHub Actions CI/CD workflows, extension manifest structure, and
multi-registry publishing pipeline. This work stream packages the Rust LSP
binary (from Phase 1+2) and the TypeScript extension (from WS-1/2/3) into
distributable VSIX files for each target platform.

## 2. Cross-Phase Dependencies

| Phase 1/2 Contract | Usage in This Spec |
| --- | --- |
| SPEC_LSP_CORE.md — LSP-001 (stdio transport) | The packaged binary MUST be the same `vhs-analyzer-lsp` that communicates via stdio |
| SPEC_LSP_CORE.md — FC-LSP-04 (MSRV 1.85) | Cross-compilation MUST use Rust ≥1.85 toolchain |
| Phase 1+2 CI (`.github/workflows/ci.yml`) | Phase 3 CI extends existing Rust CI with TypeScript checks |

| Phase 3 Spec | Integration |
| --- | --- |
| SPEC_CLIENT.md — CLI-001 (Binary discovery) | Bundled binary path: `{extensionPath}/server/vhs-analyzer-lsp[.exe]` |
| SPEC_CLIENT.md — CLI-008 (TextMate grammar) | Grammar file included in VSIX |
| SPEC_CLIENT.md — CLI-009 (No-server mode) | Universal VSIX operates in no-server mode |

## 3. References

| Source | Role |
| --- | --- |
| [rust-analyzer release.yaml](https://github.com/rust-lang/rust-analyzer/blob/main/.github/workflows/release.yaml) | Canonical CI blueprint for Rust LSP platform VSIX |
| [vscode-platform-specific-sample](https://github.com/microsoft/vscode-platform-specific-sample) | Microsoft's reference implementation for platform VSIX |
| [Publishing Extensions (vsce)](https://code.visualstudio.com/docs/tools/vscecli) | `vsce package --target`, VSIX publishing |
| [cross-rs](https://github.com/cross-rs/cross) | Docker-based Rust cross-compilation |
| [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) | Zig-based Rust cross-compilation |
| [pnpm + vsce workaround](https://opensciencelabs.org/blog/packaging-a-vs-code-extension-using-pnpm-and-vsce/) | `--no-dependencies` flag for pnpm compatibility |
| Rust Best Practices skill | Cross-compilation targets and binary packaging constraints |

## 4. Requirements

### PKG-001 — Target Platform Matrix

| Field | Value |
| --- | --- |
| **ID** | PKG-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The CI MUST produce platform-specific VSIX packages for the following targets, each embedding the corresponding Rust binary at `server/vhs-analyzer-lsp[.exe]`: (1) `win32-x64` (Windows x86_64, `.exe`), (2) `darwin-arm64` (macOS Apple Silicon), (3) `darwin-x64` (macOS Intel), (4) `linux-x64` (Linux x86_64, glibc), (5) `linux-arm64` (Linux ARM64, glibc), (6) `alpine-x64` (Linux x86_64, musl — for Alpine/Docker). Additionally, a universal VSIX without bundled binary MUST be produced for unsupported platforms (no-server mode). Total: 7 VSIX packages. |
| **Verification** | CI matrix produces 7 VSIX files. Each platform-specific VSIX contains the binary at `server/vhs-analyzer-lsp`. Universal VSIX does not contain `server/`. |

### PKG-002 — Rust Binary Cross-Compilation

| Field | Value |
| --- | --- |
| **ID** | PKG-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The CI MUST cross-compile the `vhs-analyzer-lsp` binary for each target triple: (1) `x86_64-pc-windows-msvc`, (2) `aarch64-apple-darwin`, (3) `x86_64-apple-darwin`, (4) `x86_64-unknown-linux-gnu`, (5) `aarch64-unknown-linux-gnu`, (6) `x86_64-unknown-linux-musl`. Binaries MUST be built in `--release` mode with LTO enabled (`lto = true` in `Cargo.toml` release profile). Binaries MUST be stripped of debug symbols (`strip = true`). |
| **Verification** | Each binary runs on its target platform. `file` command confirms correct architecture. Binary size is reasonable (<15 MB stripped). |

### PKG-003 — Extension Manifest (package.json)

| Field | Value |
| --- | --- |
| **ID** | PKG-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `package.json` MUST include: (1) `name: "vhs-analyzer"`, (2) `displayName: "VHS Analyzer"`, (3) `publisher: "{publisher-id}"`, (4) `engines.vscode: "^1.85.0"`, (5) `main: "./dist/extension.js"` (esbuild bundle output), (6) `activationEvents: ["onLanguage:tape"]`, (7) `categories: ["Programming Languages", "Linters", "Formatters"]`, (8) `contributes: { languages, grammars, commands, configuration, menus, keybindings }` as defined in SPEC_CLIENT.md and SPEC_CODELENS.md. The `scripts` section MUST define: `build`, `watch`, `lint`, `format`, `test`, `typecheck`, `package`, `publish`. |
| **Verification** | `vsce ls` shows all expected files. `vsce package` succeeds without warnings. `code --install-extension *.vsix` installs successfully. |

### PKG-004 — esbuild Bundle Configuration

| Field | Value |
| --- | --- |
| **ID** | PKG-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST be bundled using esbuild with the following configuration: (1) entry: `src/extension.ts`, (2) outfile: `dist/extension.js`, (3) format: `cjs` (VSCode requires CommonJS), (4) platform: `node`, (5) external: `["vscode"]`, (6) bundle: `true`, (7) minify: `true` (production), (8) sourcemap: `true` (development) / `false` (production). The bundled extension MUST be a single file with no `node_modules` dependency at runtime (all npm packages inlined except `vscode`). |
| **Verification** | `dist/extension.js` is a single file. Extension activates and all features work from the bundle. Bundle size is <500 KB. |

### PKG-005 — pnpm + vsce Compatibility

| Field | Value |
| --- | --- |
| **ID** | PKG-005 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Since `pnpm` uses a symlinked `node_modules` structure incompatible with `vsce`, the packaging step MUST use `vsce package --no-dependencies` to skip the `npm install --production` step that `vsce` normally performs. The esbuild bundle (PKG-004) already contains all runtime dependencies. The `.vscodeignore` file MUST exclude: `node_modules/`, `src/`, `*.ts` (source), `.github/`, `biome.json`, `tsconfig.json`, `vitest.config.ts`. |
| **Verification** | `vsce package --no-dependencies` succeeds. VSIX contains only `dist/`, `server/` (if platform-specific), `syntaxes/`, `language-configuration.json`, `package.json`, `README.md`, `LICENSE`, and `CHANGELOG.md`. |

### PKG-006 — GitHub Actions Release Workflow

| Field | Value |
| --- | --- |
| **ID** | PKG-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The release workflow (`.github/workflows/release.yml`) MUST: (1) Trigger on version tag push (`v*`) or manual `workflow_dispatch`. (2) Run jobs in sequence: `lint-and-test` → `build-rust` → `package-vsix` → `publish`. (3) `lint-and-test` job: `pnpm install`, `biome check`, `vitest run`, `tsc --noEmit`. (4) `build-rust` job: matrix build for 6 targets (PKG-002), upload binaries as artifacts. (5) `package-vsix` job: download binary artifacts, place in `server/`, run `vsce package --target {platform} --no-dependencies` for each target + universal. (6) `publish` job: publish all VSIX files to VSCode Marketplace and Open VSX. |
| **Verification** | Push `v0.3.0` tag → workflow runs all jobs → 7 VSIX files published. Manual dispatch → same result. |

### PKG-007 — TypeScript CI Pipeline

| Field | Value |
| --- | --- |
| **ID** | PKG-007 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | A CI workflow (`.github/workflows/ci.yml` extension or separate `extension-ci.yml`) MUST run on every push and PR affecting `editors/vscode/**`. Steps: (1) `pnpm install --frozen-lockfile`, (2) `pnpm run typecheck` (`tsc --noEmit`), (3) `pnpm run lint` (`biome check .`), (4) `pnpm run test` (`vitest run`), (5) `pnpm run build` (esbuild production bundle). The workflow MUST use the same Node.js version as the extension's `engines.node` constraint (≥18). |
| **Verification** | PR modifying `editors/vscode/src/extension.ts` → CI runs TypeScript checks. PR modifying only `crates/` → TypeScript CI does not run (path filter). |

### PKG-008 — Publishing Strategy

| Field | Value |
| --- | --- |
| **ID** | PKG-008 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST be published to: (1) **VSCode Marketplace** (primary) — via `vsce publish --target {platform} --packagePath {vsix}`, (2) **Open VSX Registry** (Cursor/open-source editor compatibility) — via `ovsx publish --target {platform} {vsix}`, (3) **GitHub Releases** — attach all 7 VSIX files as release assets for manual installation. Publishing credentials (PAT tokens) MUST be stored as GitHub Actions secrets: `VSCE_PAT` and `OVSX_PAT`. |
| **Verification** | Extension installable from VSCode Marketplace, Open VSX, and GitHub Releases. |

### PKG-009 — No-Server Fallback VSIX

| Field | Value |
| --- | --- |
| **ID** | PKG-009 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The universal (no-server) VSIX MUST be packaged without the `--target` flag (or with `--target universal`). It MUST NOT contain any binary in `server/`. On installation, the extension operates in no-server mode (SPEC_CLIENT.md CLI-009): syntax highlighting, CodeLens, and Preview work; completion, diagnostics, hover, and formatting do not. The VSIX description MUST clearly state: "Universal version — install the platform-specific version for full language server support." |
| **Verification** | Install universal VSIX → extension activates in no-server mode. Syntax highlighting works. Completion does not work. |

### PKG-010 — Biome Configuration

| Field | Value |
| --- | --- |
| **ID** | PKG-010 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `editors/vscode/biome.json` MUST configure Biome as the sole linter and formatter for the TypeScript codebase. Configuration MUST enable: (1) `formatter.indentStyle: "space"`, `formatter.indentWidth: 2`, (2) `linter.enabled: true` with recommended rules, (3) `organizeImports.enabled: true`. Files excluded: `dist/`, `node_modules/`, `*.json` (VSCode JSON uses its own schema validation). |
| **Verification** | `biome check .` passes with zero errors. `biome format --write .` produces no changes on CI (already formatted). |

### PKG-011 — Vitest Configuration

| Field | Value |
| --- | --- |
| **ID** | PKG-011 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `editors/vscode/vitest.config.ts` MUST configure Vitest for the extension test suite: (1) `test.include: ["src/**/*.test.ts"]`, (2) `test.environment: "node"`, (3) `test.globals: false` (explicit imports), (4) `test.coverage.provider: "v8"`, (5) `test.coverage.include: ["src/**/*.ts"]`, (6) `test.coverage.exclude: ["src/**/*.test.ts", "src/**/*.d.ts"]`. External modules (`vscode`) MUST be mocked via `vi.mock("vscode", ...)`. |
| **Verification** | `vitest run` executes all tests. `vitest run --coverage` produces coverage report. Tests pass in CI. |

### PKG-012 — Rust Release Profile

| Field | Value |
| --- | --- |
| **ID** | PKG-012 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | The `Cargo.toml` workspace SHOULD include a release profile optimized for distribution: `[profile.release]` with `lto = true` (link-time optimization), `strip = true` (strip debug symbols), `codegen-units = 1` (maximum optimization), `opt-level = "s"` (size optimization — LSP servers benefit from fast startup and small binary over raw throughput). |
| **Verification** | Release binary size is <15 MB for each platform. Startup time is <200ms. |

## 5. Design Options Analysis

### 5.1 Cross-Compilation Approach

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Native runners + cross-rs for non-native** | macOS targets on macOS runner (native), Windows on Windows runner (native), Linux ARM on Ubuntu with cross-rs | Native builds for critical platforms (macOS universal signing); cross-rs for Linux ARM | Multiple runner OSes; macOS runners are expensive; cross-rs Docker overhead |
| **B: cargo-zigbuild for all from Ubuntu** | All 6 targets compiled on a single Ubuntu runner using Zig as the linker | Single runner; fast CI; no Docker; small binary sizes | Windows MSVC target is experimental in zigbuild; macOS code signing not possible from Linux |
| **C: All native runners** | Each target on its own native runner (macOS-arm64 for darwin-arm64, etc.) | Simplest build; guaranteed compatibility | 6 different runners; expensive; macOS ARM64 runner availability |

**Recommended: Option A (Native runners + cross-rs).** This matches the
rust-analyzer CI blueprint. macOS builds MUST use native macOS runners for
code signing and universal binary support. Windows MSVC builds are most
reliable on Windows runners. Linux targets use Ubuntu runners with cross-rs
for ARM64 and musl. cargo-zigbuild (Option B) has known issues with
Windows MSVC targets and cannot produce macOS signed binaries. Option C
is prohibitively expensive for macOS ARM64 runners.

### 5.2 CI Workflow Structure

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Single unified workflow** | One `release.yml` with all jobs: Rust build matrix → TS lint/test → VSIX packaging → publish | Single trigger; shared context; artifact passing between jobs | Large workflow file; all steps run on every release |
| **B: Separate workflows with dispatch** | `rust-build.yml` (Rust matrix), `extension-ci.yml` (TS checks), `release.yml` (orchestrator calling both) | Modular; reusable; independent debugging | Cross-workflow artifact passing is complex; dispatch chain |
| **C: Separate workflows, release combines** | `ci.yml` (Rust + TS checks on PR/push), `release.yml` (builds + packages + publishes on tag) | CI and release are independent; CI runs on every push; release is tag-only | Duplication of some build steps between CI and release |

**Recommended: Option C (Separate CI + Release).** `ci.yml` runs on every
push/PR for both Rust and TypeScript (path-filtered). `release.yml` runs only
on version tags and performs the full build → package → publish pipeline.
This avoids running expensive cross-compilation on every PR while ensuring
every tag produces distributable artifacts. This matches the rust-analyzer
pattern.

### 5.3 VSIX Packaging Method

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: vsce package --target per platform** | Run `vsce package --target {platform} --no-dependencies` for each target, placing the binary beforehand | Official method; platform-specific VSIX format | Multiple `vsce package` invocations; requires binary placement per target |
| **B: Manual VSIX assembly** | Create VSIX zip files manually with correct structure | Full control; no vsce quirks | Must replicate VSIX format exactly; fragile; no manifest validation |
| **C: vsce with pre-install script** | Use vsce `prepackage` script to copy correct binary based on `--target` | Automated binary placement | vsce does not pass `--target` to pre-scripts; hard to parametrize |

**Recommended: Option A (vsce package --target).** The official method.
The CI workflow downloads the correct binary artifact, places it at
`server/vhs-analyzer-lsp`, then runs `vsce package --target {platform}
--no-dependencies`. After packaging, the binary is removed before the
next target. This is exactly what rust-analyzer does.

### 5.4 Open VSX Publishing

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Publish to both Marketplace and Open VSX** | Use both `vsce publish` and `ovsx publish` in the release workflow | Maximum reach; Cursor/VSCodium/Gitpod users get the extension | Two sets of credentials; two publishing steps |
| **B: VSCode Marketplace only** | Publish only to the official marketplace | Simplest; single credential | Cursor/VSCodium users cannot install from their default registry |
| **C: Open VSX only** | Publish only to Open VSX | Open-source friendly | VSCode users lose one-click install from the built-in marketplace |

**Recommended: Option A (Both Marketplace and Open VSX).** The extension
targets both VSCode and Cursor (per ROADMAP.md). Cursor uses Open VSX as
its extension registry. Publishing to both ensures all users can install
via their editor's built-in marketplace. The additional CI step is trivial.

## 6. Target Platform ↔ Rust Triple Mapping

| vsce Target | Rust Triple | Runner OS | Build Method | Binary Name |
| --- | --- | --- | --- | --- |
| `win32-x64` | `x86_64-pc-windows-msvc` | `windows-latest` | Native cargo | `vhs-analyzer-lsp.exe` |
| `darwin-arm64` | `aarch64-apple-darwin` | `macos-14` (ARM) | Native cargo | `vhs-analyzer-lsp` |
| `darwin-x64` | `x86_64-apple-darwin` | `macos-13` (Intel) | Native cargo | `vhs-analyzer-lsp` |
| `linux-x64` | `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Native cargo | `vhs-analyzer-lsp` |
| `linux-arm64` | `aarch64-unknown-linux-gnu` | `ubuntu-latest` | cross-rs | `vhs-analyzer-lsp` |
| `alpine-x64` | `x86_64-unknown-linux-musl` | `ubuntu-latest` | cross-rs | `vhs-analyzer-lsp` |
| (universal) | — | — | No binary | — |

## 7. Release Workflow Pseudocode

```yaml
name: Release
on:
  push:
    tags: ["v*"]
  workflow_dispatch:

jobs:
  lint-and-test:
    runs-on: ubuntu-latest
    steps:
      - checkout
      - setup-node (20.x)
      - setup-pnpm
      - pnpm install --frozen-lockfile (in editors/vscode/)
      - pnpm run typecheck
      - pnpm run lint
      - pnpm run test
      - cargo fmt --all -- --check
      - cargo clippy --workspace -- -D warnings
      - cargo test --workspace

  build-rust:
    needs: lint-and-test
    strategy:
      matrix:
        include:
          - { target: x86_64-pc-windows-msvc,     os: windows-latest, use-cross: false }
          - { target: aarch64-apple-darwin,         os: macos-14,       use-cross: false }
          - { target: x86_64-apple-darwin,          os: macos-13,       use-cross: false }
          - { target: x86_64-unknown-linux-gnu,     os: ubuntu-latest,  use-cross: false }
          - { target: aarch64-unknown-linux-gnu,    os: ubuntu-latest,  use-cross: true  }
          - { target: x86_64-unknown-linux-musl,    os: ubuntu-latest,  use-cross: true  }
    runs-on: ${{ matrix.os }}
    steps:
      - checkout
      - install rust toolchain (stable, target: ${{ matrix.target }})
      - if use-cross: cargo install cross
      - build: cross/cargo build --release --target ${{ matrix.target }} -p vhs-analyzer-lsp
      - upload artifact: target/${{ matrix.target }}/release/vhs-analyzer-lsp[.exe]

  package-vsix:
    needs: build-rust
    runs-on: ubuntu-latest
    strategy:
      matrix:
        include:
          - { vsce-target: win32-x64,     rust-target: x86_64-pc-windows-msvc,   binary: vhs-analyzer-lsp.exe }
          - { vsce-target: darwin-arm64,   rust-target: aarch64-apple-darwin,       binary: vhs-analyzer-lsp }
          - { vsce-target: darwin-x64,     rust-target: x86_64-apple-darwin,        binary: vhs-analyzer-lsp }
          - { vsce-target: linux-x64,      rust-target: x86_64-unknown-linux-gnu,   binary: vhs-analyzer-lsp }
          - { vsce-target: linux-arm64,    rust-target: aarch64-unknown-linux-gnu,  binary: vhs-analyzer-lsp }
          - { vsce-target: alpine-x64,     rust-target: x86_64-unknown-linux-musl,  binary: vhs-analyzer-lsp }
    steps:
      - checkout
      - setup-node, setup-pnpm
      - pnpm install --frozen-lockfile (in editors/vscode/)
      - pnpm run build (esbuild production bundle)
      - download rust binary artifact for ${{ matrix.rust-target }}
      - mkdir -p editors/vscode/server/
      - cp artifact/${{ matrix.binary }} editors/vscode/server/
      - chmod +x editors/vscode/server/${{ matrix.binary }}
      - cd editors/vscode && vsce package --target ${{ matrix.vsce-target }} --no-dependencies
      - upload VSIX artifact

  package-universal:
    needs: [lint-and-test]
    runs-on: ubuntu-latest
    steps:
      - checkout
      - setup-node, setup-pnpm
      - pnpm install, pnpm run build
      - cd editors/vscode && vsce package --no-dependencies
      - upload VSIX artifact

  publish:
    needs: [package-vsix, package-universal]
    runs-on: ubuntu-latest
    steps:
      - download all VSIX artifacts
      - for each VSIX:
          vsce publish --packagePath $vsix --pat ${{ secrets.VSCE_PAT }}
          ovsx publish $vsix --pat ${{ secrets.OVSX_PAT }}
      - create GitHub Release with all VSIX files attached
```

## 8. Extension Manifest Structure (package.json overview)

```json
{
  "name": "vhs-analyzer",
  "displayName": "VHS Analyzer",
  "description": "Language server, live preview, and CodeLens for VHS .tape files",
  "version": "0.3.0",
  "publisher": "dreden33773",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/DrEden33773/vhs-analyzer"
  },
  "engines": {
    "vscode": "^1.85.0"
  },
  "categories": [
    "Programming Languages",
    "Linters",
    "Formatters"
  ],
  "keywords": [
    "vhs",
    "tape",
    "terminal",
    "recording",
    "gif",
    "lsp"
  ],
  "activationEvents": [
    "onLanguage:tape"
  ],
  "main": "./dist/extension.js",
  "contributes": {
    "languages": [
      {
        "id": "tape",
        "extensions": [".tape"],
        "aliases": ["VHS Tape", "tape"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "tape",
        "scopeName": "source.tape",
        "path": "./syntaxes/tape.tmLanguage.json"
      }
    ],
    "commands": "/* see SPEC_CODELENS.md §6 */",
    "configuration": "/* see SPEC_CLIENT.md §8 */",
    "menus": "/* see SPEC_CODELENS.md §6 */",
    "keybindings": "/* see SPEC_CODELENS.md §6 */"
  },
  "scripts": {
    "build": "esbuild src/extension.ts --bundle --outfile=dist/extension.js --external:vscode --format=cjs --platform=node --minify",
    "watch": "esbuild src/extension.ts --bundle --outfile=dist/extension.js --external:vscode --format=cjs --platform=node --sourcemap --watch",
    "lint": "biome check .",
    "format": "biome format --write .",
    "test": "vitest run",
    "typecheck": "tsc --noEmit",
    "package": "vsce package --no-dependencies",
    "publish": "vsce publish --no-dependencies"
  },
  "devDependencies": {
    "@biomejs/biome": "^1.x",
    "@types/node": "^18",
    "@types/vscode": "^1.85.0",
    "@vscode/vsce": "^2.x",
    "esbuild": "^0.x",
    "typescript": "^5.x",
    "vitest": "^3.x"
  },
  "dependencies": {
    "vscode-languageclient": "^9.x"
  }
}
```

## 9. .vscodeignore Configuration

```gitignore
**/*.ts
**/src/
**/node_modules/
**/tsconfig.json
**/biome.json
**/vitest.config.ts
**/.github/
**/.gitignore
**/pnpm-lock.yaml
**/*.test.ts
**/coverage/
```

## 10. CI Integration with Existing Rust CI

The existing Phase 1+2 Rust CI (`.github/workflows/ci.yml`) runs:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --all-targets --locked`

Phase 3 extends this with TypeScript checks. Two approaches:

| Approach | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Extend existing ci.yml** | Add a `typescript` job to the existing workflow, filtered by `editors/vscode/**` path | Single CI config; unified status checks | Larger workflow; Rust jobs run on TS-only changes (unless path-filtered) |
| **B: Separate extension-ci.yml** | New workflow triggered by `editors/vscode/**` path changes | Clean separation; independent caching | Two CI workflows; PR requires both to pass |

**Recommended: Option B (Separate extension-ci.yml).** Path-filtered workflows
ensure TypeScript CI only runs when extension code changes, and Rust CI only
runs when Rust code changes. Both are required status checks for merge.
The release workflow (`release.yml`) runs both Rust and TypeScript checks
unconditionally.

## 11. Freeze Candidates

### FC-PKG-01 — macOS Universal Binary

**Question:** Should macOS produce a universal binary (fat binary containing
both ARM64 and x86_64), or separate binaries per architecture?

**Analysis:** A universal binary via `lipo -create` produces a single binary
that runs natively on both Intel and Apple Silicon Macs. This allows publishing
a single `darwin-universal` VSIX instead of two separate macOS VSIXes.
However, `vsce` supports `darwin-arm64` and `darwin-x64` as separate targets.
rust-analyzer publishes separate per-architecture VSIXes.

**Leaning:** Separate per-architecture VSIXes (`darwin-arm64` + `darwin-x64`),
matching rust-analyzer. A universal binary may be added later.

### FC-PKG-02 — Binary Size Optimization Level

**Question:** Should the release profile use `opt-level = "s"` (size) or
`opt-level = 3` (speed)?

**Analysis:** An LSP server is I/O-bound (waiting for JSON-RPC messages,
parsing small files). CPU throughput is not the bottleneck. Smaller binary
= faster extension installation and smaller VSIX download. `opt-level = "s"`
typically produces 20-30% smaller binaries with negligible performance impact
for I/O-bound workloads.

**Leaning:** `opt-level = "s"` for size optimization.

### FC-PKG-03 — pnpm Version Pinning

**Question:** Should the CI pin a specific pnpm version or use `latest`?

**Analysis:** Pinning ensures reproducible builds. `pnpm-lock.yaml` format
may change between major versions. Using `corepack enable && corepack prepare
pnpm@9 --activate` (or `packageManager` field in root `package.json`)
provides deterministic pnpm version resolution.

**Leaning:** Pin via `"packageManager": "pnpm@9.x.x"` in `package.json`.

### FC-PKG-04 — Extension Version Scheme

**Question:** What versioning scheme should the extension follow? Should it
track the Rust crate version or have its own?

**Analysis:** The extension and the Rust LSP server are tightly coupled but
have different release cadences. rust-analyzer uses a date-based scheme
(`YYYY-MM-DD`). Traditional `semver` is clearer for marketplace users.
The extension version should be independent of the Rust crate version, since
extension-only changes (UI, Preview improvements) do not require a Rust
release.

**Leaning:** Independent semver for the extension (`0.3.0` = first Phase 3
release). The Rust crate remains at `0.2.x` from Phase 2.

### FC-PKG-05 — GitHub Releases: Pre-release vs. Release

**Question:** Should early releases be marked as pre-release on GitHub?

**Analysis:** Pre-release tags (`v0.3.0-beta.1`) allow early adopters to
test while stable users are not notified. The VSCode Marketplace supports
pre-release extensions via the `--pre-release` flag.

**Leaning:** Use pre-release for `v0.3.0-beta.*` and stable for `v0.3.0`.
The release workflow detects pre-release from the tag format.

### FC-PKG-06 — ovsx CLI Installation in CI

**Question:** Should the CI use the `ovsx` npm package directly or the
`open-vsx/publish-extensions` GitHub Action?

**Analysis:** The `ovsx` CLI (`npx ovsx publish`) is the direct approach.
The GitHub Action wraps it with additional features (retry, error handling).
For simplicity, `npx ovsx publish` is sufficient.

**Leaning:** Direct `npx ovsx publish` invocation.
