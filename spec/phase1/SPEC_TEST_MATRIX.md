# SPEC_TEST_MATRIX.md — Phase 1 Acceptance Test Matrix

**Phase:** 1 — LSP Foundation
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect
**Last Updated:** 2026-03-18
**Frozen By:** Architect (Claude) — Stage B

---

> **CONTRACT_FROZEN** — This test matrix is frozen as of 2026-03-18.
> Builder MUST implement all P0 test scenarios before Phase 1 completion.
> P1 scenarios SHOULD be implemented. P2 scenarios MAY be deferred.

---

## 1. Purpose

Define acceptance test scenarios for all Phase 1 requirements. Each scenario
specifies an ID, description, input, expected output, and the requirement(s)
it verifies. The Builder MUST use this matrix to guide test implementation
and verify contract compliance.

## 2. Test ID Convention

- `T-LEX-NNN` — Lexer tests
- `T-PAR-NNN` — Parser tests
- `T-LSP-NNN` — LSP Core tests
- `T-HOV-NNN` — Hover tests
- `T-FMT-NNN` — Formatting tests

## 3. Lexer Tests (WS-1)

### 3.1 Token Correctness

| ID | Description | Input | Expected Output | Req | Priority |
| --- | --- | --- | --- | --- | --- |
| T-LEX-001 | Round-trip lossless property | Any `.tape` file | `tokens.map(\|t\| t.text).collect::<String>() == source` | LEX-001 | P0 |
| T-LEX-002 | Empty input | `""` | Empty `Vec<Token>` | LEX-001 | P0 |
| T-LEX-003 | Whitespace-only input | `"   \t  "` | `[WHITESPACE("   \t  ")]` | LEX-003 | P0 |
| T-LEX-004 | Single newline variants | `"\n"`, `"\r\n"`, `"\r"` | `[NEWLINE]` for each, text matches input | LEX-003 | P0 |
| T-LEX-005 | Comment token | `"# this is a comment"` | `[COMMENT("# this is a comment")]` | LEX-004 | P0 |
| T-LEX-006 | Comment with leading whitespace | `"  # indented"` | `[WHITESPACE("  "), COMMENT("# indented")]` | LEX-003, LEX-004 | P0 |
| T-LEX-007 | All command keywords | `"Output"`, `"Set"`, `"Type"`, etc. | Each maps to its `*_KW` kind | LEX-005 | P0 |
| T-LEX-008 | Case sensitivity | `"output"`, `"TYPE"`, `"set"` | All → `IDENT` (not keywords) | LEX-005 | P0 |
| T-LEX-009 | Integer literal | `"42"` | `[INTEGER("42")]` | LEX-006 | P0 |
| T-LEX-010 | Float literal | `"3.14"` | `[FLOAT("3.14")]` | LEX-006 | P0 |
| T-LEX-011 | Float without leading digit | `".5"` | `[FLOAT(".5")]` | LEX-006 | P0 |
| T-LEX-012 | Double-quoted string | `"\"hello\""` | `[STRING("\"hello\"")]` | LEX-007 | P0 |
| T-LEX-013 | Single-quoted string | `"'world'"` | `[STRING("'world'")]` | LEX-007 | P0 |
| T-LEX-014 | Backtick string | `` "`test`" `` | `[STRING("`test`")]` | LEX-007 | P0 |
| T-LEX-015 | Unterminated string | `"\"unterminated"` | `[STRING("\"unterminated")]` | LEX-007 | P0 |
| T-LEX-016 | Time literal ms | `"500ms"` | `[TIME("500ms")]` | LEX-008 | P1 |
| T-LEX-017 | Time literal s | `"2s"` | `[TIME("2s")]` | LEX-008 | P1 |
| T-LEX-018 | Time literal float | `"0.5s"` | `[TIME("0.5s")]` | LEX-008 | P1 |
| T-LEX-019 | Regex literal | `"/World/"` | `[REGEX("/World/")]` | LEX-009 | P0 |
| T-LEX-020 | JSON literal | `"{ \"name\": \"Dracula\" }"` | `[JSON("{ \"name\": \"Dracula\" }")]` | LEX-010 | P0 |
| T-LEX-021 | Nested JSON braces | `"{ \"a\": { \"b\": 1 } }"` | Single `JSON` token | LEX-010 | P0 |
| T-LEX-022 | Path with slash | `"./out/demo.gif"` | `[PATH("./out/demo.gif")]` | LEX-011 | P1 |
| T-LEX-023 | Path with known extension | `"demo.gif"` | `[PATH("demo.gif")]` | LEX-011 | P1 |
| T-LEX-024 | Bare word unknown extension | `"file.unknown"` | `[IDENT("file"), ...]` (not PATH) | LEX-011 | P1 |
| T-LEX-025 | Punctuation @ | `"@"` | `[AT("@")]` | LEX-012 | P0 |
| T-LEX-026 | Punctuation + | `"+"` | `[PLUS("+")]` | LEX-012 | P0 |
| T-LEX-027 | Punctuation % | `"%"` | `[PERCENT("%")]` | LEX-012 | P0 |
| T-LEX-028 | Boolean true | `"true"` | `[BOOLEAN("true")]` | LEX-005 | P0 |
| T-LEX-029 | Boolean false | `"false"` | `[BOOLEAN("false")]` | LEX-005 | P0 |
| T-LEX-030 | ScrollUp keyword | `"ScrollUp"` | `[SCROLLUP_KW("ScrollUp")]` | LEX-005 | P0 |
| T-LEX-031 | ScrollDown keyword | `"ScrollDown"` | `[SCROLLDOWN_KW("ScrollDown")]` | LEX-005 | P0 |
| T-LEX-032 | Screenshot keyword | `"Screenshot"` | `[SCREENSHOT_KW("Screenshot")]` | LEX-005 | P0 |
| T-LEX-033 | Setting name keywords | `"FontSize"`, `"Theme"`, etc. | Each maps to its `*_KW` kind | LEX-005 | P0 |
| T-LEX-034 | Modifier keywords | `"Ctrl"`, `"Alt"`, `"Shift"` | `CTRL_KW`, `ALT_KW`, `SHIFT_KW` | LEX-005 | P0 |

### 3.2 Error Resilience

| ID | Description | Input | Expected Output | Req | Priority |
| --- | --- | --- | --- | --- | --- |
| T-LEX-050 | Unrecognized byte | `"\x00"` | `[ERROR("\x00")]` | LEX-002 | P0 |
| T-LEX-051 | Mixed valid/invalid | `"Type \x01 \"hello\""` | Tokens include ERROR for `\x01`, valid tokens around it | LEX-002 | P0 |
| T-LEX-052 | Fuzz: arbitrary bytes | Random byte sequences (proptest/quickcheck) | No panics; lossless round-trip holds | LEX-001, LEX-002 | P0 |
| T-LEX-053 | Very long input | 100KB of repeated `Type "hello"\n` | Completes without stack overflow | LEX-002 | P0 |

## 4. Parser Tests (WS-2)

### 4.1 Directive Coverage

| ID | Description | Input | Expected Output | Req | Priority |
| --- | --- | --- | --- | --- | --- |
| T-PAR-001 | Output command | `"Output demo.gif\n"` | `SOURCE_FILE → OUTPUT_COMMAND[OUTPUT_KW, PATH]` | PAR-006 | P0 |
| T-PAR-002 | Set command (integer) | `"Set FontSize 14\n"` | `SOURCE_FILE → SET_COMMAND[SET_KW, SETTING[FONTSIZE_KW, INTEGER]]` | PAR-006 | P0 |
| T-PAR-003 | Set command (string) | `"Set Shell \"bash\"\n"` | `SOURCE_FILE → SET_COMMAND[SET_KW, SETTING[SHELL_KW, STRING]]` | PAR-006 | P0 |
| T-PAR-004 | Set command (JSON theme) | `"Set Theme { \"name\": \"Dracula\" }\n"` | `SET_COMMAND` with `SETTING[THEME_KW, JSON]` | PAR-006 | P0 |
| T-PAR-005 | Set CursorBlink boolean | `"Set CursorBlink false\n"` | `SETTING[CURSORBLINK_KW, BOOLEAN]` | PAR-006 | P0 |
| T-PAR-006 | Set LoopOffset with % | `"Set LoopOffset 50%\n"` | `SETTING[LOOPOFFSET_KW, LOOP_OFFSET_SUFFIX[FLOAT/INTEGER, PERCENT]]` | PAR-006 | P0 |
| T-PAR-007 | Env command | `"Env HELLO \"WORLD\"\n"` | `ENV_COMMAND[ENV_KW, STRING/IDENT, STRING]` | PAR-006 | P0 |
| T-PAR-008 | Sleep command | `"Sleep 500ms\n"` | `SLEEP_COMMAND[SLEEP_KW, TIME]` | PAR-006 | P0 |
| T-PAR-009 | Type command basic | `"Type \"hello\"\n"` | `TYPE_COMMAND[TYPE_KW, STRING]` | PAR-006 | P0 |
| T-PAR-010 | Type with duration | `"Type@500ms \"slow\"\n"` | `TYPE_COMMAND[TYPE_KW, DURATION[AT, TIME], STRING]` | PAR-006 | P0 |
| T-PAR-011 | Key command (Enter) | `"Enter\n"` | `KEY_COMMAND[ENTER_KW]` | PAR-006 | P0 |
| T-PAR-012 | Key command with count | `"Backspace 5\n"` | `KEY_COMMAND[BACKSPACE_KW, INTEGER]` | PAR-006 | P0 |
| T-PAR-013 | Key command with duration and count | `"Tab@100ms 3\n"` | `KEY_COMMAND[TAB_KW, DURATION[AT, TIME], INTEGER]` | PAR-006 | P0 |
| T-PAR-014 | ScrollUp command | `"ScrollUp 10\n"` | `KEY_COMMAND[SCROLLUP_KW, INTEGER]` | PAR-006 | P0 |
| T-PAR-015 | ScrollDown with duration | `"ScrollDown@100ms 12\n"` | `KEY_COMMAND[SCROLLDOWN_KW, DURATION, INTEGER]` | PAR-006 | P0 |
| T-PAR-016 | Ctrl+key | `"Ctrl+C\n"` | `CTRL_COMMAND[CTRL_KW, PLUS, IDENT]` | PAR-006 | P0 |
| T-PAR-017 | Ctrl+Alt+key | `"Ctrl+Alt+Delete\n"` | `CTRL_COMMAND[CTRL_KW, PLUS, ALT_KW, PLUS, IDENT]` | PAR-006 | P0 |
| T-PAR-018 | Ctrl+Shift+key | `"Ctrl+Shift+A\n"` | `CTRL_COMMAND[CTRL_KW, PLUS, SHIFT_KW, PLUS, IDENT]` | PAR-006 | P0 |
| T-PAR-019 | Alt+key | `"Alt+Tab\n"` | `ALT_COMMAND[ALT_KW, PLUS, TAB_KW]` | PAR-006 | P0 |
| T-PAR-020 | Shift+key | `"Shift+Enter\n"` | `SHIFT_COMMAND[SHIFT_KW, PLUS, ENTER_KW]` | PAR-006 | P0 |
| T-PAR-021 | Hide command | `"Hide\n"` | `HIDE_COMMAND[HIDE_KW]` | PAR-006 | P0 |
| T-PAR-022 | Show command | `"Show\n"` | `SHOW_COMMAND[SHOW_KW]` | PAR-006 | P0 |
| T-PAR-023 | Copy standalone | `"Copy\n"` | `COPY_COMMAND[COPY_KW]` | PAR-006 | P0 |
| T-PAR-024 | Copy with string | `"Copy \"text\"\n"` | `COPY_COMMAND[COPY_KW, STRING]` | PAR-006 | P0 |
| T-PAR-025 | Paste command | `"Paste\n"` | `PASTE_COMMAND[PASTE_KW]` | PAR-006 | P0 |
| T-PAR-026 | Screenshot command | `"Screenshot examples/screenshot.png\n"` | `SCREENSHOT_COMMAND[SCREENSHOT_KW, PATH]` | PAR-006 | P0 |
| T-PAR-027 | Wait basic | `"Wait /World/\n"` | `WAIT_COMMAND[WAIT_KW, REGEX]` | PAR-006 | P0 |
| T-PAR-028 | Wait with scope | `"Wait+Screen /World/\n"` | `WAIT_COMMAND[WAIT_KW, WAIT_SCOPE[PLUS, SCREEN_KW], REGEX]` | PAR-006 | P0 |
| T-PAR-029 | Wait with scope and duration | `"Wait+Line@10ms /World/\n"` | `WAIT_COMMAND[WAIT_KW, WAIT_SCOPE, DURATION, REGEX]` | PAR-006 | P0 |
| T-PAR-030 | Require command | `"Require git\n"` | `REQUIRE_COMMAND[REQUIRE_KW, IDENT/STRING]` | PAR-006 | P0 |
| T-PAR-031 | Source command | `"Source config.tape\n"` | `SOURCE_COMMAND[SOURCE_KW, STRING/PATH]` | PAR-006 | P0 |

### 4.2 Lossless CST and Error Recovery

| ID | Description | Input | Expected Output | Req | Priority |
| --- | --- | --- | --- | --- | --- |
| T-PAR-050 | Lossless round-trip | Any valid `.tape` file | `SyntaxNode::new_root(green).text() == source` | PAR-002 | P0 |
| T-PAR-051 | Empty file | `""` | `SOURCE_FILE` with no children | PAR-003 | P0 |
| T-PAR-052 | Comments and blank lines only | `"# comment\n\n# another\n"` | `SOURCE_FILE` with `COMMENT` and `NEWLINE` children | PAR-002 | P0 |
| T-PAR-053 | Error localization | `"Type \"ok\"\nINVALID_STUFF\nEnter\n"` | Line 1: valid `TYPE_COMMAND`, Line 2: `ERROR` node, Line 3: valid `KEY_COMMAND` | PAR-004 | P0 |
| T-PAR-054 | Missing argument | `"Set FontSize\n"` | `SET_COMMAND` with error reported (missing value); next line parses correctly | PAR-003, PAR-004 | P0 |
| T-PAR-055 | Extra tokens on line | `"Hide extra tokens\n"` | `HIDE_COMMAND` containing `ERROR` wrapping extra tokens | PAR-004 | P0 |
| T-PAR-056 | Fuel exhaustion | Pathological input designed to cause loops | Parser terminates; no hang | PAR-005 | P0 |
| T-PAR-057 | Fuzz: arbitrary tokens | Random token sequences (proptest) | No panics; valid `GreenNode` produced | PAR-003 | P0 |
| T-PAR-058 | Strict one-command-per-line | `"Type \"a\" Type \"b\"\n"` | First `Type` parsed; `Type "b"` wrapped in `ERROR` | PAR-003 | P0 |
| T-PAR-059 | Mixed valid and error lines | Multi-line file with alternating valid/invalid | Valid lines have correct nodes; invalid lines have ERROR | PAR-004 | P0 |

### 4.3 Typed AST Layer

| ID | Description | Input | Expected Output | Req | Priority |
| --- | --- | --- | --- | --- | --- |
| T-PAR-070 | TypeCommand accessor | `"Type \"hello\"\n"` | `TypeCommand::string_arg()` returns `Some("\"hello\"")` | PAR-007 | P1 |
| T-PAR-071 | SetCommand accessor | `"Set FontSize 14\n"` | `SetCommand::setting()` returns a `Setting` with name and value | PAR-007 | P1 |
| T-PAR-072 | KeyCommand key_kind | `"Enter\n"` | `KeyCommand::key_kind()` returns `ENTER_KW` | PAR-007 | P1 |
| T-PAR-073 | Duration accessor | `"Type@500ms \"text\"\n"` | `TypeCommand::duration()` returns `Some(Duration)` with time | PAR-007 | P1 |

## 5. LSP Core Tests (WS-3)

| ID | Description | Input | Expected Output | Req | Priority |
| --- | --- | --- | --- | --- | --- |
| T-LSP-001 | Server starts via stdio | Launch binary | Process starts, accepts stdin | LSP-001 | P0 |
| T-LSP-002 | Initialize handshake | Send `initialize` request | Response contains `textDocumentSync`, `hoverProvider`, `documentFormattingProvider` capabilities | LSP-002 | P0 |
| T-LSP-003 | Server info in initialize | Send `initialize` request | Response `serverInfo.name == "vhs-analyzer"`, `serverInfo.version == "0.1.0"` | LSP-002 | P0 |
| T-LSP-004 | didOpen stores document | Send `didOpen` with content | Document stored in state; parse tree available | LSP-003 | P0 |
| T-LSP-005 | didChange re-parses | Send `didOpen` then `didChange` with new content | Stored parse tree reflects new content | LSP-003 | P0 |
| T-LSP-006 | didClose removes document | Send `didOpen` then `didClose` | Document removed from state | LSP-003 | P0 |
| T-LSP-007 | Concurrent document access | Open doc A, open doc B, hover on A while changing B | No deadlock, no data corruption | LSP-004 | P0 |
| T-LSP-008 | Shutdown then exit | Send `shutdown` then `exit` | Process terminates with exit code 0 | LSP-005 | P0 |
| T-LSP-009 | No panic on invalid state | Force internal error (e.g., hover on unknown URI) | LSP error response returned; server continues | LSP-006 | P0 |
| T-LSP-010 | Parse-error diagnostics published | `didOpen` with `"INVALID\n"` | `publishDiagnostics` notification with at least one error diagnostic | LSP-008 | P1 |
| T-LSP-011 | Diagnostics clear on fix | `didChange` replacing error with valid content | `publishDiagnostics` with empty list | LSP-008 | P1 |
| T-LSP-012 | Diagnostics clear on close | `didClose` after error document | `publishDiagnostics` with empty list for that URI | LSP-008 | P1 |
| T-LSP-013 | Logging to stderr | Start server with tracing enabled | Log output appears on stderr, not stdout | LSP-007 | P1 |

## 6. Hover Tests (WS-4)

| ID | Description | Input (hover position) | Expected Output | Req | Priority |
| --- | --- | --- | --- | --- | --- |
| T-HOV-001 | Hover on Type keyword | `"Type \"hello\"\n"` → hover at col 0 | Markdown with "Type", description, syntax, example | HOV-002 | P0 |
| T-HOV-002 | Hover on Sleep keyword | `"Sleep 500ms\n"` → hover at col 0 | Markdown with "Sleep" documentation | HOV-002 | P0 |
| T-HOV-003 | Hover on Output keyword | `"Output demo.gif\n"` → hover at col 0 | Markdown with "Output" documentation | HOV-002 | P0 |
| T-HOV-004 | Hover on Set keyword | `"Set FontSize 14\n"` → hover on `Set` | Markdown with "Set" documentation | HOV-002 | P0 |
| T-HOV-005 | Hover on FontSize setting | `"Set FontSize 14\n"` → hover on `FontSize` | Markdown with "FontSize" setting docs (type: float) | HOV-003 | P0 |
| T-HOV-006 | Hover on Theme setting | `"Set Theme \"Dracula\"\n"` → hover on `Theme` | Markdown with "Theme" setting docs (type: string/JSON) | HOV-003 | P0 |
| T-HOV-007 | Hover on all 19 setting keywords | Each setting keyword in `Set <name> <value>` | Each returns non-empty Markdown documentation | HOV-003 | P0 |
| T-HOV-008 | Hover on Ctrl modifier | `"Ctrl+C\n"` → hover on `Ctrl` | Modifier documentation | HOV-004 | P1 |
| T-HOV-009 | Hover on Alt modifier | `"Alt+Tab\n"` → hover on `Alt` | Modifier documentation | HOV-004 | P1 |
| T-HOV-010 | Hover on whitespace | `"Type \"hello\"\n"` → hover on space between | `null` (no hover) | HOV-001 | P0 |
| T-HOV-011 | Hover on comment | `"# comment\n"` → hover on `#` | `null` (no hover) | HOV-001 | P0 |
| T-HOV-012 | Hover range matches token | `"Type \"hello\"\n"` → hover on `Type` | Hover range spans exactly `Type` (col 0..4) | HOV-001 | P0 |
| T-HOV-013 | Context-sensitive Enter (command) | `"Enter\n"` → hover on `Enter` | "Press the Enter key" (KEY_COMMAND context) | HOV-006 | P0 |
| T-HOV-014 | Context-sensitive Enter (Ctrl target) | `"Ctrl+Enter\n"` → hover on `Enter` | "Target key for Ctrl combination" (CTRL_COMMAND context) | HOV-006 | P0 |
| T-HOV-015 | Hover on literal value | `"Sleep 500ms\n"` → hover on `500ms` | Optional: "Duration: 500 milliseconds" | HOV-005 | P2 |
| T-HOV-016 | All command keywords have hover | Each of the 27 command keywords | Non-empty Markdown returned for each | HOV-002 | P0 |

## 7. Formatting Tests (WS-5)

| ID | Description | Input | Expected Output | Req | Priority |
| --- | --- | --- | --- | --- | --- |
| T-FMT-001 | Already canonical — zero edits | Canonical `.tape` file (per §4.3 of SPEC_FORMATTING) | Empty `Vec<TextEdit>` | FMT-001 | P0 |
| T-FMT-002 | Idempotence | Any `.tape` file, format twice | Second formatting returns zero edits | FMT-001 | P0 |
| T-FMT-003 | Remove leading indentation | `"  Type \"hello\"\n"` | `"Type \"hello\"\n"` | FMT-002 | P0 |
| T-FMT-004 | Remove leading tabs | `"\tSet FontSize 14\n"` | `"Set FontSize 14\n"` | FMT-002 | P0 |
| T-FMT-005 | Collapse multiple spaces | `"Set   FontSize   14\n"` | `"Set FontSize 14\n"` | FMT-003 | P0 |
| T-FMT-006 | No space around + in Ctrl | `"Ctrl + C\n"` | `"Ctrl+C\n"` | FMT-004 | P0 |
| T-FMT-007 | No space around @ in duration | `"Type @ 500ms \"text\"\n"` | `"Type@500ms \"text\"\n"` | FMT-004 | P0 |
| T-FMT-008 | Collapse blank lines | `"Type \"a\"\n\n\n\nEnter\n"` | `"Type \"a\"\n\nEnter\n"` (single blank line preserved) | FMT-005 | P1 |
| T-FMT-009 | Preserve single blank line | `"Type \"a\"\n\nEnter\n"` | Unchanged | FMT-005 | P1 |
| T-FMT-010 | Remove trailing whitespace | `"Type \"hello\"   \n"` | `"Type \"hello\"\n"` | FMT-006 | P0 |
| T-FMT-011 | Add final newline | `"Type \"hello\""` (no trailing newline) | `"Type \"hello\"\n"` | FMT-007 | P1 |
| T-FMT-012 | Collapse trailing newlines | `"Type \"hello\"\n\n\n"` | `"Type \"hello\"\n"` | FMT-007 | P1 |
| T-FMT-013 | Comment preserved verbatim | `"# My comment\n"` | Unchanged | FMT-008 | P0 |
| T-FMT-014 | Indented comment stripped | `"  # indented\n"` | `"# indented\n"` | FMT-008, FMT-002 | P0 |
| T-FMT-015 | Error line preserved | `"Type \"ok\"\nINVALID STUFF\nEnter\n"` | Line 1 and 3 formatted; line 2 unchanged | FMT-009 | P0 |
| T-FMT-016 | Complex mixed file | Full `.tape` with various issues | All formatting rules applied correctly | FMT-001 through FMT-009 | P0 |
| T-FMT-017 | Directive order preserved | `"Set FontSize 14\nOutput demo.gif\n"` | Order unchanged (Output after Set) | FMT-001 | P0 |
| T-FMT-018 | Ctrl+Alt+Shift spacing | `"Ctrl + Alt + Shift + A\n"` | `"Ctrl+Alt+Shift+A\n"` | FMT-004 | P0 |

## 8. Coverage Summary

| Category | P0 (MUST) | P1 (SHOULD) | P2 (MAY) | Total |
| --- | --- | --- | --- | --- |
| Lexer (T-LEX) | 30 | 6 | 0 | 36 |
| Parser (T-PAR) | 21 | 4 | 0 | 25 |
| LSP Core (T-LSP) | 9 | 4 | 0 | 13 |
| Hover (T-HOV) | 11 | 2 | 1 | 14 |
| Formatting (T-FMT) | 13 | 4 | 0 | 17 |
| **Total** | **84** | **20** | **1** | **105** |

## 9. Property-Based Testing Requirements

The following requirements MUST be verified via property-based testing
(proptest or quickcheck), not just example-based tests:

| Property | Generator | Assertion | Reqs |
| --- | --- | --- | --- |
| Lexer lossless | Arbitrary byte strings | Token texts concatenate to input | LEX-001 |
| Lexer no-panic | Arbitrary byte strings | No panics | LEX-002 |
| Parser lossless | Arbitrary token sequences | `SyntaxNode.text() == source` | PAR-002 |
| Parser no-panic | Arbitrary token sequences | No panics | PAR-003 |
| Formatter idempotence | Any valid `.tape` string | `format(format(input)) == format(input)` | FMT-001 |

## 10. Integration Test Scenario

One end-to-end integration test MUST verify the full pipeline:

1. Start LSP server via stdio.
2. Send `initialize` → verify capabilities.
3. Send `didOpen` with a `.tape` file containing valid and invalid commands.
4. Send `hover` on a command keyword → verify Markdown response.
5. Send `formatting` → verify `TextEdit` list.
6. Send `didChange` with corrected content → verify diagnostics clear.
7. Send `shutdown` then `exit` → verify clean termination.

**Test ID:** T-INT-001
**Priority:** P0
**Verifies:** LSP-001 through LSP-006, HOV-001, FMT-001
