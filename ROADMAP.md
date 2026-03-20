# VHS-Analyzer: The Definitive Language Server & Extension for Tape

**Version:** 1.0 | **Date:** March 2026 | **Status:** Strategic Kickoff

---

Execution tracking and model-boundary handoff log: [`EXECUTION_TRACKER.md`](EXECUTION_TRACKER.md)
Machine-readable execution status: [`STATUS.yaml`](STATUS.yaml)
Agent recovery/navigation entrypoint: [`AGENTS.md`](AGENTS.md)
Agentic engineering methodology: [`docs/agentic-workflow.md`](docs/agentic-workflow.md)
Behavior specifications: [`spec/`](spec/README.md)
Kick files (role-scoped prompts): [`prompt/`](prompt/)

---

## 1. Executive Summary & Problem Statement

### 1.1 The Context

[VHS](https://github.com/charmbracelet/vhs) by Charmbracelet has become the standard tool for generating terminal GIFs and MP4s via declarative `.tape` scripts. It is heavily utilized for OSS project READMEs, CLI documentation, and agent demonstrations (including `eden-skills`).

### 1.2 The "IDE Experience" Vacuum

Despite its popularity, the developer experience for writing `.tape` files is severely lacking:

1. **Absence of a Language Server (LSP):** There is currently no official or third-party Language Server Protocol implementation for VHS. Developers lack autocomplete, hover documentation, error diagnostics, and formatting.
2. **Primitive Editor Plugins:** The existing `vscode-vhs` plugin relies on Charmbracelet's [`tree-sitter-vhs`](https://github.com/charmbracelet/tree-sitter-vhs) grammar for syntax highlighting only. It has no live preview, no semantic understanding, no diagnostics, and no command integration. Helix editor also integrates `tree-sitter-vhs` for highlighting but offers nothing beyond that.
3. **Friction in Iteration:** To preview a `.tape` file, a user must save the file, switch to the terminal, run `vhs tape.tape`, wait for rendering, and manually open the resulting GIF.

---

## 2. Strategic Vision & Core Architectural Decisions

### 2.1 The Goal: "rust-analyzer" Level Experience

Our objective is to build a production-grade, highly resilient Language Server and a rich VSCode/Cursor extension that provides an out-of-the-box, immersive IDE experience.

### 2.2 Core Technical Pillars

Based on Scout's architectural research, we have locked in the following 5 technical decisions:

1. **Parser & AST Strategy: `rowan` (Lossless Syntax Tree)**
   * We will build a handcrafted lexer and recursive descent parser using [`rowan`](https://github.com/rust-analyzer/rowan) (the exact foundation of `rust-analyzer`; v0.16.1, 11M+ downloads, actively maintained).
   * *Why:* It provides extreme error resiliency. Even when the user's `.tape` file is partially written or contains syntax errors, the AST will not collapse, allowing the LSP to continue providing accurate autocomplete and hover features.
   * *Reference:* matklad's [Resilient LL Parsing Tutorial](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html) provides the authoritative implementation blueprint for this exact architecture. The official [`tree-sitter-vhs` grammar.js](https://github.com/charmbracelet/tree-sitter-vhs/blob/main/grammar.js) serves as the ground-truth specification for the VHS tape language and will be used to validate our parser's correctness.

2. **LSP Framework: [`tower-lsp-server`](https://github.com/tower-lsp-community/tower-lsp-server) (Async Architecture)**
   * We use `tower-lsp-server` (v0.23.0), the actively maintained community fork of the now-unmaintained `tower-lsp`. Notable adopters include [oxc](https://github.com/oxc-project/oxc). It uses native `async fn` in traits (Rust 1.75+), eliminating the `async_trait` macro dependency.
   * *Why:* An asynchronous backbone is required to handle I/O-bound diagnostics (e.g., verifying if an executable exists in `$PATH`, checking if a directory exists for `Output`) without blocking user typing or autocomplete requests.

3. **Preview Mechanism: On-Demand Native Rendering**
   * *Why:* Pure frontend mock rendering cannot accurately represent a user's local shell state.
   * *Solution:* The VSCode client will provide integrated commands and CodeLens (`▶ Run this tape`). Execution triggers the native `vhs` CLI in the background, piping output to the editor's panel, and auto-refreshing a side-by-side Webview containing the rendered GIF/MP4 upon completion.
   * *Runtime Dependencies:* VHS itself requires `ttyd` (terminal emulation) and `ffmpeg` (video encoding). The extension should detect and surface missing dependencies via LSP diagnostics at startup, rather than failing silently at render time.

4. **Distribution Strategy: Target-Platform "Fat" VSIX**
   * *Why:* To guarantee a deterministic, zero-setup installation.
   * *Solution:* CI will cross-compile the Rust LSP binary for Windows, macOS (Intel/Apple Silicon), and Linux, bundling them directly into platform-specific VSIX packages. No runtime downloads required. A universal "no-server" fallback VSIX should also be published for unsupported architectures (e.g., RISC-V, LoongArch).
   * *CI Reference:* rust-analyzer's [`release.yaml`](https://github.com/rust-lang/rust-analyzer/blob/main/.github/workflows/release.yaml) is the canonical template for this exact workflow — GitHub Actions matrix build + `cross` tool + `vsce package --target`.

5. **Security & Validation: Proactive Safety Checks**
   * *Why:* `.tape` files execute real shell commands via `ttyd`.
   * *Solution:* The LSP will feature deep diagnostic rules to warn users about potentially destructive shell commands (e.g., `rm -rf`, `mkfs`) inside `Type` directives, acting as an essential safety net for downloaded or generated scripts.

---

## 3. AI Collaboration Roadmap

Following the proven `eden-skills` Agentic Engineering Workflow, we will leverage **Claude** (Architect / High Reasoning) and **GPT / Claude / Gemini** (Builder / High Speed Code Gen).

**Repository Location:** Hosted under the personal namespace [`DrEden33773/vhs-analyzer`](https://github.com/DrEden33773/vhs-analyzer) to align with hardcore systems engineering, while serving as a strategic top-of-funnel for the `AI-Eden` organization.
**Go-to-Market Strategy:** Build in **Private** for maximum velocity during Parser/LSP scaffolding. Flip to **Public** at the *end* of `Phase 1 & 2` / *early* `Phase 3` (once Hover and Diagnostics are demo-able in VSCode), launching with a "Show, don't tell" VHS-recorded demo GIF in the Charmbracelet community.

### Phase 1: The LSP Foundation (`vhs-analyzer`)

* **Role:** Architect defines the LSP protocol subset, AST node definitions, and the async state management design.
* **Role:** Builder writes the Rust implementation.
* **Deliverables:**
    1. **Lexer & Parser:** Full coverage of all VHS directives as defined by `tree-sitter-vhs` grammar.js:
       * **Output:** `Output` (`.gif`, `.mp4`, `.webm`)
       * **Actions:** `Type`, `Backspace`, `Down`, `Enter`, `Escape`, `Left`, `Right`, `Space`, `Tab`, `Up`, `PageUp`, `PageDown`, `Sleep`
       * **Modifiers:** `Ctrl+`, `Alt+`, `Shift+`
       * **Special:** `Hide`, `Show`, `Copy`, `Paste`, `Wait`, `Require`, `Source`, `Env`
       * **Settings (`Set`):** `Shell`, `FontFamily`, `FontSize`, `Framerate`, `PlaybackSpeed`, `Height`, `Width`, `LetterSpacing`, `TypingSpeed`, `LineHeight`, `Padding`, `Theme`, `LoopOffset`, `BorderRadius`, `Margin`, `MarginFill`, `WindowBar`, `WindowBarSize`, `CursorBlink`
       * **Literals:** string, regex, integer, float, boolean, json, path, time/duration
    2. **Language Server Core:** Connection via `tower-lsp-server`.
    3. **Basic Capabilities:** Hover documentation (pulling directly from VHS README definitions) and document formatting.

### Phase 2: Intelligence & Diagnostics

* **Role:** Architect designs the validation matrix and autocomplete heuristics.
* **Role:** Builder implements the diagnostic engine.
* **Deliverables:**
    1. **Context-Aware Autocomplete:** Smart completion for settings (`Set FontSize`, `Set Theme`), theme names, and time units.
    2. **Environment Diagnostics:** Warning on missing `Output` paths, invalid hex colors, or missing `Require` dependencies.
    3. **Safety Check Engine:** AST-based regex scanning of `Type` directives for high-risk bash commands.

### Phase 3: The VSCode Client (`vhs-analyzer`)

* **Role:** Architect defines the Extension API, Webview messaging protocol, and CI packaging spec.
* **Role:** Builder writes the TypeScript VSCode extension and GitHub Actions workflows.
* **Deliverables:**
    1. **LSP Client Bootstrapping:** Seamlessly launching the bundled Rust binary.
    2. **Side-by-Side Live Preview:** Webview panel for playing `.gif`/`.mp4` outputs with an auto-refresh listener on the generated artifact.
    3. **CodeLens & Commands:** Inline execution buttons.
    4. **Platform-Specific Packaging:** Matrix builds across `win32-x64`, `darwin-arm64`, `darwin-x64`, `linux-x64`, `linux-arm64`, plus a universal no-server fallback VSIX.

---

## 4. Next Steps (Action Items)

Operational progress for these items will be tracked in `STATUS.yaml` and `EXECUTION_TRACKER.md`.

### Pre-Phase 1: Project Initialization

1. [x] **Initialize Repo:** Create `vhs-analyzer` monorepo (`crates/vhs-analyzer-core`, `crates/vhs-analyzer-lsp`, `editors/vscode`). **(Completed)**
2. [x] **Workflow Setup:** Port `prompt/`, `spec/`, `trace/`, and coordination files from `eden-skills`. **(Completed)**

### Phase 1: LSP Foundation

1. [x] **(Architect Stage A)** Draft exploratory specs for Lexer, Parser, LSP Core, Hover, and Formatting. **(Completed)**
2. [x] **(Architect Stage B)** Freeze all specs with MUST/SHOULD/MAY contracts and test matrix. **(Completed)**
3. [x] **(Builder)** Handcraft Lexer and map VHS tokens. **(Completed)**
4. [x] **(Builder)** Implement Recursive Descent Parser. **(Completed)**
5. [x] **(Builder)** Wire up `tower-lsp-server` and implement `initialize` / `textDocument/didChange`. **(Completed)**
6. [x] **(Builder)** Implement `textDocument/hover` provider. **(Completed)**

### Phase 2: Diagnostics & Autocomplete

1. [x] Implement semantic validation (syntax errors, invalid paths).
2. [x] Implement Safety Check Engine (warn on destructive commands).
3. [x] Implement `textDocument/completion` provider.

### Phase 3: Extension Client

1. [x] Develop TypeScript client using `vscode-languageclient`.
2. [x] Build Live Preview Webview.
3. [x] Implement runtime dependency detection (warn if `vhs`, `ttyd`, `ffmpeg` are missing).
4. [x] Setup multi-target CI/CD via `vsce`.

---

## 5. Reference Materials & Validation

### 5.1 Competitive Landscape (as of March 2026)

There is **no existing LSP implementation** for VHS tape files anywhere. The only editor tooling available is syntax highlighting via `tree-sitter-vhs`. This project occupies a 100% blank market.

| Existing Tool | Scope | Gap |
| --- | --- | --- |
| [`vscode-vhs`](https://github.com/griimick/vscode-vhs) | tree-sitter syntax highlighting only | No LSP, no completion, no diagnostics, no preview |
| [`tree-sitter-vhs`](https://github.com/charmbracelet/tree-sitter-vhs) | Grammar definition for highlighting | No semantic analysis |
| [Helix VHS support](https://github.com/helix-editor/helix/pull/4486) | tree-sitter highlighting in Helix | No LSP |

### 5.2 Key Dependencies & Their Health

| Crate | Version | Status | Downloads (90d) |
| --- | --- | --- | --- |
| [`rowan`](https://crates.io/crates/rowan) | 0.16.1 | Active (rust-analyzer team, master development line) | 1.6M+ |
| [`tower-lsp-server`](https://crates.io/crates/tower-lsp-server) | 0.23.0 | Active (community fork, MSRV 1.85) | 166K+ |

### 5.3 Authoritative References

* **Parser Architecture:** [Resilient LL Parsing Tutorial](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html) by matklad (creator of rust-analyzer & rowan)
* **Language Specification:** [`tree-sitter-vhs/grammar.js`](https://github.com/charmbracelet/tree-sitter-vhs/blob/main/grammar.js) — the ground-truth grammar for VHS tape syntax
* **VHS Source (Go):** [`charmbracelet/vhs`](https://github.com/charmbracelet/vhs) — lexer/parser/token packages serve as behavioral reference
* **CI Blueprint:** rust-analyzer's [`release.yaml`](https://github.com/rust-lang/rust-analyzer/blob/main/.github/workflows/release.yaml) — platform-specific VSIX matrix build
* **Pratt Parsing:** [Simple but Powerful Pratt Parsing](https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html) by matklad (for potential future expression-level extensions)
* **`tower-lsp` Migration:** [oxc PR #10298](https://github.com/oxc-project/oxc/pull/10298) — real-world migration from `tower-lsp` to `tower-lsp-server`
