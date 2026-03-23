# SPEC_TEST_MATRIX.md — Phase 2 Acceptance Test Scenarios

**Phase:** 2 — Intelligence & Diagnostics
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect → Builder
**Last Updated:** 2026-03-19

---

> **CONTRACT_FROZEN** — This specification is frozen as of 2026-03-19.
> No changes without explicit user approval.

---

## 1. Overview

This matrix defines acceptance test scenarios for all three Phase 2 work
streams. Each scenario specifies an ID, description, input, and expected
output. The Builder MUST implement tests covering every scenario listed here.

**Test ID Prefixes:**

- `T-CMP-NNN` — Completion (WS-1)
- `T-DIA-NNN` — Diagnostics (WS-2)
- `T-SAF-NNN` — Safety (WS-3)
- `T-INT2-NNN` — Phase 2 integration

## 2. WS-1: Completion Test Scenarios

### 2.1 Capability Advertisement

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-001 | completionProvider advertised | Send `initialize` request | Response contains `completionProvider` with `triggerCharacters: []` and `resolveProvider: false` |
| T-CMP-002 | Save capability advertised | Send `initialize` request | Response contains `textDocumentSync.save` with `includeText: false` |

### 2.2 Command Keyword Completions

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-010 | Keywords at empty line | Completion at column 0 of empty line | All VHS command keywords returned (Output, Set, Type, Sleep, etc.) with `kind: Keyword` |
| T-CMP-011 | Keywords at file start | Completion at position (0,0) of empty file | All VHS command keywords returned |
| T-CMP-012 | Keywords after newline | Completion at start of line after `Output demo.gif\n` | All VHS command keywords returned |
| T-CMP-013 | Keywords inside ERROR node | Completion at line start where previous line has parse error | All VHS command keywords returned |
| T-CMP-014 | Keyword detail text | Completion at empty line | Each keyword item has a non-empty `detail` description |
| T-CMP-015 | Keywords after partial line-start prefix | `S` at line start → completion | Includes command keywords such as `Set` and `Sleep` |

### 2.3 Setting Name Completions

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-020 | Setting names after Set | `Set` + space → completion after space | All 19 setting names returned with `kind: Property` |
| T-CMP-021 | Setting detail includes type | `Set` + space → completion | `FontSize` detail mentions numeric type; `Theme` detail mentions theme |
| T-CMP-022 | No settings outside Set | `Type` + space → completion after space | No setting name items returned |

### 2.4 Theme Name Completions

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-030 | Theme names after Set Theme | `Set Theme` + space → completion | Theme names returned with `kind: EnumMember`; at least 300 entries |
| T-CMP-031 | Dracula in list | `Set Theme` + space → completion | List contains `Dracula` |
| T-CMP-032 | Catppuccin Mocha quoted | `Set Theme` + space → completion | `Catppuccin Mocha` has `insertText: "\"Catppuccin Mocha\""` (wrapped in quotes) |
| T-CMP-033 | Nord unquoted | `Set Theme` + space → completion | `Nord` has `insertText: "Nord"` (no quotes, no spaces) |
| T-CMP-034 | Dark+ quoted | `Set Theme` + space → completion | `Dark+` has `insertText: "\"Dark+\""` (wrapped in quotes) |
| T-CMP-034A | catppuccin-frappe quoted | `Set Theme` + space → completion | `catppuccin-frappe` has `insertText: "\"catppuccin-frappe\""` (wrapped in quotes) |
| T-CMP-034B | Theme names inside empty quoted string | `Set Theme ""` → completion with cursor between quotes | Theme names returned with the same value-completion behavior as the bare value position |
| T-CMP-034C | Theme names inside empty single-quoted string | `Set Theme ''` → completion with cursor between quotes | Theme names returned with the same value-completion behavior as the bare value position |
| T-CMP-034D | Theme names inside partial quoted string | `Set Theme "D"` → completion with cursor after `D` | Theme names returned and filtered as Theme completions, not generic word suggestions |
| T-CMP-034E | Quoted Theme acceptance preserves quotes | Accept `Catppuccin Mocha` inside `Set Theme ""` | Completion uses `textEdit` so only the quoted contents are replaced; surrounding quotes stay intact |
| T-CMP-035 | No themes for other settings | `Set FontSize` + space → completion | No theme name items returned |

### 2.5 Setting Value Completions

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-040 | CursorBlink boolean values | `Set CursorBlink` + space → completion | Returns `true`, `false` with `kind: Value` |
| T-CMP-041 | WindowBar style values | `Set WindowBar` + space → completion | Returns `Colorful`, `ColorfulRight`, `Rings`, `RingsRight` with `kind: EnumMember` |
| T-CMP-042 | Shell common values | `Set Shell` + space → completion | Returns `bash`, `zsh`, `fish`, `sh`, `powershell`, `pwsh` with `kind: Value` |
| T-CMP-042A | Shell values inside empty quoted string | `Set Shell ""` → completion with cursor between quotes | Returns `bash`, `zsh`, `fish`, `sh`, `powershell`, `pwsh` |

### 2.6 Snippet Templates

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-050 | Type snippet | Completion at empty line | Contains `Type` snippet with `insertTextFormat: Snippet` and template `Type "${1:text}"` |
| T-CMP-051 | Output snippet | Completion at empty line | Contains `Output` snippet with extension choice tab stop |
| T-CMP-052 | Sleep snippet | Completion at empty line | Contains `Sleep` snippet with duration placeholder |

### 2.7 Output Extension Completions

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-060 | Output file extensions | `Output demo` → completion | Returns `.gif`, `.mp4`, `.webm` with `kind: File` |

### 2.8 Modifier Key Target Completions

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-070 | Ctrl key targets | `Ctrl+` → completion after `+` | Returns A–Z letters and special keys (Enter, Tab, etc.) with `kind: EnumMember` |
| T-CMP-071 | Alt key targets | `Alt+` → completion after `+` | Same target set as Ctrl |
| T-CMP-072 | Shift key targets | `Shift+` → completion after `+` | Same target set as Ctrl |

### 2.9 Edge Cases

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-080 | No completion in Type string | `Type "hello` → completion inside string | Returns `Ok(None)` — no completions |
| T-CMP-081 | No completion in comment | `# this is a comment` → completion inside | Returns `Ok(None)` — no completions |
| T-CMP-082 | Empty file | Completion at (0,0) of completely empty file | Returns command keyword completions |
| T-CMP-083 | No panic on arbitrary position | Property: completion at random offset in arbitrary `.tape` content | Never panics; returns `Ok(Some(...))` or `Ok(None)` |

### 2.10 Time Unit Completions

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-CMP-090 | Time units after first Sleep digit | `Sleep 1` → completion after `1` | Returns `ms`, `s` |
| T-CMP-091 | Time units after first Type duration digit | `Type@1 "x"` → completion after `1` | Returns `ms`, `s` |
| T-CMP-092 | Time units after first TypingSpeed digit | `Set TypingSpeed 1` → completion after `1` | Returns `ms`, `s` |
| T-CMP-093 | Time units after subsequent Sleep digits | `Sleep 10` → completion after `0` | Returns `ms`, `s` |
| T-CMP-094 | Time units after partial Sleep suffix | `Sleep 1000m` → completion after `m` | Returns `ms`, `s`; accepting `ms` replaces only the suffix fragment |
| T-CMP-095 | Time units after complete Sleep suffix | `Sleep 1000ms` → completion after trailing `s` | Returns `ms`, `s`; completion still targets only the suffix range |
| T-CMP-096 | Time units after partial Type duration suffix | `Type@1000m "x"` → completion after `m` | Returns `ms`, `s` |
| T-CMP-097 | Time units after partial TypingSpeed suffix | `Set TypingSpeed 1000m` → completion after `m` | Returns `ms`, `s` |

## 3. WS-2: Diagnostics Test Scenarios

### 3.1 Diagnostic Source and Code

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-001 | Source tag on all diagnostics | File with syntax error + semantic issue | All diagnostics have `source = "vhs-analyzer"` |
| T-DIA-002 | Semantic diagnostics have code | File with `Set FontSize 0` | Diagnostic has `code = "value-out-of-range"` |

### 3.2 Missing Output Directive

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-010 | Warning when no Output | File: `Set Theme Dracula\nType "hello"` | Warning on line 0: `"Missing Output directive..."` with code `missing-output` |
| T-DIA-011 | No warning with Output | File: `Output demo.gif\nType "hello"` | No `missing-output` diagnostic |
| T-DIA-012 | Warning clears on adding Output | Add `Output demo.gif` to file without Output | `missing-output` warning disappears |

### 3.3 Invalid Output Extension

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-020 | Error on .pdf extension | `Output demo.pdf` | Error: `"Invalid output format..."` with code `invalid-extension` |
| T-DIA-021 | No error on .gif | `Output demo.gif` | No `invalid-extension` diagnostic |
| T-DIA-022 | Case-insensitive .MP4 | `Output demo.MP4` | No `invalid-extension` diagnostic |
| T-DIA-023 | No error on .ascii | `Output golden.ascii` | No `invalid-extension` diagnostic |
| T-DIA-024 | No error on .txt | `Output golden.txt` | No `invalid-extension` diagnostic |
| T-DIA-025 | No error on directory path | `Output frames/` | No `invalid-extension` diagnostic |

### 3.4 Invalid Screenshot Extension

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-030 | No error on .png | `Screenshot demo.png` | No diagnostic |
| T-DIA-031 | Case-insensitive .PNG | `Screenshot demo.PNG` | No diagnostic |
| T-DIA-032 | Error on .jpg | `Screenshot demo.jpg` | Error: `"Invalid screenshot format. Supported: .png"` with code `invalid-screenshot-extension` |

### 3.5 Duplicate Set

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-040 | Warning on duplicate Set | `Set FontSize 14\nSet FontSize 20` | Warning on second occurrence with code `duplicate-set` and `relatedInformation` pointing to first |
| T-DIA-041 | No warning on different settings | `Set FontSize 14\nSet Width 800` | No `duplicate-set` diagnostic |

### 3.6 Invalid Hex Color

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-050 | Valid 6-digit hex | `Set MarginFill "#ff0000"` | No diagnostic |
| T-DIA-051 | Valid 3-digit hex | `Set MarginFill "#f00"` | No diagnostic |
| T-DIA-052 | Valid 8-digit hex (with alpha) | `Set MarginFill "#ff000080"` | No diagnostic |
| T-DIA-053 | Invalid 5-digit hex | `Set MarginFill "#12345"` | Error with code `invalid-hex-color` |
| T-DIA-054 | Invalid non-hex chars | `Set MarginFill "#xyz"` | Error with code `invalid-hex-color` |
| T-DIA-055 | Non-hex string is not validated | `Set MarginFill "wallpaper.png"` | No `invalid-hex-color` diagnostic |

### 3.7 Numeric Out of Range

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-060 | FontSize zero | `Set FontSize 0` | Error with code `value-out-of-range` |
| T-DIA-061 | FontSize valid | `Set FontSize 14` | No diagnostic |
| T-DIA-062 | Framerate negative | `Set Framerate -1` | Error with code `value-out-of-range` |
| T-DIA-063 | Padding zero valid | `Set Padding 0` | No diagnostic (>= 0 allowed) |
| T-DIA-064 | Padding negative | `Set Padding -5` | Error with code `value-out-of-range` |
| T-DIA-065 | BorderRadius zero valid | `Set BorderRadius 0` | No diagnostic (>= 0 allowed) |
| T-DIA-066 | Valid bare built-in theme | `Set Theme Dracula` | No parse error and no `unknown-theme` diagnostic |
| T-DIA-067 | Unknown built-in theme | `Set Theme "D"` | Error with code `unknown-theme` |

### 3.8 Require Program Not Found (Heavyweight)

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-070 | Warning for missing program | `Require nonexistent_program_xyz` (after save) | Warning with code `require-not-found` |
| T-DIA-071 | No warning for existing program | `Require sh` (after save, `sh` exists) | No `require-not-found` diagnostic |
| T-DIA-072 | Only on save, not on change | Type `Require nonexistent_program_xyz` without saving | No `require-not-found` diagnostic (heavyweight check not triggered) |

### 3.9 Source File Not Found (Heavyweight)

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-080 | Warning for missing source | `Source "nonexistent.tape"` (after save) | Warning with code `source-not-found` |
| T-DIA-081 | No warning for existing source | `Source "existing.tape"` with file present (after save) | No `source-not-found` diagnostic |

### 3.10 Timing and Pipeline

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-DIA-090 | Lightweight on didChange | Type invalid content without saving | Parse errors + lightweight semantic diagnostics appear immediately |
| T-DIA-091 | Heavyweight preserved across edits | Save with `Require missing`, then edit text | `require-not-found` diagnostic persists until next save |
| T-DIA-092 | Diagnostics cleared on didClose | Close document | All diagnostics for that document cleared |
| T-DIA-093 | No panic on arbitrary AST | Property: run lightweight diagnostics on arbitrary AST input | Never panics |

## 4. WS-3: Safety Test Scenarios

### 4.1 Destructive Filesystem Detection

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-SAF-001 | rm -rf / detected | `Type "rm -rf /"` | Critical (Error): `safety/destructive-fs` |
| T-SAF-002 | rm -fr ~/* detected | `Type "rm -fr ~/*"` | Critical (Error): `safety/destructive-fs` |
| T-SAF-003 | mkfs detected | `Type "mkfs.ext4 /dev/sda"` | Critical (Error): `safety/destructive-fs` |
| T-SAF-004 | dd overwrite detected | `Type "dd if=/dev/zero of=/dev/sda"` | Critical (Error): `safety/destructive-fs` |
| T-SAF-005 | Fork bomb detected | `Type ":(){ :\|: & };:"` | Critical (Error): `safety/destructive-fs` |

### 4.2 Privilege Escalation Detection

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-SAF-010 | sudo detected | `Type "sudo apt update"` | Warning: `safety/privilege-escalation` |
| T-SAF-011 | su root detected | `Type "su root"` | Warning: `safety/privilege-escalation` |
| T-SAF-012 | doas detected | `Type "doas reboot"` | Warning: `safety/privilege-escalation` |

### 4.3 Remote Code Execution Detection

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-SAF-020 | curl pipe to sh | `Type "curl https://x.com/s \| sh"` | Critical (Error): `safety/remote-exec` |
| T-SAF-021 | wget pipe to bash | `Type "wget -O- url \| bash"` | Critical (Error): `safety/remote-exec` |
| T-SAF-022 | eval detected | `Type "eval \"$cmd\""` | Info: `safety/remote-exec` |

### 4.4 Permission Modification Detection

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-SAF-030 | chmod 777 detected | `Type "chmod 777 /var/www"` | Warning: `safety/permission-mod` |
| T-SAF-031 | Recursive chmod on root | `Type "chmod -R 777 /"` | Critical (Error): `safety/permission-mod` |

### 4.5 False Positive Prevention

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-SAF-040 | rm file.txt not flagged | `Type "rm file.txt"` | No safety diagnostic |
| T-SAF-041 | chmod 644 not flagged | `Type "chmod 644 file.txt"` | No safety diagnostic |
| T-SAF-042 | curl without pipe not flagged | `Type "curl https://example.com"` | No safety diagnostic |
| T-SAF-043 | sudo not Critical | `Type "sudo apt update"` | Warning (NOT Critical) |

### 4.6 Suppression Mechanism

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-SAF-050 | Inline suppression works | `# vhs-analyzer-ignore: safety\nType "rm -rf /"` | No safety diagnostic |
| T-SAF-051 | Without suppression | `Type "rm -rf /"` (no preceding comment) | Critical safety diagnostic |
| T-SAF-052 | Partial suppression | `# vhs-analyzer-ignore: safety/destructive-fs\nType "rm -rf /"` | No `destructive-fs` diagnostic |

### 4.7 Pipeline Integration

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-SAF-060 | Pipeline stage detection | `Type "echo hello \| sudo rm -rf /"` | Detects `rm -rf /` in second pipeline stage |
| T-SAF-061 | Multiple string args | `Type "echo" "hello"` | Extracted text: `echo hello` |

### 4.8 Property-Based

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-SAF-070 | No panic on arbitrary Type content | Property: safety check on arbitrary string content in Type | Never panics |

## 5. Phase 2 Integration Test Scenarios

| ID | Description | Input | Expected Output |
| --- | --- | --- | --- |
| T-INT2-001 | Combined diagnostics | File with parse error + missing Output + `Type "rm -rf /"` | All three diagnostic types appear: syntax Error, `missing-output` Warning, `safety/destructive-fs` Critical |
| T-INT2-002 | Completion + diagnostics coexist | File with `Set FontSize 0` → request completion at new line | Completion returns keywords; `value-out-of-range` diagnostic present |
| T-INT2-003 | Server version 0.1.0 | Send `initialize` | `serverInfo.version` is `"0.1.0"` |
| T-INT2-004 | All Phase 1 features preserved | Hover and formatting still work after Phase 2 additions | Hover returns documentation; formatting produces correct output |
