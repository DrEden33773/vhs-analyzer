# SPEC_HOVER.md — Hover Documentation Provider

**Phase:** 1 — LSP Foundation
**Work Stream:** WS-4 (Hover)
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect
**Depends On:** WS-2 (SPEC_PARSER.md), WS-3 (SPEC_LSP_CORE.md)
**Last Updated:** 2026-03-18
**Frozen By:** Architect (Claude) — Stage B

---

> **CONTRACT_FROZEN** — This specification is frozen as of 2026-03-18.
> All Freeze Candidates have been resolved. No changes without explicit user approval.

---

## 1. Purpose

Define the hover documentation provider that maps AST node positions to
human-readable documentation strings. When a user hovers over a VHS keyword,
setting name, or command in their editor, the LSP returns a Markdown-formatted
tooltip with a description, syntax, and examples sourced from the official
VHS documentation.

## 2. References

| Source | Role |
| --- | --- |
| [VHS README](https://github.com/charmbracelet/vhs?tab=readme-ov-file) | Authoritative documentation for hover content |
| [LSP Hover Specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover) | Protocol contract |
| SPEC_PARSER.md | AST node kinds used for hover resolution |

## 3. Requirements

### HOV-001 — Hover Response Format

| Field | Value |
| --- | --- |
| **ID** | HOV-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `textDocument/hover` handler MUST return `Hover { contents: MarkupContent { kind: Markdown, value }, range }`. The `range` MUST span the token or node that the hover documentation applies to. If no hover information is available at the cursor position, the handler MUST return `Ok(None)`. |
| **Verification** | Send hover request at a known keyword position; verify Markdown content and range. Send hover request at whitespace; verify `null` response. |

### HOV-002 — Command Keyword Hover

| Field | Value |
| --- | --- |
| **ID** | HOV-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Hovering over a VHS command keyword (e.g., `Type`, `Sleep`, `Output`, `Set`, `Hide`, `Show`, `Ctrl`, etc.) MUST return documentation including: (1) a brief description, (2) syntax signature, (3) at least one example. Content MUST be sourced from the VHS README command reference. |
| **Verification** | Hover on `Type` returns description mentioning "emulate key presses", syntax `Type[@<time>] "<text>"`, and an example. |

### HOV-003 — Setting Name Hover

| Field | Value |
| --- | --- |
| **ID** | HOV-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Hovering over a setting name keyword within a `SET_COMMAND` (e.g., `FontSize`, `Theme`, `Shell`) MUST return documentation including: (1) description of the setting, (2) value type and constraints, (3) an example. |
| **Verification** | Hover on `FontSize` in `Set FontSize 14` returns description, type (`float`), and example. |

### HOV-004 — Modifier Key Hover

| Field | Value |
| --- | --- |
| **ID** | HOV-004 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | Hovering over modifier keywords (`Ctrl`, `Alt`, `Shift`) SHOULD return documentation explaining the modifier combination syntax and available key targets. |
| **Verification** | Hover on `Ctrl` in `Ctrl+C` returns modifier documentation. |

### HOV-005 — Literal Value Hover

| Field | Value |
| --- | --- |
| **ID** | HOV-005 |
| **Priority** | P2 (MAY) |
| **Owner** | Architect → Builder |
| **Statement** | Hovering over a literal value (e.g., a time value `500ms`, a boolean `true`) MAY return a tooltip describing the value type and its role in the current command context. |
| **Verification** | Hover on `500ms` in `Sleep 500ms` returns "Duration: 500 milliseconds". |

### HOV-006 — Hover Resolution Algorithm

| Field | Value |
| --- | --- |
| **ID** | HOV-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The hover resolution algorithm MUST: (1) Find the token at the cursor offset using `SyntaxNode::token_at_offset()`. (2) Walk ancestors to find the innermost meaningful AST node. (3) Look up documentation from a static registry keyed by `SyntaxKind`. (4) Return the documentation with the token/node range. |
| **Verification** | Hover at various positions within a command returns correct, contextual documentation. |

## 4. Design Options Analysis

### 4.1 Documentation Storage

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Static `phf` map** | Compile-time perfect hash map `SyntaxKind → &str` | Zero runtime cost; no allocation | Requires `phf` crate; less flexible for dynamic content |
| **B: `match` expression** | `match kind { TYPE_KW => "...", ... }` in hover handler | Zero deps; simple; compiler-optimized | Large match block; harder to maintain |
| **C: Embedded TOML/JSON** | Load documentation from embedded file at build time via `include_str!` | Separates content from code; easy to edit | Parse overhead (negligible if done once); extra format to maintain |

**Recommended: Option B (`match` expression) for Phase 1.** The VHS command
set is finite and small (~25 commands + ~19 settings = ~44 entries). A `match`
expression is the simplest, zero-dependency approach. If the documentation
grows significantly in Phase 2 (e.g., context-sensitive docs), upgrade to
Option C.

### 4.2 Hover Context Sensitivity

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Token-only** | Hover resolves based solely on the `SyntaxKind` of the token under the cursor | Simple; covers most cases | `Enter` keyword has different docs as a command vs. as a `Ctrl+Enter` target |
| **B: Token + parent node** | Hover inspects both the token kind and its parent node kind for context | Context-sensitive; more accurate docs | Slightly more complex resolution logic |

**Recommended: Option B (Token + parent node).** The additional complexity is
minimal (one ancestor walk) and enables accurate context-sensitive documentation.
For example:

- `Enter` at line start (parent: `KEY_COMMAND`) → "Press the Enter key"
- `Enter` after `Ctrl+` (parent: `CTRL_COMMAND`) → "Target key for Ctrl combination"

### 4.3 Markdown Content Richness

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Plain text** | Simple text descriptions, no formatting | Universal compatibility | Boring; poor readability |
| **B: Basic Markdown** | Headers, code blocks, bold | Good readability; supported by all major editors | Slightly more content to author |
| **C: Rich Markdown** | Tables, links to VHS docs, images | Maximum information | Some clients may not render all features |

**Recommended: Option B (Basic Markdown).** Use headers for command name,
fenced code blocks for syntax and examples, bold for emphasis. Avoid tables
and links in Phase 1 (some LSP clients render them poorly).

## 5. Hover Content Registry — Complete Mapping

### 5.1 Command Keywords

| Token Kind | Hover Title | Brief Description |
| --- | --- | --- |
| `OUTPUT_KW` | **Output** | Specify the output file path and format (.gif, .mp4, .webm) |
| `SET_KW` | **Set** | Configure terminal appearance and behavior |
| `ENV_KW` | **Env** | Set an environment variable (key-value pair) |
| `SLEEP_KW` | **Sleep** | Pause recording for a specified duration |
| `TYPE_KW` | **Type** | Emulate typing text into the terminal |
| `BACKSPACE_KW` | **Backspace** | Press the Backspace key |
| `DOWN_KW` | **Down** | Press the Down arrow key |
| `ENTER_KW` | **Enter** | Press the Enter key |
| `ESCAPE_KW` | **Escape** | Press the Escape key |
| `LEFT_KW` | **Left** | Press the Left arrow key |
| `RIGHT_KW` | **Right** | Press the Right arrow key |
| `SPACE_KW` | **Space** | Press the Space bar |
| `TAB_KW` | **Tab** | Press the Tab key |
| `UP_KW` | **Up** | Press the Up arrow key |
| `PAGEUP_KW` | **PageUp** | Press the Page Up key |
| `PAGEDOWN_KW` | **PageDown** | Press the Page Down key |
| `SCROLLUP_KW` | **ScrollUp** | Scroll the terminal viewport up |
| `SCROLLDOWN_KW` | **ScrollDown** | Scroll the terminal viewport down |
| `WAIT_KW` | **Wait** | Wait for a regex pattern to appear on screen |
| `REQUIRE_KW` | **Require** | Assert a program exists in $PATH before execution |
| `SOURCE_KW` | **Source** | Include and execute commands from another tape file |
| `HIDE_KW` | **Hide** | Stop capturing frames (hide subsequent commands from output) |
| `SHOW_KW` | **Show** | Resume capturing frames |
| `COPY_KW` | **Copy** | Copy text to clipboard |
| `PASTE_KW` | **Paste** | Paste text from clipboard |
| `SCREENSHOT_KW` | **Screenshot** | Capture current frame as a PNG screenshot |
| `CTRL_KW` | **Ctrl** | Control modifier key combination |
| `ALT_KW` | **Alt** | Alt modifier key combination |
| `SHIFT_KW` | **Shift** | Shift modifier key combination |

### 5.2 Setting Name Keywords

| Token Kind | Hover Title | Value Type | Brief Description |
| --- | --- | --- | --- |
| `SHELL_KW` | **Shell** | string | Set the shell program (e.g., `bash`, `zsh`, `fish`) |
| `FONTFAMILY_KW` | **FontFamily** | string | Set the terminal font family |
| `FONTSIZE_KW` | **FontSize** | float | Set the font size in pixels |
| `FRAMERATE_KW` | **Framerate** | integer | Set the recording frame rate (fps) |
| `PLAYBACKSPEED_KW` | **PlaybackSpeed** | float | Set playback speed multiplier (1.0 = normal) |
| `HEIGHT_KW` | **Height** | integer | Set terminal height in pixels |
| `WIDTH_KW` | **Width** | integer | Set terminal width in pixels |
| `LETTERSPACING_KW` | **LetterSpacing** | float | Set letter spacing (tracking) |
| `TYPINGSPEED_KW` | **TypingSpeed** | time | Set default typing speed per character |
| `LINEHEIGHT_KW` | **LineHeight** | float | Set line height multiplier |
| `PADDING_KW` | **Padding** | float | Set terminal frame padding in pixels |
| `THEME_KW` | **Theme** | string/identifier/JSON | Set color theme by name or JSON definition |
| `LOOPOFFSET_KW` | **LoopOffset** | float[%] | Set GIF loop start frame offset |
| `BORDERRADIUS_KW` | **BorderRadius** | integer | Set terminal window border radius in pixels |
| `MARGIN_KW` | **Margin** | integer | Set video margin in pixels |
| `MARGINFILL_KW` | **MarginFill** | string | Set margin fill color (hex) or image file |
| `WINDOWBAR_KW` | **WindowBar** | string | Set window bar style (Colorful, Rings, etc.) |
| `WINDOWBARSIZE_KW` | **WindowBarSize** | integer | Set window bar size in pixels |
| `CURSORBLINK_KW` | **CursorBlink** | boolean | Enable or disable cursor blinking |

## 6. Example Hover Content

For `Type` keyword, the hover should produce:

```markdown
**Type**

Emulate typing text into the terminal.

**Syntax:**
    ```tape

    Type[@<speed>] "<text>"

    ```

**Example:**
    ```tape
    Type "echo 'Hello, World!'"
    Type@500ms "Slow typing"
    ```

Override typing speed per-command with `@<duration>`.

```

For `FontSize` setting keyword:

```markdown
**Set FontSize**

Set the font size for the terminal in pixels.

**Syntax:**
    ```tape

    Set FontSize <number>

    ```

**Example:**
    ```tape
    Set FontSize 14
    Set FontSize 46
    ```

```

## 7. Resolved Design Decisions

All Freeze Candidates from Stage A have been closed with definitive decisions.

### FC-HOV-01 — Hover Documentation Storage (RESOLVED: Embedded match expression)

**Decision:** Hover documentation MUST be embedded in Rust source code as a
`match` expression on `SyntaxKind`, returning `&'static str` Markdown content.

**Rationale:** The VHS command set is small and stable (~27 commands + ~19
settings = ~46 entries). A `match` expression is:
(1) zero-dependency — no TOML/JSON parser needed,
(2) compiler-checked — a missing arm causes a compile error,
(3) zero-allocation — returns `&'static str` references,
(4) inline-friendly — the Rust compiler can optimize the match into a jump
table. Per Rust Best Practices (Apollo handbook, Chapter 3): avoid unnecessary
heap allocation; prefer static data for fixed-size lookup tables.
Phase 2 MAY migrate to `include_str!()` with an embedded TOML file if the
documentation grows significantly (e.g., context-sensitive multi-paragraph docs).

### FC-HOV-02 — Links in Hover Content (RESOLVED: MUST NOT include in Phase 1)

**Decision:** Hover content MUST NOT include clickable links to the VHS
documentation website in Phase 1.

**Rationale:** LSP hover rendering varies across editors. VSCode renders
Markdown links well, but Neovim's built-in LSP client, Helix, and other
editors may render `[text](url)` as raw text or strip links entirely. Including
non-functional links degrades the experience for non-VSCode users. Phase 2
MAY add links as a configurable option, guarded by a client capability check
(`general.markdown.allowedTags` in LSP 3.17).

### FC-HOV-03 — Repeatable Key Hover Content (RESOLVED: Template + unique descriptions)

**Decision:** Repeatable key commands (Backspace, Enter, Down, etc.) MUST use
a shared template for syntax and examples, with per-key unique brief
descriptions.

**Template structure:**

```text
**{KeyName}**

{UniqueDescription}

**Syntax:**
{KeyName}[@<speed>] [<count>]

**Example:**
{KeyName} 5
{KeyName}@100ms 3
```

**Per-key unique descriptions (examples):**

- `Backspace` → "Delete the character before the cursor"
- `Enter` → "Press the Enter/Return key"
- `Down` → "Press the Down arrow key"
- `ScrollUp` → "Scroll the terminal viewport up by rows"

**Rationale:** All 13 repeatable key commands share identical syntax
(`Key[@time] [count]`). A template avoids maintaining 13 nearly-identical
Markdown strings. The per-key unique description provides meaningful
differentiation. The Builder SHOULD implement this as a helper function
`fn key_hover(name: &str, description: &str) -> String` that fills the
template.
