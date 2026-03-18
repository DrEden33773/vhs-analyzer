# SPEC_LEXER.md — VHS Token Set & Lexer Behavior

**Phase:** 1 — LSP Foundation
**Work Stream:** WS-1 (Lexer)
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect
**Last Updated:** 2026-03-18
**Frozen By:** Architect (Claude) — Stage B

---

> **CONTRACT_FROZEN** — This specification is frozen as of 2026-03-18.
> All Freeze Candidates have been resolved. No changes without explicit user approval.

---

## 1. Purpose

Define the complete VHS token set, lexer behavior, error token strategy, and
whitespace/comment handling for the `vhs-analyzer-core` crate. The lexer
transforms raw `.tape` source text into a flat sequence of tokens suitable
for consumption by the rowan-based recursive descent parser (WS-2).

## 2. Ground-Truth References

| Source | Role |
| --- | --- |
| [`tree-sitter-vhs/grammar.js`](https://github.com/charmbracelet/tree-sitter-vhs/blob/main/grammar.js) | Canonical syntax definition |
| [VHS README (charmbracelet/vhs)](https://github.com/charmbracelet/vhs) | Behavioral semantics & directive documentation |
| [matklad — Resilient LL Parsing Tutorial](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html) | Lexer architecture reference |

**Divergence note:** VHS v0.11.0 added `ScrollUp`, `ScrollDown`, and
`Screenshot` commands that are not yet reflected in the tree-sitter-vhs
grammar.js. This spec includes them based on the VHS README as the behavioral
ground truth.

## 3. Requirements

### LEX-001 — Lossless Tokenization

| Field | Value |
| --- | --- |
| **ID** | LEX-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The lexer MUST produce a token stream where the concatenation of all token texts exactly equals the original source text (lossless property). Every byte of input MUST map to exactly one token. |
| **Verification** | Round-trip property test: `tokens.map(\|t\| t.text).collect::<String>() == source` for any input. |

### LEX-002 — Error Resilience

| Field | Value |
| --- | --- |
| **ID** | LEX-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The lexer MUST NOT panic, return an error, or skip any byte of input. Unrecognized characters MUST be emitted as `ERROR` tokens. The lexer MUST always terminate. |
| **Verification** | Fuzz test with arbitrary byte sequences; assert no panics and lossless property holds. |

### LEX-003 — Whitespace Preservation

| Field | Value |
| --- | --- |
| **ID** | LEX-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Whitespace (spaces, tabs) MUST be emitted as `WHITESPACE` tokens. Newlines (`\n`, `\r\n`, `\r`) MUST be emitted as `NEWLINE` tokens, distinct from `WHITESPACE`. |
| **Verification** | Parse files with mixed line endings; verify `NEWLINE` tokens carry exact text. |

### LEX-004 — Comment Tokens

| Field | Value |
| --- | --- |
| **ID** | LEX-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Lines beginning with `#` (possibly preceded by whitespace) MUST be lexed as `COMMENT` tokens. The `COMMENT` token spans from `#` to end-of-line (excluding the newline itself). |
| **Verification** | Verify `# this is a comment` produces a single `COMMENT` token with text `# this is a comment`. |

### LEX-005 — Keyword Recognition

| Field | Value |
| --- | --- |
| **ID** | LEX-005 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | All VHS command keywords and setting name keywords (see §4) MUST be recognized as their specific keyword token types. Recognition is case-sensitive and exact-match. Bare words that do not match any keyword MUST be emitted as `IDENT` tokens. |
| **Verification** | Unit test each keyword string maps to the expected token kind. |

### LEX-006 — Numeric Literals

| Field | Value |
| --- | --- |
| **ID** | LEX-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Integer literals (`\d+`) MUST be emitted as `INTEGER`. Float literals (`\d*\.\d+`) MUST be emitted as `FLOAT`. The lexer MUST prefer `FLOAT` when a decimal point is present. |
| **Verification** | `42` → `INTEGER`, `3.14` → `FLOAT`, `.5` → `FLOAT`, `100` → `INTEGER`. |

### LEX-007 — String Literals

| Field | Value |
| --- | --- |
| **ID** | LEX-007 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Quoted strings delimited by `"`, `'`, or `` ` `` MUST be emitted as `STRING` tokens. The token text includes the delimiters. Unterminated strings (missing closing delimiter before newline or EOF) MUST be emitted as `STRING` tokens covering the available text, with the parser responsible for error reporting. |
| **Verification** | `"hello"` → `STRING`, `'world'` → `STRING`, `` `test` `` → `STRING`, `"unterminated` → `STRING` (with error flag deferred to parser). |

### LEX-008 — Time Literals

| Field | Value |
| --- | --- |
| **ID** | LEX-008 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | Time literals matching `\d*\.?\d+(ms\|s)` SHOULD be emitted as `TIME` tokens when the suffix `ms` or `s` is present. Without a suffix, the numeric part SHOULD be emitted as `INTEGER` or `FLOAT`. |
| **Verification** | `500ms` → `TIME`, `2s` → `TIME`, `0.5s` → `TIME`, `100` → `INTEGER`. |

### LEX-009 — Regex Literals

| Field | Value |
| --- | --- |
| **ID** | LEX-009 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Regex literals delimited by `/` MUST be emitted as `REGEX` tokens. The token text includes the delimiters. Unterminated regex (missing closing `/` before newline) MUST be emitted covering available text. |
| **Verification** | `/World/` → `REGEX`, `/pattern/` → `REGEX`. |

### LEX-010 — JSON Literals

| Field | Value |
| --- | --- |
| **ID** | LEX-010 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | JSON-like objects starting with `{` and ending with `}` MUST be emitted as `JSON` tokens. Brace matching MUST be performed to handle nested braces. |
| **Verification** | `{ "name": "Dracula" }` → single `JSON` token. |

### LEX-011 — Path Literals

| Field | Value |
| --- | --- |
| **ID** | LEX-011 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | File path literals matching `[\.\-\/A-Za-z0-9%]+` (containing at least one `/` or `.` to disambiguate from bare words) SHOULD be emitted as `PATH` tokens. |
| **Verification** | `demo.gif` → `PATH`, `./out/video.mp4` → `PATH`, `frames/` → `PATH`. |

### LEX-012 — Punctuation Tokens

| Field | Value |
| --- | --- |
| **ID** | LEX-012 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `@` character MUST be emitted as `AT`. The `+` character MUST be emitted as `PLUS`. The `%` character MUST be emitted as `PERCENT`. |
| **Verification** | Unit tests for each punctuation character. |

## 4. Complete Token Kind Enumeration

The following `SyntaxKind` variants represent token-level kinds. Node-level
kinds are defined in SPEC_PARSER.md. Both share the same `SyntaxKind` enum
(see PAR-001).

```text
// === Trivia & Error ===
ERROR           // unrecognized byte sequence
WHITESPACE      // spaces, tabs (not newlines)
NEWLINE         // \n, \r\n, \r
COMMENT         // # ... (to end of line)

// === Punctuation ===
AT              // @
PLUS            // +
PERCENT         // %

// === Literals ===
INTEGER         // \d+
FLOAT           // \d*\.\d+
STRING          // "...", '...', `...`
IDENT           // [A-Za-z][A-Za-z0-9_]* (not matching any keyword)
REGEX           // /.../
JSON            // { ... }
PATH            // file path with . or /
BOOLEAN         // true | false
TIME            // \d*\.?\d+(ms|s)

// === Command Keywords ===
OUTPUT_KW       // Output
SET_KW          // Set
ENV_KW          // Env
SLEEP_KW        // Sleep
TYPE_KW         // Type
BACKSPACE_KW    // Backspace
DOWN_KW         // Down
ENTER_KW        // Enter
ESCAPE_KW       // Escape
LEFT_KW         // Left
RIGHT_KW        // Right
SPACE_KW        // Space
TAB_KW          // Tab
UP_KW           // Up
PAGEUP_KW       // PageUp
PAGEDOWN_KW     // PageDown
SCROLLUP_KW     // ScrollUp     (VHS v0.11.0)
SCROLLDOWN_KW   // ScrollDown   (VHS v0.11.0)
WAIT_KW         // Wait
REQUIRE_KW      // Require
SOURCE_KW       // Source
HIDE_KW         // Hide
SHOW_KW         // Show
COPY_KW         // Copy
PASTE_KW        // Paste
SCREENSHOT_KW   // Screenshot   (VHS README, not in grammar.js)

// === Modifier Keywords ===
CTRL_KW         // Ctrl
ALT_KW          // Alt
SHIFT_KW        // Shift

// === Setting Name Keywords ===
SHELL_KW        // Shell
FONTFAMILY_KW   // FontFamily
FONTSIZE_KW     // FontSize
FRAMERATE_KW    // Framerate
PLAYBACKSPEED_KW // PlaybackSpeed
HEIGHT_KW       // Height
LETTERSPACING_KW // LetterSpacing
TYPINGSPEED_KW  // TypingSpeed
LINEHEIGHT_KW   // LineHeight
PADDING_KW      // Padding
THEME_KW        // Theme
LOOPOFFSET_KW   // LoopOffset
WIDTH_KW        // Width
BORDERRADIUS_KW // BorderRadius
MARGIN_KW       // Margin
MARGINFILL_KW   // MarginFill
WINDOWBAR_KW    // WindowBar
WINDOWBARSIZE_KW // WindowBarSize
CURSORBLINK_KW  // CursorBlink

// === Wait Scope Keywords ===
SCREEN_KW       // Screen
LINE_KW         // Line
```

**Total token kinds:** 63

## 5. Design Options Analysis

### 5.1 Lexer Implementation Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Hand-written** | Char-by-char state machine, `match` on current byte | Full control over error recovery; zero extra deps; matches matklad pattern | More boilerplate; manual maintenance |
| **B: `logos` crate** | Derive macro generates DFA from regex attributes | Less boilerplate; proven fast; well-tested | Extra dependency; less control over error-token boundaries; might complicate lossless guarantee |
| **C: `winnow`/`nom`** | Parser combinator for lexing | Composable; good error types | Overkill for flat token stream; combinator overhead |

**Recommended: Option A (Hand-written).** The VHS token set is small (63 kinds),
the grammar has no ambiguous lexical contexts, and hand-written lexers give
maximum control over the lossless and error-resilience guarantees required by
LEX-001 and LEX-002. This also matches the matklad tutorial architecture and
avoids adding dependencies to the core crate.

### 5.2 Keyword vs. Identifier Handling

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Always-keyword** | Bare words matching keyword list are ALWAYS emitted as keyword tokens; parser accepts keywords in value positions | Simple lexer; no state needed; parser handles ambiguity | `Require Set` produces `REQUIRE_KW SET_KW` — parser must accept `SET_KW` where a string value is expected |
| **B: Contextual** | Lexer tracks position context to decide keyword vs. ident | More "correct" token kinds | Complex state machine; violates lexer simplicity |
| **C: All-ident** | Lex all bare words as `IDENT`; parser matches by text | Simplest lexer | Parser loses type safety; keyword typos not caught early |

**Recommended: Option A (Always-keyword).** VHS commands always appear at
line-start, and the parser can trivially accept keyword tokens in value
positions. This keeps the lexer stateless and context-free.

### 5.3 Time Literal Handling

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Lex as TIME** | Recognize `\d+(ms\|s)` as a single `TIME` token | Clean token type; matches grammar.js | Lexer must look ahead past digits for suffix |
| **B: Separate tokens** | Emit `INTEGER` + `IDENT` for `500ms` | Simpler lexer | Parser must reassemble; hover on time literal is awkward |

**Recommended: Option A (Lex as TIME).** The lookahead for `ms`/`s` suffix is
trivial (2 chars max), and producing a unified `TIME` token simplifies both
the parser and hover provider.

### 5.4 Path vs. Ident Disambiguation

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Lexer heuristic** | Emit `PATH` when the token contains `/` or starts with `.`; otherwise `IDENT` | Correct in most cases | `demo.gif` is `PATH` but `demo` is `IDENT` — boundary can be fuzzy |
| **B: Parser decides** | Emit as `IDENT` or generic token; parser contextually promotes to path | Simpler lexer | Loses path info at token level |

**Recommended: Option A (Lexer heuristic).** Use the presence of `/` or a `.`
followed by a file extension (`gif`, `mp4`, `webm`, `tape`, `png`, `txt`,
`ascii`) as the heuristic. This covers all practical VHS use cases and avoids
burdening the parser.

## 6. Lexer API Contract

```rust
pub struct Token {
    pub kind: SyntaxKind,
    pub text: SmolStr,    // or &str with lifetime
}

pub fn lex(source: &str) -> Vec<Token>;
```

**Invariants:**

- `lex()` always returns a non-empty vector (at minimum, a single `NEWLINE` or `WHITESPACE` for empty-ish input, or tokens for any content).
- For empty input `""`, `lex()` returns an empty `Vec<Token>` (the lossless property holds vacuously).
- No `EOF` token is emitted by the lexer; the parser injects it as a virtual sentinel.

## 7. Resolved Design Decisions

All Freeze Candidates from Stage A have been closed with definitive decisions.

### FC-LEX-01 — ScrollUp/ScrollDown/Screenshot Keywords (RESOLVED: MUST include)

**Decision:** MUST include `SCROLLUP_KW`, `SCROLLDOWN_KW`, and `SCREENSHOT_KW`
as keyword token kinds.

**Rationale:** The VHS README (verified 2026-03-18) is the authoritative
behavioral specification. `ScrollUp` and `ScrollDown` were added in VHS v0.11.0
(March 2026) and are documented with the standard `Key[@time] [count]` syntax.
`Screenshot` accepts a path argument. The `tree-sitter-vhs` grammar.js has not
yet been updated to include these commands (last updated November 2025), but
the VHS README takes precedence per project convention (ROADMAP.md §2.2.1).
The lexer MUST recognize these keywords; ignoring them would cause false
`IDENT` tokens and break hover/completion for valid VHS commands.

### FC-LEX-02 — Copy Command String Argument (RESOLVED: MUST follow VHS README)

**Decision:** The `COPY_KW` token remains a simple keyword token. Whether `Copy`
accepts an optional string argument is a parser-level concern (see FC-PAR-04).
At the lexer level, no change is required — `Copy` is always lexed as `COPY_KW`,
and any following string is lexed as `STRING`.

**Rationale:** The VHS README documents `Copy "text"` with an explicit string
argument (e.g., `Copy "https://github.com/charmbracelet"`). The lexer is
stateless and does not need to know about argument structure. This is resolved
at the parser level in FC-PAR-04.

### FC-LEX-03 — Unterminated String Literals (RESOLVED: Single STRING token)

**Decision:** Unterminated string literals MUST be emitted as a single `STRING`
token covering all text from the opening delimiter to the end of the line
(or EOF). The parser is responsible for reporting the "unterminated string"
error.

**Rationale:** A single `STRING` token preserves the lossless property (LEX-001)
with minimal complexity. Splitting into `ERROR` + partial tokens would
complicate both the lexer and hover provider without improving error recovery
quality — the parser already isolates errors per-line (PAR-004). This matches
the rust-analyzer pattern where the lexer is lenient and the parser provides
semantic error reporting.

### FC-LEX-04 — Boolean Token Kind (RESOLVED: Dedicated BOOLEAN kind)

**Decision:** `true` and `false` MUST be lexed as `BOOLEAN` tokens, not as
keyword tokens (`TRUE_KW`/`FALSE_KW`) and not as `IDENT`.

**Rationale:** Boolean values are literals, not commands or setting names. Using
a dedicated `BOOLEAN` kind aligns with the semantic role of these tokens (they
appear only as values for `Set CursorBlink`). This also simplifies hover: a
`BOOLEAN` token in a setting context can provide "Boolean value: true/false"
documentation. Using keyword tokens would conflate value literals with command
keywords in the `SyntaxKind` enum.

### FC-LEX-05 — PATH vs. IDENT Disambiguation (RESOLVED: Extension allowlist)

**Decision:** The lexer MUST use an extension allowlist to distinguish `PATH`
from `IDENT`. A bare word containing a `.` followed by a recognized extension
is emitted as `PATH`. A bare word containing `/` is always `PATH`. All other
bare words are `IDENT`.

**Recognized extensions:** `gif`, `mp4`, `webm`, `tape`, `png`, `txt`, `ascii`,
`svg`, `jpg`, `jpeg`.

**Rationale:** VHS uses a finite set of file formats for `Output` and
`Screenshot` paths. An allowlist avoids false positives (e.g., `file.unknown`
should not be `PATH`) while covering all practical VHS use cases. The
allowlist is easily extensible if VHS adds new output formats. Bare words
with `/` are unambiguously paths (e.g., `./out/demo.gif`, `frames/`).
