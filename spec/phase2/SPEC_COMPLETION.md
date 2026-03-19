# SPEC_COMPLETION.md — Context-Aware Autocomplete

**Phase:** 2 — Intelligence & Diagnostics
**Work Stream:** WS-1 (Completion)
**Status:** Stage A (Exploratory Design)
**Owner:** Architect
**Depends On:** phase1/SPEC_PARSER.md (AST), phase1/SPEC_LSP_CORE.md (server capabilities)
**Last Updated:** 2026-03-19

---

## 1. Purpose

Define the context-aware autocomplete provider for VHS `.tape` files. The
completion provider resolves the cursor position against the AST to determine
what category of completion to offer, then returns an appropriate list of
`CompletionItem` entries. This feature extends the Phase 1 server capabilities
(SPEC_LSP_CORE.md §5) by adding `completionProvider` to `InitializeResult`.

## 2. Cross-Phase Dependencies

| Phase 1 Contract | Usage in This Spec |
| --- | --- |
| SPEC_PARSER.md — PAR-001 (SyntaxKind enum) | Completion context resolution inspects `SyntaxKind` of tokens and nodes at cursor position |
| SPEC_PARSER.md — PAR-007 (Typed AST accessors) | `SetCommand`, `TypeCommand`, etc. provide structured access to command children |
| SPEC_PARSER.md — §4 (Node Kind Enumeration) | Node kinds (`SET_COMMAND`, `TYPE_COMMAND`, etc.) determine completion category |
| SPEC_PARSER.md — §5 (Ungrammar) | Tree shape defines expected children per command, guiding position-aware completion |
| SPEC_LSP_CORE.md — LSP-002 (Initialize) | `completionProvider` capability added to `InitializeResult` |
| SPEC_LSP_CORE.md — LSP-004 (Document State) | Completion handler reads parsed AST from `DashMap<Url, DocumentState>` |
| SPEC_HOVER.md — HOV-006 (Hover resolution) | The cursor-position → AST-context resolution pattern is reused for completion context |

## 3. References

| Source | Role |
| --- | --- |
| [LSP 3.17 — textDocument/completion](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion) | Protocol contract for completion requests and responses |
| [VHS THEMES.md](https://github.com/charmbracelet/vhs/blob/main/THEMES.md) | Authoritative list of built-in theme names (300+ entries) |
| [VHS README](https://github.com/charmbracelet/vhs?tab=readme-ov-file) | Directive semantics, setting value types, command syntax |
| [`tower-lsp-server` v0.23 — `completion()`](https://docs.rs/tower-lsp-server/latest/tower_lsp_server/trait.LanguageServer.html) | Handler signature: `fn completion(&self, params: CompletionParams) -> ... Result<Option<CompletionResponse>>` |
| [CompletionOptions (lsp-types)](https://docs.rs/lsp-types/latest/lsp_types/struct.CompletionOptions.html) | Rust types for `triggerCharacters`, `resolveProvider` |

## 4. Requirements

### CMP-001 — completionProvider Capability

| Field | Value |
| --- | --- |
| **ID** | CMP-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `InitializeResult` MUST advertise a `completionProvider` capability with: (1) `triggerCharacters: []` (empty — rely on client word-boundary triggers and manual invocation), (2) `resolveProvider: false` (all completion data is returned eagerly). The `CompletionOptions` MUST NOT include `workDoneProgress`. |
| **Verification** | Send `initialize`; verify response contains `completionProvider` with empty `triggerCharacters` and `resolveProvider: false`. |

### CMP-002 — Completion Context Resolution Algorithm

| Field | Value |
| --- | --- |
| **ID** | CMP-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The completion handler MUST implement a context resolution algorithm that: (1) Finds the token at or immediately before the cursor offset using `token_at_offset()`. (2) Walks ancestors to find the innermost command node. (3) Determines the completion category based on the command node kind and the cursor's position relative to the command's children (see §6 Completion Context Matrix). (4) Returns `CompletionItem` entries for the resolved category. If no meaningful context is found, the handler MUST return `Ok(None)`. |
| **Verification** | Request completion at various cursor positions; verify correct category items are returned per the §6 matrix. |

### CMP-003 — Command Keyword Completions

| Field | Value |
| --- | --- |
| **ID** | CMP-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | When the cursor is at line start (no enclosing command node, or within an ERROR node at line start), the completion handler MUST return all VHS command keywords as CompletionItems: `Output`, `Set`, `Env`, `Sleep`, `Type`, `Backspace`, `Down`, `Enter`, `Escape`, `Left`, `Right`, `Space`, `Tab`, `Up`, `PageUp`, `PageDown`, `ScrollUp`, `ScrollDown`, `Hide`, `Show`, `Copy`, `Paste`, `Screenshot`, `Wait`, `Require`, `Source`, `Ctrl`, `Alt`, `Shift`. Each item MUST have `kind: CompletionItemKind::Keyword` and a brief `detail` description. |
| **Verification** | Request completion at column 0 of an empty line; verify all command keywords are returned. |

### CMP-004 — Setting Name Completions

| Field | Value |
| --- | --- |
| **ID** | CMP-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | When the cursor is inside a `SET_COMMAND` after the `SET_KW` token and before a setting name keyword, the completion handler MUST return all setting name keywords: `Shell`, `FontFamily`, `FontSize`, `Framerate`, `PlaybackSpeed`, `Height`, `Width`, `LetterSpacing`, `TypingSpeed`, `LineHeight`, `Padding`, `Theme`, `LoopOffset`, `BorderRadius`, `Margin`, `MarginFill`, `WindowBar`, `WindowBarSize`, `CursorBlink`. Each item MUST have `kind: CompletionItemKind::Property` and `detail` indicating the expected value type. |
| **Verification** | Type `Set` followed by a space and request completion; verify all 19 setting names are returned with correct value type details. |

### CMP-005 — Theme Name Completions

| Field | Value |
| --- | --- |
| **ID** | CMP-005 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | When the cursor is inside a `SET_COMMAND` whose setting is `Theme` and positioned after the `THEME_KW` token, the completion handler MUST return theme names from the built-in theme registry (§8). Each item MUST have `kind: CompletionItemKind::EnumMember`, `detail: "VHS built-in theme"`, and `insertText` wrapping the theme name in double quotes if the name contains spaces (e.g., `"Catppuccin Mocha"`). |
| **Verification** | Type `Set Theme` followed by a space and request completion; verify the list includes `Dracula`, `Catppuccin Mocha`, `Nord`, and at least 300 entries total. |

### CMP-006 — Setting Value Completions

| Field | Value |
| --- | --- |
| **ID** | CMP-006 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | For settings with enumerable value sets, the completion handler SHOULD return typed value completions when the cursor is in the value position of a `SET_COMMAND`. Specifically: (1) `CursorBlink` → `true`, `false` with `kind: Value`. (2) `WindowBar` → `"Colorful"`, `"ColorfulRight"`, `"Rings"`, `"RingsRight"` with `kind: EnumMember`. (3) `Shell` → common shells (`"bash"`, `"zsh"`, `"fish"`, `"sh"`, `"powershell"`, `"pwsh"`) with `kind: Value`. |
| **Verification** | Type `Set CursorBlink` then space → returns `true`/`false`. Type `Set WindowBar` then space → returns the four window bar styles. |

### CMP-007 — Snippet Templates

| Field | Value |
| --- | --- |
| **ID** | CMP-007 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | Command keyword completions (CMP-003) SHOULD include snippet variants with `insertTextFormat: Snippet` for commands that take arguments. The snippet template SHOULD use tab stops for argument placeholders per LSP snippet syntax. See §9 for the snippet template registry. Snippet items MUST have `kind: CompletionItemKind::Snippet`. |
| **Verification** | Select `Type` snippet completion; verify the editor inserts `Type "${1:text}"` with cursor positioned inside the string. |

### CMP-008 — Output Extension Completions

| Field | Value |
| --- | --- |
| **ID** | CMP-008 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | When the cursor is inside an `OUTPUT_COMMAND` after the `OUTPUT_KW` token, the completion handler SHOULD return file extension completions: `.gif`, `.mp4`, `.webm`. Each item SHOULD have `kind: CompletionItemKind::File` and `detail` describing the output format. |
| **Verification** | Type `Output demo` and request completion; verify `.gif`, `.mp4`, `.webm` suffix completions are offered. |

### CMP-009 — Time Unit Completions

| Field | Value |
| --- | --- |
| **ID** | CMP-009 |
| **Priority** | P2 (MAY) |
| **Owner** | Architect → Builder |
| **Statement** | When the cursor follows a numeric literal in a time-accepting context (e.g., `Sleep`, `TypingSpeed`, duration override `@`), the completion handler MAY return time unit suffixes: `ms` (milliseconds), `s` (seconds). |
| **Verification** | Type `Sleep 500` and request completion; verify `ms` and `s` suffixes are offered. |

### CMP-010 — Modifier Key Target Completions

| Field | Value |
| --- | --- |
| **ID** | CMP-010 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | When the cursor is inside a `CTRL_COMMAND`, `ALT_COMMAND`, or `SHIFT_COMMAND` after the final `PLUS` token, the completion handler SHOULD return valid key targets. The target set includes: (1) single letters `A`–`Z`, (2) special keys `Enter`, `Tab`, `Backspace`, `Escape`, `Up`, `Down`, `Left`, `Right`, `Space`. Each item SHOULD have `kind: CompletionItemKind::EnumMember`. |
| **Verification** | Type `Ctrl+` and request completion; verify letters A–Z and special keys are returned. |

## 5. Design Options Analysis

### 5.1 Trigger Characters Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: No trigger characters** | `triggerCharacters: []`; rely on client word-boundary automatic triggers and manual Ctrl+Space | Zero noise; simplest server logic; works universally | Requires manual trigger in some edge cases (e.g., after typing `Set Theme`) |
| **B: Space trigger** | `triggerCharacters: [" "]` | Auto-triggers after keyword → argument transitions | Extremely noisy; triggers on every space in the document including inside strings |
| **C: Limited triggers** | `triggerCharacters: ["+"]` | Auto-triggers modifier key target completions | Incomplete; only helps one context; `+` in strings would false-trigger |

**Recommended: Option A (No trigger characters).** VHS is line-oriented
with simple command structures. Most editors (VSCode, Neovim, Helix) already
auto-trigger completion on word boundaries. Manual Ctrl+Space covers edge
cases. Option B is unacceptably noisy — every space character in the
document (including inside `Type "text with spaces"`) would trigger a
completion request. Per Rust Best Practices: prefer simplicity when the
domain is small.

### 5.2 Theme Registry Storage

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Compile-time static array** | `&[&str]` array of theme names embedded in Rust source via `include_str!` + `const` parsing, or a hand-written array | Zero runtime allocation; instant lookup | 300+ strings in source; updating requires rebuild |
| **B: Embedded TOML/JSON** | Theme names loaded from an embedded data file at build time via `include_str!` | Separates data from logic; easy to update | Requires parsing at startup (negligible for 300 entries) |
| **C: Runtime fetch** | Download theme list from VHS repository at startup | Always up-to-date | Network dependency; startup latency; offline breakage |

**Recommended: Option A (Compile-time static array).** The VHS theme list
changes infrequently (last update was PR #377). A static `&[&str]` array
provides zero-allocation completion. The 300+ entries are approximately
8 KB of string data — trivial for a binary. Updating requires only
editing the array and recompiling. Per Rust Best Practices (Chapter 3):
prefer stack/static allocation for fixed-size data.

### 5.3 Completion Response Format

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Eager (CompletionList)** | Return all items in a single `CompletionList` response; `resolveProvider: false` | Simple; one round-trip; client handles filtering | May be slightly larger payload for theme list |
| **B: Lazy resolve** | Return minimal items; `resolveProvider: true`; client calls `completionItem/resolve` for details | Smaller initial payload | Extra round-trips per item; more complex server logic |
| **C: Partial lists with `isIncomplete`** | Return `isIncomplete: true` for large lists; client re-requests as user types | Progressive filtering | Complex state management; multiple round-trips |

**Recommended: Option A (Eager CompletionList).** The total completion
payload (300 theme names + 30 keywords + 20 settings) is approximately
15 KB of JSON — well within acceptable bounds. Lazy resolve adds
complexity without meaningful benefit for a dataset this small. Per the
LSP specification: `resolveProvider` is intended for expensive per-item
computation (e.g., fetching documentation from a database), which is not
needed here since all data is static and cheap.

## 6. Completion Context Matrix

The context resolution algorithm maps cursor position to completion category:

| Cursor Position | Enclosing Node | Completion Category | Items |
| --- | --- | --- | --- |
| Line start / empty line / inside ERROR at line start | None or SOURCE_FILE | Command keywords | CMP-003 |
| After `Set` keyword, before setting name | SET_COMMAND | Setting names | CMP-004 |
| After `Set Theme`, in value position | SET_COMMAND (Theme) | Theme names | CMP-005 |
| After `Set CursorBlink`, in value position | SET_COMMAND (CursorBlink) | Boolean values | CMP-006 |
| After `Set WindowBar`, in value position | SET_COMMAND (WindowBar) | Window bar styles | CMP-006 |
| After `Set Shell`, in value position | SET_COMMAND (Shell) | Shell names | CMP-006 |
| After `Output`, in path position | OUTPUT_COMMAND | File extensions | CMP-008 |
| After `Ctrl+` / `Alt+` / `Shift+` | CTRL/ALT/SHIFT_COMMAND | Key targets | CMP-010 |
| After numeric in time context | SLEEP_COMMAND, DURATION | Time units | CMP-009 |
| Inside `Type` string argument | TYPE_COMMAND | None (free text) | No completions |
| Inside comment | COMMENT token | None | No completions |

## 7. Completion Context Resolution Algorithm (Pseudocode)

```text
fn resolve_completion_context(root: &SyntaxNode, offset: TextSize) -> CompletionContext:
    token = find_token_at_offset(root, offset)
    if token is None:
        return CommandKeywords

    // Walk ancestors to find enclosing command node
    node = token.parent()
    while node is not None and not is_command_node(node):
        node = node.parent()

    if node is None:
        // At top level — offer command keywords
        return CommandKeywords

    match node.kind():
        SET_COMMAND:
            setting_kw = find_setting_keyword_child(node)
            if setting_kw is None or offset <= setting_kw.text_range().start():
                return SettingNames
            match setting_kw.kind():
                THEME_KW    -> return ThemeNames
                CURSORBLINK_KW -> return BooleanValues
                WINDOWBAR_KW -> return WindowBarStyles
                SHELL_KW    -> return ShellNames
                _           -> return NoCompletion

        OUTPUT_COMMAND | SCREENSHOT_COMMAND:
            return FileExtensions

        CTRL_COMMAND | ALT_COMMAND | SHIFT_COMMAND:
            if cursor_is_after_last_plus(node, offset):
                return KeyTargets
            return NoCompletion

        SLEEP_COMMAND:
            return TimeUnits

        TYPE_COMMAND:
            return NoCompletion  // free text

        _:
            return NoCompletion
```

## 8. Built-In Theme Name Registry

The theme registry MUST be derived from the authoritative
[VHS THEMES.md](https://github.com/charmbracelet/vhs/blob/main/THEMES.md)
file. As of 2026-03-19, this file contains **318 theme names**.

Representative entries (for verification, not exhaustive):

```text
3024 Day, 3024 Night, Afterglow, Andromeda, Atom, Aurora,
Builtin Dark, Builtin Light, Builtin Pastel Dark,
Catppuccin Frappe, Catppuccin Latte, Catppuccin Macchiato, Catppuccin Mocha,
Dracula, Dracula+, GruvboxDark, GruvboxDarkHard, Gruvbox Light,
Monokai Pro, Monokai Vivid, Nord, nord,
OneDark, OneHalfDark, OneHalfLight,
Rose Pine, rose-pine, rose-pine-dawn, rose-pine-moon,
Solarized Darcula, Solarized Dark - Patched,
TokyoNight, tokyonight, tokyonight-day, tokyonight-storm,
Tomorrow Night, Ubuntu, Zenburn
```

**Implementation note:** Theme names containing spaces (e.g.,
`"Catppuccin Mocha"`) MUST be wrapped in double quotes when inserted.
Theme names without spaces (e.g., `Dracula`) SHOULD be inserted without
quotes. The Builder MUST extract the full list from THEMES.md at
development time and embed it as a static `&[&str]` array.

## 9. Snippet Template Registry

| Command | Snippet Template | Tab Stops |
| --- | --- | --- |
| `Output` | `Output ${1:demo}.${2\|gif,mp4,webm\|}` | 1: filename, 2: extension choice |
| `Set FontSize` | `Set FontSize ${1:14}` | 1: size value |
| `Set Theme` | `Set Theme "${1:Catppuccin Mocha}"` | 1: theme name |
| `Set Shell` | `Set Shell "${1:bash}"` | 1: shell name |
| `Set TypingSpeed` | `Set TypingSpeed ${1:75ms}` | 1: speed value |
| `Type` | `Type "${1:text}"` | 1: typed text |
| `Type@speed` | `Type@${1:500ms} "${2:text}"` | 1: speed, 2: text |
| `Sleep` | `Sleep ${1:1s}` | 1: duration |
| `Env` | `Env ${1:KEY} ${2:VALUE}` | 1: key, 2: value |
| `Require` | `Require ${1:program}` | 1: program name |
| `Source` | `Source "${1:file.tape}"` | 1: file path |
| `Screenshot` | `Screenshot ${1:screenshot.png}` | 1: file path |
| `Wait` | `Wait ${1:/regex/}` | 1: regex pattern |

## 10. Phase 2 Server Capabilities (Extension of LSP-002)

Phase 2 extends the Phase 1 `InitializeResult` capabilities:

```json
{
  "capabilities": {
    "textDocumentSync": {
      "openClose": true,
      "change": 1,
      "save": { "includeText": false }
    },
    "hoverProvider": true,
    "documentFormattingProvider": true,
    "completionProvider": {
      "triggerCharacters": [],
      "resolveProvider": false
    }
  },
  "serverInfo": {
    "name": "vhs-analyzer",
    "version": "0.2.0"
  }
}
```

Changes from Phase 1:

- Added `completionProvider` with empty trigger characters.
- Added `textDocumentSync.save` to support `didSave` notifications
  (consumed by SPEC_DIAGNOSTICS.md heavyweight checks).
- Version bumped to `0.2.0`.

## 11. Freeze Candidates

### FC-CMP-01 — Trigger Characters Set

**Status:** Open

**Question:** Should the completion provider use empty trigger characters
(rely on client word-boundary triggers) or include specific characters
like `+` for modifier key contexts?

**Current recommendation:** Empty (`[]`). See §5.1 analysis.

**Resolution criteria:** Validate with VSCode and Neovim LSP clients
that word-boundary auto-trigger provides a satisfactory UX without
explicit trigger characters. If modifier key completion after `Ctrl+`
requires a trigger, add `"+"` as a single trigger character.

### FC-CMP-02 — WindowBar Style Enumeration

**Status:** Open

**Question:** The VHS README and source code reference WindowBar styles
`"Colorful"`, `"ColorfulRight"`, `"Rings"`, `"RingsRight"`. Is this the
complete list? Are there undocumented styles?

**Current recommendation:** Include the four known styles. The Builder
MUST verify against the VHS Go source code (`windowbar.go` or equivalent)
during implementation.

**Resolution criteria:** Cross-reference with VHS source code to produce
a definitive list.

### FC-CMP-03 — Theme Registry Update Strategy

**Status:** Open

**Question:** How should the built-in theme list be kept in sync with
upstream VHS releases? Options: (A) manual update with each VHS release,
(B) build script that fetches THEMES.md at compile time,
(C) accept staleness as a non-critical issue.

**Current recommendation:** Option A — manual update. The theme list
changes infrequently and the LSP binary is versioned independently.
Embedding a fixed list provides offline reliability.

**Resolution criteria:** Decide during Stage B freeze.
