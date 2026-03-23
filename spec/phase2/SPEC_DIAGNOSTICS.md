# SPEC_DIAGNOSTICS.md — Semantic Diagnostics & Environment Checks

**Phase:** 2 — Intelligence & Diagnostics
**Work Stream:** WS-2 (Diagnostics)
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect
**Depends On:** phase1/SPEC_PARSER.md (AST), phase1/SPEC_LSP_CORE.md (server lifecycle, LSP-008 parse diagnostics)
**Last Updated:** 2026-03-19
**Frozen By:** Architect (Claude) — Stage B

---

> **CONTRACT_FROZEN** — This specification is frozen as of 2026-03-19.
> All Freeze Candidates have been resolved. No changes without explicit user approval.

---

## 1. Purpose

Define the semantic diagnostic rule set, severity mapping, timing strategy,
and unified diagnostic pipeline for VHS `.tape` files. Phase 1 (LSP-008)
already publishes parse-error diagnostics. Phase 2 extends this with
semantic validation (invalid values, missing directives, duplicate settings)
and environment checks (file existence, `$PATH` program lookup).

## 2. Cross-Phase Dependencies

| Phase 1 Contract | Usage in This Spec |
| --- | --- |
| SPEC_PARSER.md — PAR-001 (SyntaxKind enum) | Diagnostic rules inspect `SyntaxKind` of tokens to classify command and setting types |
| SPEC_PARSER.md — PAR-007 (Typed AST accessors) | `SetCommand::setting_keyword()`, `RequireCommand::string_arg()`, `OutputCommand::path()` etc. extract children for validation |
| SPEC_PARSER.md — §4 (Node Kind Enumeration) | `SET_COMMAND`, `OUTPUT_COMMAND`, `REQUIRE_COMMAND`, `SOURCE_COMMAND` are the primary diagnostic targets |
| SPEC_LSP_CORE.md — LSP-004 (Document State) | Diagnostics read parsed AST from `DashMap<Url, DocumentState>` |
| SPEC_LSP_CORE.md — LSP-008 (Parse-error diagnostics) | Phase 2 diagnostics extend (not replace) Phase 1 parse-error diagnostics into a unified pipeline |
| SPEC_LSP_CORE.md — FC-LSP-02 (didSave) | Phase 2 adds a `didSave` handler for heavyweight diagnostics |

## 3. References

| Source | Role |
| --- | --- |
| [LSP 3.17 — publishDiagnostics](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics) | Protocol contract for diagnostic notifications |
| [LSP 3.17 — Diagnostic](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#diagnostic) | `Diagnostic` struct: range, severity, code, source, message, tags, relatedInformation |
| [VHS README](https://github.com/charmbracelet/vhs?tab=readme-ov-file) | Valid output extensions, setting value constraints, directive semantics |
| Rust Async Patterns skill | Tokio patterns for background task spawning and cancellation in heavyweight checks |

## 4. Requirements

### DIA-001 — Diagnostic Source Tag

| Field | Value |
| --- | --- |
| **ID** | DIA-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | ALL diagnostics emitted by the server (both parse-error and semantic) MUST set `source: Some("vhs-analyzer")`. Semantic diagnostics MUST additionally set `code` to a rule identifier string (e.g., `"missing-output"`, `"invalid-extension"`, `"duplicate-set"`) to allow per-rule filtering. |
| **Verification** | Open a file with both syntax errors and semantic issues; verify all diagnostics have `source = "vhs-analyzer"` and semantic diagnostics have distinct `code` values. |

### DIA-002 — Severity Mapping

| Field | Value |
| --- | --- |
| **ID** | DIA-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Diagnostic severities MUST follow the mapping defined in §6. Parse errors are always `Error`. Semantic rule severities are assigned per-rule and MUST NOT be configurable in Phase 2. |
| **Verification** | Verify each diagnostic rule emits the severity specified in §6. |

### DIA-003 — Missing Output Directive

| Field | Value |
| --- | --- |
| **ID** | DIA-003 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | If a `.tape` file contains no `OUTPUT_COMMAND` node, the server SHOULD emit a `Warning` diagnostic on the first line (range: line 0, columns 0–0) with message `"Missing Output directive. VHS will not produce an output file."` and code `"missing-output"`. |
| **Verification** | Open a file without `Output`; verify warning appears. Add `Output demo.gif`; verify warning disappears. |

### DIA-004 — Invalid Output Path Extension

| Field | Value |
| --- | --- |
| **ID** | DIA-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | If an `OUTPUT_COMMAND` path has a file extension (contains `.` after the last `/` or from start) and that extension is not one of `.gif`, `.mp4`, `.webm`, `.ascii`, `.txt` (case-insensitive), the server MUST emit an `Error` diagnostic on the `PATH` token range with message `"Invalid output format. Supported: .gif, .mp4, .webm, .ascii, .txt"` and code `"invalid-extension"`. Paths ending with `/` (directory for PNG frame sequences) or paths without a recognizable extension MUST NOT be flagged. |
| **Verification** | `Output demo.pdf` → Error. `Output demo.gif` → no error. `Output demo.MP4` → no error (case-insensitive). `Output golden.ascii` → no error. `Output frames/` → no error. |

### DIA-005 — Duplicate Set for Same Setting

| Field | Value |
| --- | --- |
| **ID** | DIA-005 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | If multiple `SET_COMMAND` nodes set the same setting (identified by the setting name keyword kind), the server SHOULD emit a `Warning` diagnostic on each duplicate (all occurrences after the first) with message `"Duplicate Set {setting}. Only the last value takes effect."` and code `"duplicate-set"`. The `relatedInformation` field SHOULD point to the first occurrence. |
| **Verification** | File with `Set FontSize 14` and `Set FontSize 20` → warning on the second occurrence with related link to the first. |

### DIA-006 — Invalid Hex Color in MarginFill

| Field | Value |
| --- | --- |
| **ID** | DIA-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | If a `SET_COMMAND` for `MarginFill` has a string value starting with `#`, the server MUST validate the hex color format. Valid formats: `#RGB` (3 hex digits), `#RRGGBB` (6 hex digits), `#RRGGBBAA` (8 hex digits). Invalid hex patterns MUST produce an `Error` diagnostic on the string token range with message `"Invalid hex color. Expected #RGB, #RRGGBB, or #RRGGBBAA"` and code `"invalid-hex-color"`. Non-`#` strings in `MarginFill` are assumed to be image file paths and MUST NOT be validated as hex. |
| **Verification** | `Set MarginFill "#ff0000"` → no error. `Set MarginFill "#xyz"` → error. `Set MarginFill "#12345"` → error (5 digits). `Set MarginFill "wallpaper.png"` → no error. |

### DIA-007 — Numeric Value Out of Range

| Field | Value |
| --- | --- |
| **ID** | DIA-007 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Numeric setting values MUST be validated against their allowed ranges. The server MUST emit an `Error` diagnostic on the value token when a value is out of range. Ranges defined in §7. Code: `"value-out-of-range"`. |
| **Verification** | `Set FontSize 0` → error. `Set FontSize 14` → no error. `Set Framerate -1` → error. `Set Padding -5` → error. |

### DIA-008 — Require Program Not Found

| Field | Value |
| --- | --- |
| **ID** | DIA-008 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | For each `REQUIRE_COMMAND`, the server SHOULD check whether the specified program exists in the system `$PATH`. If not found, emit a `Warning` diagnostic on the string argument token with message `"Program '{name}' not found in $PATH"` and code `"require-not-found"`. This is a heavyweight check (see DIA-010). |
| **Verification** | `Require git` with git installed → no warning. `Require nonexistent_program_xyz` → warning. |

### DIA-009 — Source File Not Found

| Field | Value |
| --- | --- |
| **ID** | DIA-009 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | For each `SOURCE_COMMAND`, the server SHOULD check whether the referenced tape file exists (resolved relative to the current file's directory, or to the workspace root). If not found, emit a `Warning` diagnostic on the string argument token with message `"Source file '{path}' not found"` and code `"source-not-found"`. This is a heavyweight check (see DIA-010). |
| **Verification** | `Source "setup.tape"` when `setup.tape` exists → no warning. `Source "nonexistent.tape"` → warning. |

### DIA-010 — Diagnostic Timing Classification

| Field | Value |
| --- | --- |
| **ID** | DIA-010 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Diagnostic rules MUST be classified as either **lightweight** or **heavyweight** per §8. Lightweight rules MUST run synchronously on every `didChange` event. Heavyweight rules MUST run only on `didSave` events (or on initial `didOpen`), and MUST execute their I/O operations asynchronously using `tokio::spawn` to avoid blocking the LSP message loop. |
| **Verification** | Typing rapidly does not block; heavyweight diagnostics appear only after save. |

### DIA-011 — Unified Diagnostic Pipeline

| Field | Value |
| --- | --- |
| **ID** | DIA-011 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The server MUST publish a single unified diagnostic list per document via `client.publish_diagnostics()`. The list MUST contain: (1) parse errors from the parser (Phase 1 baseline), (2) lightweight semantic diagnostics, and (3) heavyweight diagnostic results (when available). On `didChange`, the server MUST publish parse errors + lightweight diagnostics immediately, preserving the latest heavyweight results from the last `didSave`. On `didSave`, the server MUST re-run heavyweight checks and publish the combined updated list. On `didClose`, all diagnostics MUST be cleared. |
| **Verification** | After typing (no save): parse errors and lightweight diagnostics appear, heavyweight results from last save persist. After save: all diagnostics update including heavyweight results. |

### DIA-012 — Heavyweight Check Cancellation

| Field | Value |
| --- | --- |
| **ID** | DIA-012 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | If a new `didSave` arrives while a previous heavyweight check is still running for the same document, the server SHOULD cancel the in-flight check (via `tokio_util::sync::CancellationToken` or `JoinHandle::abort()`) before starting a new one, to avoid publishing stale results. |
| **Verification** | Rapid save-save with slow heavyweight check: only the latest check's results are published. |

### DIA-013 — Invalid Screenshot Path Extension

| Field | Value |
| --- | --- |
| **ID** | DIA-013 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | If a `SCREENSHOT_COMMAND` path does not end with `.png` (case-insensitive), the server MUST emit an `Error` diagnostic on the `PATH` token range with message `"Invalid screenshot format. Supported: .png"` and code `"invalid-screenshot-extension"`. |
| **Verification** | `Screenshot demo.png` → no error. `Screenshot demo.PNG` → no error (case-insensitive). `Screenshot demo.jpg` → Error. |

### DIA-014 — Unknown Built-In Theme Name

| Field | Value |
| --- | --- |
| **ID** | DIA-014 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | If a `SET_COMMAND` for `Theme` uses a string-like built-in theme value (`IDENT` or quoted `STRING`) and that value is not present in the built-in theme registry, the server SHOULD emit an `Error` diagnostic on the value token range with message `"Unknown VHS theme '{name}'"` and code `"unknown-theme"`. JSON theme objects MUST NOT be validated against the built-in registry. Valid bare identifiers such as `Set Theme Dracula` MUST NOT be reported as parse errors. |
| **Verification** | `Set Theme Dracula` → no parse or semantic error. `Set Theme "D"` → `unknown-theme` error. `Set Theme { "name": "Custom" }` → no `unknown-theme` error. |

## 5. Design Options Analysis

### 5.1 Diagnostic Pipeline Architecture

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Layered collectors** | Separate functions per diagnostic category (parse, semantic-lightweight, semantic-heavyweight); caller merges into single Vec before publishing | Clean separation; each layer testable independently; easy to add new rules | Must merge before publish; caller manages combination |
| **B: Single-pass walker** | One AST walk collects all diagnostics in a single pass | Efficient; single traversal | Mixes concerns; harder to test individual rules; heavyweight I/O mixed with pure analysis |
| **C: Rule engine** | Diagnostic rules registered as trait objects; engine runs all applicable rules | Extensible; pluggable; Phase 3 could add user-defined rules | Over-engineered for ~10 rules; dynamic dispatch overhead |

**Recommended: Option A (Layered collectors).** The diagnostic set naturally
splits into three layers with different timing characteristics (parse,
lightweight-semantic, heavyweight-semantic). Each layer is a pure function
`fn collect_xxx(tree: &SyntaxNode) -> Vec<Diagnostic>` that can be unit
tested independently. The LSP handler merges layers before publishing.
This follows Rust Best Practices: composition over abstraction when the
domain has clear orthogonal axes.

### 5.2 Heavyweight Check Execution Model

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Spawned task per didSave** | `tokio::spawn` a task that runs heavyweight checks, then calls `client.publish_diagnostics()` upon completion | Non-blocking; clean async model; natural cancellation via `JoinHandle` | Must handle stale results (document changed since spawn); task management |
| **B: Debounced background loop** | A background task polls a channel for save events, debounces, then runs checks | Automatic debouncing; single task | More complex setup; channel management; harder to test |
| **C: Blocking in didSave** | Run heavyweight checks synchronously in the `did_save` handler | Simplest | Blocks the LSP message loop; PATH lookups may take hundreds of milliseconds |

**Recommended: Option A (Spawned task per didSave).** Per Rust Async
Patterns skill: "Use `tokio::spawn` for concurrent tasks; use
`CancellationToken` for graceful shutdown." The spawned task captures
the current document state (AST + URI), runs async I/O checks
(`tokio::fs::metadata`, `tokio::process::Command` for which-style lookup),
and publishes results via the `Client` handle. DIA-012 ensures stale
tasks are cancelled.

### 5.3 $PATH Program Existence Check

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: `which` crate** | Use the `which` Rust crate to locate executables in `$PATH` | Cross-platform; handles Windows PATH; well-tested | Extra dependency |
| **B: Manual PATH iteration** | Read `$PATH`, split on `:`, check `fs::metadata` for each directory + program | Zero deps; transparent logic | Platform-specific (`;` on Windows); more code |
| **C: Shell subprocess** | Run `which <program>` via `tokio::process::Command` | Delegates to OS | Subprocess overhead; `which` not available on all systems |

**Recommended: Option A (`which` crate).** The `which` crate (5M+ downloads)
is the standard Rust solution for executable path resolution. It handles
platform differences (PATH separator, PATHEXT on Windows) and is
async-friendly when called from a spawned task. Per Rust Best Practices:
"prefer well-tested community crates over hand-rolled equivalents."

## 6. Severity Mapping

| Rule | Code | Severity | Category |
| --- | --- | --- | --- |
| Parse errors (Phase 1) | — | Error | Syntax |
| Missing Output directive | `missing-output` | Warning | Structural |
| Invalid Output extension | `invalid-extension` | Error | Value validation |
| Invalid Screenshot extension | `invalid-screenshot-extension` | Error | Value validation |
| Unknown built-in theme | `unknown-theme` | Error | Value validation |
| Duplicate Set | `duplicate-set` | Warning | Structural |
| Invalid hex color | `invalid-hex-color` | Error | Value validation |
| Numeric out of range | `value-out-of-range` | Error | Value validation |
| Require program not found | `require-not-found` | Warning | Environment |
| Source file not found | `source-not-found` | Warning | Environment |
| Safety findings (SPEC_SAFETY.md) | `safety-*` | See SPEC_SAFETY.md | Security |

## 7. Numeric Value Range Constraints

| Setting | Type | Min (exclusive) | Max | Notes |
| --- | --- | --- | --- | --- |
| FontSize | float | 0 | — | Must be > 0 |
| Framerate | integer | 0 | — | Must be > 0; typical: 24–60 |
| PlaybackSpeed | float | 0 | — | Must be > 0; typical: 0.5–5.0 |
| Height | integer | 0 | — | Must be > 0; pixel count |
| Width | integer | 0 | — | Must be > 0; pixel count |
| WindowBarSize | integer | 0 | — | Must be > 0; pixel count |
| LetterSpacing | float | — | — | Any value (can be negative for tight spacing) |
| LineHeight | float | 0 | — | Must be > 0 |
| Padding | float | 0 (inclusive) | — | Must be >= 0 |
| BorderRadius | integer | 0 (inclusive) | — | Must be >= 0 |
| Margin | integer | 0 (inclusive) | — | Must be >= 0 |

## 8. Diagnostic Timing Classification

| Rule | Timing | Rationale |
| --- | --- | --- |
| Missing Output directive | Lightweight (didChange) | Pure AST scan — O(n) children of SOURCE_FILE |
| Invalid Output extension | Lightweight (didChange) | String suffix check on PATH token |
| Invalid Screenshot extension | Lightweight (didChange) | String suffix check on PATH token |
| Duplicate Set | Lightweight (didChange) | Collect SET_COMMAND nodes, group by setting keyword |
| Invalid hex color | Lightweight (didChange) | Regex match on string token text |
| Numeric out of range | Lightweight (didChange) | Parse numeric token, compare against bounds |
| Require program not found | Heavyweight (didSave) | `$PATH` lookup requires filesystem I/O |
| Source file not found | Heavyweight (didSave) | `fs::metadata` call for file existence |
| Safety findings | Lightweight (didChange) | Pure string pattern matching on AST (see SPEC_SAFETY.md) |

## 9. Unified Pipeline Pseudocode

```rust
fn on_did_change(uri, source):
    parse = vhs_analyzer_core::parse(&source)
    store.insert(uri, DocumentState { source, parse })

    parse_diags   = map_parse_errors(parse.errors)
    semantic_diags = collect_lightweight_diagnostics(parse.syntax())
    safety_diags   = collect_safety_diagnostics(parse.syntax())

    // Preserve heavyweight results from last didSave
    heavyweight_diags = store.get_heavyweight_cache(uri)

    all = concat(parse_diags, semantic_diags, safety_diags, heavyweight_diags)
    client.publish_diagnostics(uri, all, version)

fn on_did_save(uri):
    state = store.get(uri)

    // Cancel any in-flight heavyweight task for this URI
    cancel_previous_heavyweight(uri)

    // Spawn async heavyweight checks
    let token = CancellationToken::new()
    store_cancellation_token(uri, token.clone())

    tokio::spawn(async move {
        heavyweight = collect_heavyweight_diagnostics(
            state.syntax(), uri, workspace_root, token
        ).await

        if !token.is_cancelled():
            store.set_heavyweight_cache(uri, heavyweight)

            // Re-publish combined diagnostics
            parse_diags    = map_parse_errors(state.errors)
            semantic_diags  = collect_lightweight_diagnostics(state.syntax())
            safety_diags    = collect_safety_diagnostics(state.syntax())
            all = concat(parse_diags, semantic_diags, safety_diags, heavyweight)
            client.publish_diagnostics(uri, all, version)
    })

fn on_did_close(uri):
    store.remove(uri)
    client.publish_diagnostics(uri, [], None)  // clear diagnostics
```

## 10. DocumentState Extension

Phase 2 extends the Phase 1 `DocumentState` (SPEC_LSP_CORE.md §6):

```rust
pub struct DocumentState {
    pub source: String,
    pub green: GreenNode,
    pub errors: Vec<ParseError>,
    // Phase 2 additions:
    pub heavyweight_diagnostics: Vec<Diagnostic>,
    pub heavyweight_task: Option<CancellationToken>,
}
```

The `heavyweight_diagnostics` field caches the latest heavyweight check
results. The `heavyweight_task` field holds the cancellation token for
the in-flight heavyweight task (if any).

## 11. Resolved Design Decisions

All Freeze Candidates from Stage A have been closed with definitive decisions.

### FC-DIA-01 — Output Directory Existence Check (RESOLVED: Do NOT Check)

**Decision:** The diagnostics engine MUST NOT check whether the parent
directory of the `Output` path exists in Phase 2.

**Rationale:** VHS v0.3.0+ automatically creates output directories
(verified via GitHub Issue #206). If the LSP reports "directory not found",
users would be confused because VHS would succeed at runtime. This would
be a false positive. Revisit if user feedback indicates demand.

### FC-DIA-02 — LoopOffset Percentage Validation (RESOLVED: Do NOT Validate)

**Decision:** The diagnostics engine MUST NOT validate `Set LoopOffset`
value ranges in Phase 2.

**Rationale:** The VHS documentation does not specify valid ranges for
LoopOffset. It accepts both absolute frame counts (e.g., `5`) and
percentages (e.g., `50%`). Edge values may have intentional effects.
In the absence of authoritative constraints from the VHS specification,
validation would risk false positives.

### FC-DIA-03 — Screenshot Path Extension Validation (RESOLVED: Validate `.png` Only)

**Decision:** The diagnostics engine MUST validate Screenshot path
extensions. Only `.png` is valid. This is implemented as DIA-013
(added to §4 during freeze).

**Rationale:** The VHS README (v0.11.0) explicitly states Screenshot
"captures the current frame (png format)". A PR (#635) proposing `.txt`
support has not been merged. The validation follows the same Error-level
pattern as DIA-004 for Output extensions. If VHS adds text screenshot
support in a future release, the valid extension set will be expanded.

### FC-DIA-04 — Heavyweight Diagnostic Debounce (RESOLVED: Cancellation-and-Restart Only)

**Decision:** Heavyweight diagnostics MUST use the cancellation-and-restart
approach (DIA-012) without additional debouncing.

**Rationale:** VHS `.tape` files are typically <200 lines. `$PATH` lookup
via the `which` crate takes ~5ms, `fs::metadata` takes ~1ms. The total
heavyweight check latency is well under 100ms. Cancellation-and-restart
(via `CancellationToken` or `JoinHandle::abort()`) already prevents stale
results. Debouncing would add complexity disproportionate to its benefit.
Per Rust Async Patterns: "Use `tokio::spawn` for concurrent tasks; use
`CancellationToken` for graceful shutdown."
