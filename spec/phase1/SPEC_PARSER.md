# SPEC_PARSER.md — AST Design & Recursive Descent Parser

**Phase:** 1 — LSP Foundation
**Work Stream:** WS-2 (Parser)
**Status:** Stage A (Exploratory Design)
**Owner:** Architect
**Depends On:** WS-1 (SPEC_LEXER.md)
**Last Updated:** 2026-03-18

---

## 1. Purpose

Define the rowan-based lossless concrete syntax tree (CST) design, the
recursive descent parser architecture, and the error recovery strategy for
VHS `.tape` files. The parser consumes the flat token stream from the lexer
(WS-1) and produces a `rowan::GreenNode` tree that preserves every byte of
the original source.

## 2. Architecture References

| Source | Role |
| --- | --- |
| [matklad — Resilient LL Parsing Tutorial](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html) | Parser architecture blueprint |
| [rowan v0.16 API](https://docs.rs/rowan/0.16.1/rowan/) | Green/Red tree, GreenNodeBuilder, Checkpoint |
| [`tree-sitter-vhs/grammar.js`](https://github.com/charmbracelet/tree-sitter-vhs/blob/main/grammar.js) | Ground-truth syntax structure |

## 3. Requirements

### PAR-001 — Unified SyntaxKind Enum

| Field | Value |
| --- | --- |
| **ID** | PAR-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | A single `SyntaxKind` enum (`#[repr(u16)]`) MUST contain both token-level kinds (from SPEC_LEXER.md §4) and node-level kinds (from §4 below). This enum MUST implement `From<SyntaxKind> for rowan::SyntaxKind` via the `rowan::Language` trait. |
| **Verification** | The enum compiles and the `Language` impl is accepted by `rowan::GreenNodeBuilder`. |

### PAR-002 — Lossless Concrete Syntax Tree

| Field | Value |
| --- | --- |
| **ID** | PAR-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The parser MUST produce a rowan `GreenNode` tree whose leaf tokens, when concatenated in pre-order traversal, exactly reproduce the original source text. Whitespace, newlines, and comments MUST appear as leaf tokens in the tree. |
| **Verification** | Round-trip test: `SyntaxNode::new_root(green).text() == source` for all inputs. |

### PAR-003 — Error Resilience — No Panics

| Field | Value |
| --- | --- |
| **ID** | PAR-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The parser MUST NOT panic on any input. Malformed, incomplete, or empty `.tape` files MUST produce a valid `GreenNode` tree with `ERROR` nodes wrapping unrecognized fragments. The parser MUST always consume all tokens. |
| **Verification** | Fuzz testing with arbitrary token sequences; assert no panics. |

### PAR-004 — Error Localization

| Field | Value |
| --- | --- |
| **ID** | PAR-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Syntax errors in one command MUST NOT cascade into adjacent commands. Each command is delimited by newlines; a parse error within a command MUST be contained to that command's AST node. |
| **Verification** | Parse a file with one invalid command between two valid commands; verify the valid commands have correct AST nodes with no ERROR children. |

### PAR-005 — Fuel-Based Infinite Loop Protection

| Field | Value |
| --- | --- |
| **ID** | PAR-005 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The parser MUST implement a fuel mechanism (per matklad tutorial) to detect and abort infinite loops. Each `advance()` call replenishes fuel; each `nth()` lookahead consumes fuel. Fuel exhaustion MUST trigger a controlled abort (not panic). |
| **Verification** | Craft an input that would cause a naive parser to loop; verify the parser terminates within bounded time. |

### PAR-006 — All VHS Directives Parsed

| Field | Value |
| --- | --- |
| **ID** | PAR-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The parser MUST recognize and produce dedicated AST nodes for all VHS commands listed in §4 Node Kind Enumeration. Unrecognized commands MUST be wrapped in `ERROR` nodes. |
| **Verification** | Integration test parsing a file containing every VHS command; verify each command has the expected node kind. |

### PAR-007 — Typed AST Accessor Layer

| Field | Value |
| --- | --- |
| **ID** | PAR-007 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | A typed AST layer SHOULD be provided on top of the untyped rowan `SyntaxNode`. Each AST node type (e.g., `TypeCommand`, `SetCommand`) SHOULD provide accessor methods to retrieve child tokens and nodes by role (e.g., `fn string_arg(&self) -> Option<SyntaxToken>`). |
| **Verification** | Typed accessor for `TypeCommand` returns the string argument token. |

## 4. Node Kind Enumeration

These variants are appended to the `SyntaxKind` enum alongside the token kinds
from SPEC_LEXER.md §4.

```text
// === Root ===
SOURCE_FILE         // root node wrapping the entire file

// === Command Nodes ===
OUTPUT_COMMAND      // Output <path>
SET_COMMAND         // Set <setting-name> <value>
ENV_COMMAND         // Env <string> <string>
SLEEP_COMMAND       // Sleep <time>
TYPE_COMMAND        // Type [@<time>] <string>+
KEY_COMMAND         // Backspace|Down|Enter|Escape|Left|Right|Space|Tab|Up
                    //   |PageUp|PageDown|ScrollUp|ScrollDown [@<time>] [<int>]
CTRL_COMMAND        // Ctrl+[Alt+][Shift+]<key>
ALT_COMMAND         // Alt+[Shift+]<key>
SHIFT_COMMAND       // Shift+<key>
HIDE_COMMAND        // Hide
SHOW_COMMAND        // Show
COPY_COMMAND        // Copy [<string>]
PASTE_COMMAND       // Paste
SCREENSHOT_COMMAND  // Screenshot <path>
WAIT_COMMAND        // Wait [+Screen|+Line] [@<time>] <regex>
REQUIRE_COMMAND     // Require <string>
SOURCE_COMMAND      // Source <string>

// === Sub-expression Nodes ===
SETTING             // <setting-name> <value> (child of SET_COMMAND)
DURATION            // @ <time> (speed override)
WAIT_SCOPE          // + Screen | + Line
LOOP_OFFSET_SUFFIX  // <float> [%]

// === Error ===
ERROR               // wraps tokens that cannot be parsed into valid structure
```

**Total node kinds:** 22 (plus ERROR shared with token level)

## 5. Ungrammar (Syntax Tree Shape)

This section defines the expected tree shape using [ungrammar](https://rust-analyzer.github.io/blog/2020/10/24/introducing-ungrammar.html)
notation. This is descriptive of the CST, not prescriptive of valid programs.

```text
SourceFile = (Command | Comment | Newline)*

Command =
    OutputCommand
  | SetCommand
  | EnvCommand
  | SleepCommand
  | TypeCommand
  | KeyCommand
  | CtrlCommand
  | AltCommand
  | ShiftCommand
  | HideCommand
  | ShowCommand
  | CopyCommand
  | PasteCommand
  | ScreenshotCommand
  | WaitCommand
  | RequireCommand
  | SourceCommand

OutputCommand    = 'Output' Path
SetCommand       = 'Set' Setting
EnvCommand       = 'Env' String String
SleepCommand     = 'Sleep' Time
TypeCommand      = 'Type' Duration? String+
KeyCommand       = KeyKw Duration? Integer?
CtrlCommand      = 'Ctrl' '+' ('Alt' '+')? ('Shift' '+')? KeyTarget
AltCommand       = 'Alt' '+' ('Shift' '+')? KeyTarget
ShiftCommand     = 'Shift' '+' KeyTarget
HideCommand      = 'Hide'
ShowCommand      = 'Show'
CopyCommand      = 'Copy' String?
PasteCommand     = 'Paste'
ScreenshotCommand = 'Screenshot' Path
WaitCommand      = 'Wait' WaitScope? Duration? Regex
RequireCommand   = 'Require' String
SourceCommand    = 'Source' String

Setting =
    'Shell' String
  | 'FontFamily' String
  | 'FontSize' Float
  | 'Framerate' Integer
  | 'PlaybackSpeed' Float
  | 'Height' Integer
  | 'LetterSpacing' Float
  | 'TypingSpeed' Time
  | 'LineHeight' Float
  | 'Padding' Float
  | 'Theme' (Json | String)
  | 'LoopOffset' LoopOffsetSuffix
  | 'Width' Integer
  | 'BorderRadius' Integer
  | 'Margin' Integer
  | 'MarginFill' String
  | 'WindowBar' String
  | 'WindowBarSize' Integer
  | 'CursorBlink' Boolean

LoopOffsetSuffix = Float '%'?
Duration         = '@' Time
WaitScope        = '+' ('Screen' | 'Line')

KeyKw = 'Backspace' | 'Down' | 'Enter' | 'Escape' | 'Left'
      | 'Right' | 'Space' | 'Tab' | 'Up' | 'PageUp' | 'PageDown'
      | 'ScrollUp' | 'ScrollDown'

KeyTarget = Ident | 'Enter' | 'Tab'
```

## 6. Design Options Analysis

### 6.1 Parser Event Model

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Event list + build_tree()** | Parser emits `Open`/`Close`/`Advance` events into a flat Vec; second pass builds green tree (matklad pattern) | Proven architecture; supports `open_before` for left-associative constructs; decoupled from rowan | Extra pass; slightly more memory |
| **B: Direct GreenNodeBuilder** | Parser calls `builder.start_node()` / `builder.finish_node()` / `builder.token()` directly on `rowan::GreenNodeBuilder` | Simpler code; one pass; uses rowan Checkpoint for retroactive wrapping | Tighter coupling to rowan; Checkpoint API is less flexible than `open_before` |
| **C: Tree-sitter bridge** | Use tree-sitter-vhs via `tree-sitter` Rust bindings; convert tree-sitter CST to rowan | Reuse existing grammar; less parser code | Loses lossless property; tree-sitter error recovery is suboptimal for VHS (see matklad analysis); extra C dependency |

**Recommended: Option B (Direct GreenNodeBuilder).** VHS grammar has no
left-associative binary expressions or call chains that would benefit from
`open_before`. All commands are line-oriented with fixed structure. The
`GreenNodeBuilder::checkpoint()` API is sufficient for the one case needing
retroactive node wrapping (modifier key commands where `Ctrl` is parsed
before knowing if `Alt`/`Shift` follow). Direct builder also eliminates the
intermediate event allocation.

### 6.2 Error Recovery Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Newline-delimited recovery** | On error, skip tokens until next `NEWLINE` (or EOF), wrapping skipped tokens in `ERROR` node | Simple; leverages VHS line-oriented nature; guarantees error localization per PAR-004 | May lose partial valid structure within a malformed line |
| **B: Recovery sets** | Define FOLLOW/RECOVERY token sets per production (matklad improved pattern) | Finer-grained recovery within a command | More complex; VHS commands are simple enough that intra-command recovery has limited benefit |
| **C: Hybrid** | Use newline as primary recovery boundary; use recovery sets only within `SET_COMMAND` (which has the most substructure) | Balances simplicity and quality | Two patterns in one parser |

**Recommended: Option A (Newline-delimited recovery) with refinement.**
VHS is strictly line-oriented — each command occupies exactly one logical line.
The parser top-level loop iterates over lines. On encountering an unparseable
token at line start, skip to next `NEWLINE`, wrapping all skipped tokens in an
`ERROR` node. Within a command parse function, missing expected tokens are
reported but the parse continues to the end of the line. This provides
excellent error localization (PAR-004) with minimal complexity.

### 6.3 Unified KEY_COMMAND vs. Per-Key Nodes

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Unified KEY_COMMAND** | `Backspace`, `Down`, `Enter`, etc. all produce `KEY_COMMAND` node; the keyword token inside distinguishes them | Fewer node kinds; shared parsing logic; all share the same `[@time] [count]` shape | Typed AST layer needs to inspect child token to determine specific key |
| **B: Per-key nodes** | `BACKSPACE_COMMAND`, `DOWN_COMMAND`, `ENTER_COMMAND`, etc. — separate node kind per key | Maximally typed | 13+ extra node kinds for identical structure; bloated enum |

**Recommended: Option A (Unified KEY_COMMAND).** All repeatable key commands
share the exact same syntax: `<KeyKw> [Duration] [Integer]`. A single
`KEY_COMMAND` node with the keyword token as a child is cleaner. The typed
AST accessor can provide a `key_kind(&self) -> SyntaxKind` method.

### 6.4 Modifier Command Decomposition

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Multi-token decomposition** | Lex `Ctrl+Alt+A` as `CTRL_KW PLUS ALT_KW PLUS IDENT`; parser builds `CTRL_COMMAND` node | Fine-grained hover/completion on each modifier; resilient to partial input like `Ctrl+` | More tokens; parser must handle variable structure |
| **B: Single-token regex** | Lex entire `Ctrl+Alt+A` as one token (like tree-sitter-vhs) | Simple lexer | No intra-modifier hover; partial `Ctrl+` can't be tokenized |

**Recommended: Option A (Multi-token decomposition).** LSP features like
hover on `Ctrl` vs `Alt` and completion after `Ctrl+` require token-level
granularity. The parser's `CTRL_COMMAND` production handles the fixed
`CTRL_KW PLUS [ALT_KW PLUS] [SHIFT_KW PLUS] key` pattern.

## 7. Parser API Contract

```rust
pub fn parse(source: &str) -> Parse {
    let tokens = lex(source);
    let green = Parser::new(&tokens).parse_source_file();
    Parse { green, errors }
}

pub struct Parse {
    green: GreenNode,
    errors: Vec<ParseError>,
}

pub struct ParseError {
    pub range: TextRange,
    pub message: String,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }
}
```

**Design notes:**

- Errors are collected as a side channel (not stored in the tree itself),
  following the rust-analyzer pattern.
- `Parse` owns the `GreenNode` and creates `SyntaxNode` (red tree) on demand.
- The `SyntaxNode` is the "red" wrapper providing parent pointers and offset
  information, while `GreenNode` is the immutable, interned representation.

## 8. Parser Pseudocode — Top-Level Loop

```rust
fn parse_source_file(p: &mut Parser) {
    p.start_node(SOURCE_FILE);

    while !p.eof() {
        // Skip leading trivia (whitespace, newlines, comments)
        // They are automatically attached as children of SOURCE_FILE.

        match p.current() {
            NEWLINE | WHITESPACE | COMMENT => p.bump(),  // trivia
            OUTPUT_KW      => parse_output_command(p),
            SET_KW         => parse_set_command(p),
            ENV_KW         => parse_env_command(p),
            SLEEP_KW       => parse_sleep_command(p),
            TYPE_KW        => parse_type_command(p),
            // repeatable keys
            BACKSPACE_KW | DOWN_KW | ENTER_KW | ESCAPE_KW
            | LEFT_KW | RIGHT_KW | SPACE_KW | TAB_KW
            | UP_KW | PAGEUP_KW | PAGEDOWN_KW
            | SCROLLUP_KW | SCROLLDOWN_KW
                           => parse_key_command(p),
            CTRL_KW        => parse_ctrl_command(p),
            ALT_KW         => parse_alt_command(p),
            SHIFT_KW       => parse_shift_command(p),
            HIDE_KW        => parse_hide_command(p),
            SHOW_KW        => parse_show_command(p),
            COPY_KW        => parse_copy_command(p),
            PASTE_KW       => parse_paste_command(p),
            SCREENSHOT_KW  => parse_screenshot_command(p),
            WAIT_KW        => parse_wait_command(p),
            REQUIRE_KW     => parse_require_command(p),
            SOURCE_KW      => parse_source_command(p),
            _              => parse_error_to_eol(p),
        }
    }

    p.finish_node();  // SOURCE_FILE
}
```

Each `parse_*_command` function:

1. Calls `p.start_node(XXX_COMMAND)`.
2. Bumps the keyword token.
3. Parses expected arguments, calling `p.error()` for missing tokens.
4. Consumes remaining tokens until `NEWLINE` or `EOF`.
5. Calls `p.finish_node()`.

## 9. Freeze Candidates

| ID | Item | Options Under Consideration |
| --- | --- | --- |
| **FC-PAR-01** | Should parse errors be stored in the tree (as node properties) or as a side-channel Vec? | Side-channel (recommended, rust-analyzer pattern) vs. In-tree (simpler access) |
| **FC-PAR-02** | Should the typed AST layer be hand-written or macro-generated (like rust-analyzer's `ast::generated`)? | Hand-written (simpler for small grammar) vs. Macro/codegen (scales better) |
| **FC-PAR-03** | Handling of multi-line commands — VHS does not support line continuation; should the parser strictly enforce one-command-per-line? | Strict (reject multi-line) vs. Lenient (accept but warn) |
| **FC-PAR-04** | Should `COPY_COMMAND` include an optional string child (per VHS README) or be argument-less (per grammar.js)? | Linked to FC-LEX-02 |
