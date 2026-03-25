# vhs-analyzer Publicity Strategy

## Current State

- vhs-analyzer is the **first LSP language server** for VHS `.tape` files, with full support for hover, completion, diagnostics, formatting, and safety checks.
- The VS Code / Cursor extension is published to both Marketplace and Open VSX (v0.1.1), with CodeLens and live preview.
- GitHub has 5 stars. No active promotion has been done yet.
- Solo developer, 80 commits, 287 automated tests, all three phases complete.

## Competitive Landscape

The upstream VHS project (charmbracelet/vhs) has ~19K stars and a large user base, but editor tooling is extremely limited:

| Existing tool | Capability | Gap |
| --- | --- | --- |
| `griimick/vscode-vhs` | Syntax highlighting only | No LSP, no diagnostics, no completion, no preview |
| `charmbracelet/tree-sitter-vhs` | Syntax highlighting only (Neovim/Emacs) | Same |
| Zed VHS extension | Syntax highlighting only | Same |

**No competing VHS LSP implementation exists.** VHS core contributor `caarlos0` mentioned "or a lsp server" in [issue #162](https://github.com/charmbracelet/vhs/issues/162) back in 2022. No official implementation has followed.

## Core Selling Points

Ranked by audience priority:

| Priority | Selling point | Target audience |
| --- | --- | --- |
| P0 | First VHS LSP — upgrade from plain-text editing to IDE-level experience | All VHS users |
| P0 | One-click preview — CodeLens + side-by-side Preview, no more running `vhs < file.tape` manually | Daily VHS users |
| P1 | Safety checks — warns about dangerous commands in `Type` instructions | DevOps / CI scenarios |
| P1 | AI three-role engineering — Scout/Architect/Builder separation, spec-first workflow | Rust / AI engineering community |
| P2 | Rust + rowan lossless syntax tree implementation | Rust developers |
| P2 | Cross-platform VSIX packaging + no-server fallback | Extension developers |

## Channels

### Wave 1: Charmbracelet Community (Highest ROI)

The core target audience lives here. Reach them first.

1. **Charmbracelet Discord / Slack** — Post a short introduction with a Demo GIF in the showcase channel. Include the Marketplace link.
2. **VHS GitHub Discussions** — Open a Discussion post. Reference the issue #162 conversation. Emphasize "complementary to vhs, not competing."
3. **Contact VHS maintainers** — Message `caarlos0` or `maaslalani`. Goal: get a mention in the VHS README's community tools section.

### Wave 2: Hacker News + Reddit (Largest Reach)

1. **Hacker News — Show HN** — Suggested title: "Show HN: VHS Analyzer – A Rust LSP and VS Code extension for terminal recording scripts." Post on Tuesday or Wednesday morning (US Eastern). Be ready to answer technical questions (rowan, tower-lsp-server choices).
2. **Reddit** — Use different angles for different subreddits:
   - `r/rust` — Rust implementation details
   - `r/vscode` — Extension user experience
   - `r/commandline` — Terminal recording workflow improvement
   - `r/programming` — Filling a gap in the LSP ecosystem

### Wave 3: Chinese Tech Communities

1. **V2EX** — "分享创造" (Share & Create) section
2. **Juejin** — Technical article (engineering methodology angle)
3. **Zhihu** — Reference in answers to terminal recording or Rust toolchain questions

### Wave 4: Twitter/X + Long Tail

1. **Twitter/X** — Post with GIF, @charmbracelet, hashtags `#rust #vscode #devtools #terminal`.
2. **Technical blog posts** — Publish on dev.to / Medium / personal blog. Two independent topics:
    - Product line: "Why VHS needs a Language Server"
    - Methodology line: "Building a complete LSP + extension with AI three-role engineering"

## Content to Prepare Before Launch

### P0: Must Complete Before Launch

| # | Content | Current state | Notes |
| --- | --- | --- | --- |
| 1 | Demo GIF / animation | **Missing** | The most critical gap. Record a GIF showing hover / completion / diagnostics / formatting / CodeLens / Preview in action. Using VHS itself to record the demo creates a meta loop that is itself good marketing. |
| 2 | Embed Demo GIF in README | **Missing** | Both root README and extension README are plain text. Projects without visual demos rarely get attention. |
| 3 | Feature screenshot set | **Missing** | At least 5-6 screenshots: hover, completion, diagnostics, formatting before/after, CodeLens, Preview panel. |
| 4 | GitHub topics | Not set | Add: `vhs`, `tape`, `lsp`, `language-server`, `vscode-extension`, `rust`, `terminal`, `gif`, `devtools`. |

### P1: Strongly Recommended

| # | Content | Current state | Notes |
| --- | --- | --- | --- |
| 5 | `examples/` directory | **Does not exist** | Create 3-5 typical `.tape` example files covering simple, complex, and safety-warning scenarios. |
| 6 | Issue / PR templates | **Do not exist** | Bug Report / Feature Request / PR templates lower the barrier for community participation. |
| 7 | GitHub Release Notes polish | Could be richer | Add feature highlights and screenshots to the v0.1.1 release. |
| 8 | 30-60 second demo video | **Does not exist** | More complete than a GIF. Upload to YouTube / Bilibili. Embed the link in the README. |

### P2: Can Defer but Worth Doing

| # | Content | Notes |
| --- | --- | --- |
| 9 | Technical blog post drafts | One per narrative line |
| 10 | Social media cover image | 16:9 image suitable for Twitter/X |
| 11 | Marketplace page review | Confirm the extension displays well on the store page |
| 12 | Getting Started guide | Quick walkthrough from installation to the first tape file |

## Suggested Timeline

```text
Day 0-2   Preparation
          ├─ Record Demo GIF (use VHS to record vhs-analyzer in action)
          ├─ Capture feature screenshots
          ├─ Update README (embed GIF + screenshots)
          ├─ Create examples/ directory
          ├─ Set up GitHub topics, Issue/PR templates
          └─ Polish GitHub Release Notes

Day 3     Wave 1 — Charmbracelet community
          ├─ Discord / Slack post
          └─ VHS GitHub Discussion

Day 4-5   Wave 2 — English tech communities
          ├─ Show HN (Tuesday or Wednesday morning US Eastern)
          ├─ Reddit r/rust + r/vscode + r/commandline
          └─ Twitter/X @charmbracelet

Day 5-7   Wave 3 — Chinese communities
          ├─ V2EX "Share & Create"
          ├─ Juejin article
          └─ Other Chinese platforms

Day 7+    Wave 4 — Long tail
          ├─ Technical blog posts
          ├─ Respond to issues and discussions
          └─ Iterate based on feedback
```

## Dual Narrative Strategy

The project has a unique selling point that most open-source tools lack: a fully documented AI three-role development process (`prompt/`, `trace/`, `docs/agentic-workflow.md`).

Use two separate narrative lines for different audiences on different channels:

- **Product line** (for VHS users): "Your `.tape` files finally have real IDE support"
- **Methodology line** (for Rust / AI engineers): "How I built a complete LSP + extension using AI three-role engineering"

## Key Takeaway

**The biggest barrier to promotion is the lack of visual demo materials.** The README has no GIFs or screenshots, which in the open-source world is almost equivalent to being invisible. A single 30-second Demo GIF at the top of the README can increase page dwell time and star conversion by an order of magnitude.

**First priority: record a Demo GIF and update the README.**
