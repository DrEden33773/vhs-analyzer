# SPEC_FORMATTING.md — Document Formatting Provider

**Phase:** 1 — LSP Foundation
**Work Stream:** WS-5 (Formatting)
**Status:** Stage A (Exploratory Design)
**Owner:** Architect
**Depends On:** WS-2 (SPEC_PARSER.md)
**Last Updated:** 2026-03-18

---

## 1. Purpose

Define the document formatting rules for VHS `.tape` files. The formatter
operates on the parsed AST (from WS-2) and produces a list of `TextEdit`
operations that transform the document into a canonically formatted form.
The formatting provider is invoked via `textDocument/formatting`.

## 2. Design Principles

VHS `.tape` files have an extremely flat structure — no nesting, no blocks,
no indentation hierarchy. Formatting focuses on:

1. **Consistent spacing** between command keywords and their arguments.
2. **Blank line normalization** for visual grouping.
3. **Comment alignment** where applicable.
4. **No indentation changes** — VHS commands are always at column 0.

The formatter MUST be idempotent: formatting an already-formatted file
produces zero edits.

## 3. Requirements

### FMT-001 — LSP Formatting Response

| Field | Value |
| --- | --- |
| **ID** | FMT-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `textDocument/formatting` handler MUST return `Vec<TextEdit>` representing the minimal set of edits to canonically format the document. If the document is already canonical, the handler MUST return an empty Vec. The handler MUST respect `FormattingOptions.tabSize` and `FormattingOptions.insertSpaces` from the client, though VHS files use spaces exclusively. |
| **Verification** | Format a messy file; verify edits produce canonical form. Format the result again; verify zero edits returned. |

### FMT-002 — No Leading Indentation

| Field | Value |
| --- | --- |
| **ID** | FMT-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | All commands MUST start at column 0 (no leading whitespace). The formatter MUST remove any leading spaces or tabs before command keywords. |
| **Verification** | Input `Type "hello"` → output `Type "hello"` (leading spaces removed). |

### FMT-003 — Single Space Between Tokens

| Field | Value |
| --- | --- |
| **ID** | FMT-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Within a command, exactly one space MUST separate the command keyword from its arguments, and exactly one space MUST separate each argument from the next. Multiple spaces or tabs MUST be collapsed to a single space. |
| **Verification** | Input `Set   FontSize   14` → output `Set FontSize 14`. |

### FMT-004 — No Space Around Punctuation in Modifiers

| Field | Value |
| --- | --- |
| **ID** | FMT-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | No spaces MUST appear around `+` in modifier key combinations (`Ctrl+Alt+Shift+A`). No space MUST appear between `@` and its time value in duration overrides (`Type@500ms`). |
| **Verification** | Input `Ctrl + C` → output `Ctrl+C`. Input `Type @ 50ms "text"` → output `Type@50ms "text"`. |

### FMT-005 — Blank Line Normalization

| Field | Value |
| --- | --- |
| **ID** | FMT-005 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | Consecutive blank lines (2+ empty lines) SHOULD be collapsed to a single blank line. A single blank line between logical groups of commands SHOULD be preserved. The formatter SHOULD NOT add blank lines that are not already present (it only normalizes excess blank lines). |
| **Verification** | Input with 3 blank lines between commands → output has exactly 1 blank line. Input with 1 blank line → output preserves it. |

### FMT-006 — Trailing Whitespace Removal

| Field | Value |
| --- | --- |
| **ID** | FMT-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Trailing whitespace (spaces and tabs) at the end of any line MUST be removed. |
| **Verification** | Input `Type "hello"   \n` → output `Type "hello"\n`. |

### FMT-007 — Final Newline

| Field | Value |
| --- | --- |
| **ID** | FMT-007 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | The formatted document SHOULD end with exactly one trailing newline. Multiple trailing newlines SHOULD be collapsed to one. A missing final newline SHOULD be added. |
| **Verification** | File ending with `...\n\n\n` → `...\n`. File ending with `...Type "x"` (no newline) → `...Type "x"\n`. |

### FMT-008 — Comment Preservation

| Field | Value |
| --- | --- |
| **ID** | FMT-008 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Comments MUST be preserved verbatim. The formatter MUST NOT modify comment content. Leading whitespace before a comment at line start MUST be removed (per FMT-002). Inline comments (if any) MUST be preserved with one space before the `#`. |
| **Verification** | `# This is a comment` → `# This is a comment` (unchanged). `# Indented comment` → `# Indented comment`. |

### FMT-009 — Error Tolerance

| Field | Value |
| --- | --- |
| **ID** | FMT-009 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The formatter MUST handle files containing parse errors. Lines with `ERROR` nodes MUST be preserved as-is (no formatting applied to error regions). Only well-parsed commands are subject to formatting rules. |
| **Verification** | A file with one invalid line and two valid lines: the valid lines are formatted, the invalid line is unchanged. |

## 4. Design Options Analysis

### 4.1 Formatting Implementation Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: AST-based rewrite** | Walk the AST, emit canonical text for each node, diff against original | Produces perfect canonical form; easy to reason about | Must handle trivia (comments, blank lines) carefully; may produce large diffs |
| **B: Token-stream transform** | Walk the token stream, adjust whitespace tokens only | Minimal edits; preserves structure; simple diff | Harder to enforce structural rules (e.g., command at column 0) |
| **C: Print + diff** | Pretty-print from AST to a new string, then compute minimal TextEdit diff | Clean separation of formatting and diff | Two full passes; diff computation adds complexity |

**Recommended: Option B (Token-stream transform).** VHS formatting rules are
entirely about whitespace adjustment — there is no reordering, no structural
changes. Walking the token stream and modifying only `WHITESPACE` and `NEWLINE`
tokens is the most efficient approach and naturally produces minimal `TextEdit`
operations. This approach also trivially satisfies FMT-009 (error tolerance)
because `ERROR` node tokens are simply passed through unchanged.

### 4.2 Formatting Scope

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Whole-document only** | `textDocument/formatting` formats the entire file | Simplest; sufficient for small VHS files | No range formatting support |
| **B: Range formatting** | Also implement `textDocument/rangeFormatting` | Users can format selections | Extra capability to advertise and implement |

**Recommended: Option A (Whole-document only) for Phase 1.** VHS files are
small. Whole-document formatting is sufficient. Range formatting MAY be added
later as a low-priority enhancement.

### 4.3 Canonical Form Definition

All formatting rules converge to this canonical form:

```tape
# Configuration section
Output demo.gif

Set FontSize 14
Set FontFamily "JetBrains Mono"
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set TypingSpeed 75ms

# Setup
Require git

Hide
Type "cd ~/project"
Enter
Sleep 500ms
Show

# Recording
Type "git status"
Sleep 500ms
Enter
Sleep 2s

Type "git add ."
Sleep 500ms
Enter
Sleep 1s
```

Properties of canonical form:

- All commands at column 0.
- One space between keyword and arguments.
- No trailing whitespace.
- Single blank lines between logical sections (user-placed).
- No consecutive blank lines.
- File ends with exactly one newline.
- `Ctrl+C` has no spaces around `+`.
- `Type@50ms` has no space around `@`.
- Comments at column 0 (leading whitespace stripped).

## 5. Formatter API Contract

```rust
pub fn format(
    tree: &SyntaxNode,
    options: &FormattingOptions,
) -> Vec<TextEdit>
```

The function takes the root `SyntaxNode` and client formatting options,
returning a (possibly empty) list of `TextEdit` operations. Each `TextEdit`
specifies a range in the original document and a replacement string.

The formatter operates in a single pass over the syntax tree's token stream
(`SyntaxNode::descendants_with_tokens()`) and builds edits by comparing
actual whitespace against expected whitespace at each position.

## 6. Freeze Candidates

| ID | Item | Options Under Consideration |
| --- | --- | --- |
| **FC-FMT-01** | Should the formatter enforce a specific ordering of directives (e.g., `Output` first, then `Set`, then `Require`, then commands)? | Enforce order (opinionated) vs. Preserve order (conservative) |
| **FC-FMT-02** | Should `Set` commands be grouped and sorted alphabetically? | Yes (cleaner) vs. No (preserve user intent) |
| **FC-FMT-03** | Should the `@` in `Type@500ms` be formatted as `Type @500ms` (with space) or `Type@500ms` (no space, matching VHS convention)? | No space (recommended, matches VHS docs) vs. Space (more readable) |
| **FC-FMT-04** | Should the formatter add a blank line between the settings section and the command section if none exists? | Yes (opinionated) vs. No (only normalize existing blanks) |
