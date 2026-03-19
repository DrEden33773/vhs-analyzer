# SPEC_CLIENT.md — LSP Client Bootstrapping

**Phase:** 3 — VSCode Extension Client
**Work Stream:** WS-1 (Client)
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect
**Depends On:** phase1/SPEC_LSP_CORE.md (LSP binary, stdio transport, server capabilities), phase2/SPEC_COMPLETION.md (completionProvider), phase2/SPEC_DIAGNOSTICS.md (diagnostics pipeline)
**Last Updated:** 2026-03-20
**Frozen By:** Architect (Claude) — Stage B

---

> **CONTRACT_FROZEN** — This specification is frozen as of 2026-03-20.
> All Freeze Candidates have been resolved. No changes without explicit user approval.

---

## 1. Purpose

Define the VSCode extension client architecture: how the extension discovers,
launches, and communicates with the Rust LSP binary (`vhs-analyzer`);
the extension activation/deactivation lifecycle; the user-facing configuration
schema; runtime dependency detection; and the TextMate grammar for baseline
syntax highlighting. This is the foundation work stream — WS-2 (Preview) and
WS-3 (CodeLens) depend on a functioning LSP client.

## 2. Cross-Phase Dependencies

| Phase 1/2 Contract | Usage in This Spec |
| --- | --- |
| SPEC_LSP_CORE.md — LSP-001 (stdio transport) | Client MUST use stdio transport to match the server's stdin/stdout communication |
| SPEC_LSP_CORE.md — LSP-002 (Initialize handshake) | Client receives `InitializeResult` with server capabilities |
| SPEC_LSP_CORE.md — LSP-003 (Full sync) | Client MUST send `TextDocumentSyncKind.Full` content on every change |
| SPEC_LSP_CORE.md — LSP-005 (Shutdown/exit) | Client MUST send `shutdown` then `exit` on deactivation |
| SPEC_COMPLETION.md — CMP-001 (completionProvider) | Client receives completion capabilities; MUST NOT re-implement |
| SPEC_DIAGNOSTICS.md — DIA-011 (Unified pipeline) | Client receives `publishDiagnostics` notifications; MUST NOT re-implement |
| SPEC_SAFETY.md — SAF-006 (Safety diagnostics) | Client receives safety diagnostics with `safety/*` codes; MAY surface distinctly |
| SPEC_LEXER.md — §4 (Token kinds) | TextMate grammar maps VHS token kinds to TextMate scopes |

## 3. References

| Source | Role |
| --- | --- |
| [vscode-languageclient v9 (npm)](https://www.npmjs.com/package/vscode-languageclient) | `LanguageClient`, `ServerOptions`, `LanguageClientOptions` |
| [VSCode Language Server Guide](https://code.visualstudio.com/docs/extensions/example-language-server) | Official integration pattern |
| [rust-analyzer VSCode extension](https://rust-analyzer.github.io/book/vs_code.html) | Prior art: binary discovery, `server.path` setting, no-server fallback |
| [LSP 3.17 Specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/) | Protocol reference |
| TypeScript Expert skill | tsconfig strict mode, esbuild bundling, project architecture |

## 4. Requirements

### CLI-001 — Binary Discovery Chain

| Field | Value |
| --- | --- |
| **ID** | CLI-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST discover the LSP server binary using a layered resolution chain, evaluated in order: (1) User-configured path via `vhs-analyzer.server.path` setting, (2) bundled binary at `{extensionPath}/server/vhs-analyzer[.exe]` (platform-specific VSIX), (3) `vhs-analyzer` on system `$PATH`. If all three fail, the extension MUST enter no-server mode (CLI-009). Each resolution step MUST validate that the discovered file exists and is executable. |
| **Verification** | Set `vhs-analyzer.server.path` to a valid binary → client connects. Remove setting, install platform-specific VSIX → bundled binary used. Remove bundled binary, add to PATH → PATH binary used. Remove all → no-server mode activates. |

### CLI-002 — ServerOptions Configuration

| Field | Value |
| --- | --- |
| **ID** | CLI-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `ServerOptions` passed to `LanguageClient` MUST use `TransportKind.stdio` with the discovered binary path as `command`. The `args` field MUST include any user-configured extra arguments from `vhs-analyzer.server.args`. The `options.env` MUST inherit the current process environment. Run and debug configurations MUST be identical in Phase 3. |
| **Verification** | Client launches server via stdio; LSP initialize handshake succeeds; server capabilities match Phase 2 InitializeResult. |

### CLI-003 — LanguageClientOptions Configuration

| Field | Value |
| --- | --- |
| **ID** | CLI-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `LanguageClientOptions` MUST configure: (1) `documentSelector: [{ scheme: "file", language: "tape" }]`, (2) `synchronize.fileEvents: workspace.createFileSystemWatcher("**/*.tape")`, (3) `outputChannel` set to a dedicated "VHS Analyzer" output channel, (4) `traceOutputChannel` set to a separate "VHS Analyzer Trace" output channel for protocol tracing controlled by `vhs-analyzer.trace.server`. |
| **Verification** | Open a `.tape` file → server activates. Open a non-`.tape` file → no server activation. Trace setting changes → trace output appears in the dedicated channel. |

### CLI-004 — Extension Activation Lifecycle

| Field | Value |
| --- | --- |
| **ID** | CLI-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST activate on `onLanguage:tape`. The `activate()` function MUST: (1) Resolve the binary path (CLI-001). (2) If binary found: create `LanguageClient`, call `client.start()`, register disposables. (3) If no binary: enter no-server mode (CLI-009). (4) Run runtime dependency detection (CLI-007). The `deactivate()` function MUST call `client?.stop()` to send `shutdown` + `exit` per LSP-005. |
| **Verification** | Open `.tape` file → extension activates, server starts. Close all `.tape` files and reload window → server stops cleanly. |

### CLI-005 — Error Recovery and Auto-Restart

| Field | Value |
| --- | --- |
| **ID** | CLI-005 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The `LanguageClient` MUST be configured with an `errorHandler` that restarts the server on unexpected crashes. The restart strategy MUST use exponential backoff: 1s, 2s, 4s, 8s, with a maximum of 5 restart attempts within a 3-minute window. After exhausting retries, the extension MUST show an error notification with a "Restart Server" action button. The `initializationFailedHandler` MUST return `false` (do not auto-retry initialization failures) and show a diagnostic error message. |
| **Verification** | Kill the server process → client restarts within 1s. Kill 5 times rapidly → error notification appears. Click "Restart Server" → server restarts with fresh retry counter. |

### CLI-006 — Extension Configuration Schema

| Field | Value |
| --- | --- |
| **ID** | CLI-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST register the following settings under `contributes.configuration` with `title: "VHS Analyzer"`: (1) `vhs-analyzer.server.path` (type: `string`, default: `""`, description: custom LSP binary path), (2) `vhs-analyzer.server.args` (type: `array`, items: `string`, default: `[]`, description: extra server arguments), (3) `vhs-analyzer.trace.server` (type: `string`, enum: `["off", "messages", "verbose"]`, default: `"off"`, description: LSP protocol trace level), (4) `vhs-analyzer.preview.autoRefresh` (type: `boolean`, default: `true`, description: auto-refresh preview on file save), (5) `vhs-analyzer.codelens.enabled` (type: `boolean`, default: `true`, description: show inline run buttons). Changing `server.path` or `server.args` MUST trigger a server restart. |
| **Verification** | Each setting appears in VSCode Settings UI. Changing `server.path` restarts the server. Changing `codelens.enabled` toggles CodeLens visibility. |

### CLI-007 — Runtime Dependency Detection

| Field | Value |
| --- | --- |
| **ID** | CLI-007 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | On activation, the extension SHOULD check the system `$PATH` for: `vhs`, `ttyd`, `ffmpeg`. For each missing dependency, the extension SHOULD show an `Information` message: `"{name} not found. Preview and Run features require {name}. [Install]"`. The "Install" button SHOULD open the relevant installation URL in the default browser. This check MUST be non-blocking (run asynchronously after client start). Missing dependencies MUST NOT prevent LSP features from working. |
| **Verification** | Remove `vhs` from PATH → information message appears on activation. Install `vhs` and reload → no message. Click "Install" → browser opens VHS installation page. |

### CLI-008 — Language Contribution (TextMate Grammar)

| Field | Value |
| --- | --- |
| **ID** | CLI-008 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST register a language contribution in `package.json`: `{ id: "tape", extensions: [".tape"], aliases: ["VHS Tape", "tape"], configuration: "./language-configuration.json" }`. The extension MUST provide a TextMate grammar (`syntaxes/tape.tmLanguage.json`) that maps VHS keywords, settings, literals, and comments to standard TextMate scopes (see §9). The grammar provides baseline syntax highlighting independent of the LSP server and is the sole highlighting source in no-server mode. |
| **Verification** | Open a `.tape` file in VSCode → syntax highlighting appears immediately (before server starts). Keywords are colored as `keyword.control`, strings as `string.quoted`, comments as `comment.line`. |

### CLI-009 — No-Server Fallback Mode

| Field | Value |
| --- | --- |
| **ID** | CLI-009 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | When no LSP binary is available (CLI-001 chain exhausted), the extension MUST operate in no-server mode. In this mode: (1) TextMate grammar provides syntax highlighting (CLI-008). (2) CodeLens "▶ Run this tape" buttons remain functional (VHS CLI invocation does not require the LSP server). (3) Preview Webview remains functional. (4) Completion, diagnostics, hover, and formatting are NOT available. (5) The extension MUST show a one-time information message: `"VHS Analyzer LSP server not found. Install the platform-specific extension for full language support. [Install] [Don't show again]"`. |
| **Verification** | Install universal VSIX (no bundled binary), no server on PATH → syntax highlighting works, CodeLens works, completion/hover/diagnostics do not. Information message appears once. |

### CLI-010 — Configuration Change Handling

| Field | Value |
| --- | --- |
| **ID** | CLI-010 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | The extension SHOULD listen to `workspace.onDidChangeConfiguration` for changes in the `vhs-analyzer` section. Changes to `server.path` or `server.args` SHOULD trigger a graceful server restart (stop + re-discover binary + start). Changes to `trace.server` SHOULD update the trace level without restarting. Changes to `preview.autoRefresh` and `codelens.enabled` SHOULD take effect immediately without restart. |
| **Verification** | Change `server.path` in settings → server restarts with new binary. Change `trace.server` → trace level changes immediately. |

### CLI-011 — Status Bar Indicator

| Field | Value |
| --- | --- |
| **ID** | CLI-011 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | The extension SHOULD display a status bar item showing the current server status: (1) "VHS $(check)" (green) when server is running, (2) "VHS $(warning)" (yellow) during startup/restart, (3) "VHS $(error)" (red) when server failed/no-server mode, (4) "VHS $(sync~spin)" during VHS CLI execution. Clicking the status bar item SHOULD show a quick pick with actions: "Restart Server", "Show Output", "Show Trace". |
| **Verification** | Server running → green indicator. Kill server → yellow then green (auto-restart). No binary → red indicator. Click indicator → quick pick appears. |

## 5. Design Options Analysis

### 5.1 Binary Discovery Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Layered chain (user → bundled → PATH → no-server)** | Evaluate each source in priority order; first valid binary wins | Maximum flexibility; matches rust-analyzer pattern; supports both platform VSIX and manual install | Slightly more complex discovery logic |
| **B: Bundled only + user override** | Only bundled binary or user-configured path; no PATH fallback | Simpler; deterministic | Breaks for users who build from source or install via cargo |
| **C: Download on activation** | Extension downloads the binary from GitHub Releases at first activation | Always up-to-date; works with universal VSIX | Network dependency; trust concerns; download latency; complex update logic |

**Recommended: Option A (Layered chain).** This follows the rust-analyzer precedent
(user setting → extension bundled → PATH). It supports all installation methods:
platform-specific VSIX (bundled), `cargo install` (PATH), and custom builds
(user setting). The no-server fallback ensures the extension always provides value.
Option C (download on activation) adds network dependency and trust complexity
that is not justified for Phase 3 launch scope.

### 5.2 Error Recovery Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Exponential backoff with cap** | 1s, 2s, 4s, 8s delays; max 5 retries in 3 minutes | Prevents restart storms; self-healing; standard practice | Retry window tracking adds minor state |
| **B: Fixed delay** | Always wait 3s between restarts; max 5 retries | Simpler | May be too slow for transient failures or too fast for persistent ones |
| **C: Immediate restart, no limit** | Restart immediately on crash, forever | Simplest; always tries | Restart storm on persistent crash; CPU thrashing |

**Recommended: Option A (Exponential backoff with cap).** Language servers
may crash due to transient conditions (malformed input, memory pressure).
Exponential backoff prevents restart storms while ensuring quick recovery
for one-off crashes. The 3-minute window prevents indefinite retry accumulation.
This matches the `vscode-languageclient` built-in `CloseAction.Restart`
pattern, extended with backoff tracking.

### 5.3 TextMate Grammar Scope

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Full grammar matching Phase 1 lexer** | Map all 63 token kinds to TextMate scopes | Complete highlighting; consistency with LSP semantic tokens | Large grammar file; some scopes may not have visual distinction |
| **B: Minimal grammar (keywords + strings + comments)** | Only keywords, string/regex/JSON literals, and comments | Simple; fast; covers 90% of visual needs | Missing fine-grained highlighting for setting names, paths, time literals |
| **C: Tree-sitter grammar reuse** | Adapt charmbracelet/tree-sitter-vhs as TextMate source | Leverages existing grammar | tree-sitter → TextMate conversion is lossy; tree-sitter-vhs is out of date |

**Recommended: Option A (Full grammar matching Phase 1 lexer).** The TextMate
grammar is the ONLY highlighting source in no-server mode and is always active
as baseline highlighting. A full grammar ensures consistency with the LSP
lexer's token classification. The VHS token set (63 kinds) maps cleanly to
standard TextMate scopes without excessive complexity.

### 5.4 Runtime Dependency Check Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Client-side PATH check on activation** | `child_process.exec("which vhs")` or equivalent | Non-blocking; immediate feedback; no server dependency | Platform-specific `which` vs `where` handling |
| **B: Server-side check via custom LSP notification** | Server checks PATH and sends custom notification to client | Reuses Rust `which` crate; cross-platform | Requires server running; custom notification protocol |
| **C: Deferred check on first VHS execution** | Check only when user tries to run/preview | No startup cost | Poor UX — user discovers missing deps only at execution time |

**Recommended: Option A (Client-side PATH check on activation).** This is
a client-side concern — the extension needs to know if VHS CLI is available
for Preview and CodeLens features, regardless of whether the LSP server is
running. Using Node.js `child_process.execFile("which", ["vhs"])` (Unix) or
`child_process.execFile("where", ["vhs"])` (Windows) is lightweight and
non-blocking. The check runs asynchronously after `client.start()` to avoid
blocking activation. This complements the server-side `Require` diagnostics
from Phase 2 (SPEC_DIAGNOSTICS.md DIA-008).

## 6. Binary Discovery Algorithm (Pseudocode)

```typescript
async function discoverServerBinary(): Promise<string | null>
    // Step 1: User-configured path
    const userPath = config.get("vhs-analyzer.server.path")
    if userPath and await isExecutable(userPath):
        return userPath

    // Step 2: Bundled binary (platform-specific VSIX)
    const ext = process.platform === "win32" ? ".exe" : ""
    const bundledPath = path.join(context.extensionPath, "server", `vhs-analyzer${ext}`)
    if await isExecutable(bundledPath):
        return bundledPath

    // Step 3: System PATH
    const pathBinary = await which("vhs-analyzer").catch(() => null)
    if pathBinary:
        return pathBinary

    // Step 4: No-server mode
    return null
```

## 7. Extension Activation Sequence

```typescript
export async function activate(context: ExtensionContext):
    // 1. Register language contribution (TextMate grammar registered via package.json)

    // 2. Discover LSP binary
    const serverPath = await discoverServerBinary()

    // 3. Start LSP client (if binary found)
    if serverPath:
        const serverOptions: ServerOptions = {
            command: serverPath,
            args: config.get("vhs-analyzer.server.args") ?? [],
            transport: TransportKind.stdio,
            options: { env: process.env }
        }
        const clientOptions: LanguageClientOptions = {
            documentSelector: [{ scheme: "file", language: "tape" }],
            synchronize: { fileEvents: workspace.createFileSystemWatcher("**/*.tape") },
            outputChannel: window.createOutputChannel("VHS Analyzer"),
            traceOutputChannel: window.createOutputChannel("VHS Analyzer Trace"),
        }
        client = new LanguageClient("vhs-analyzer", "VHS Analyzer", serverOptions, clientOptions)
        client.start()
    else:
        enterNoServerMode()

    // 4. Register CodeLens provider (WS-3, works in both modes)
    // 5. Register preview commands (WS-2, works in both modes)
    // 6. Register status bar item (CLI-011)
    // 7. Check runtime dependencies (CLI-007, async, non-blocking)
    checkRuntimeDependencies()

export async function deactivate():
    await client?.stop()
```

## 8. Extension Configuration Schema (package.json fragment)

```json
{
  "contributes": {
    "configuration": {
      "title": "VHS Analyzer",
      "properties": {
        "vhs-analyzer.server.path": {
          "type": "string",
          "default": "",
          "markdownDescription": "Absolute path to the `vhs-analyzer` binary. Leave empty to use the bundled binary or `$PATH`.",
          "scope": "machine-overridable"
        },
        "vhs-analyzer.server.args": {
          "type": "array",
          "items": { "type": "string" },
          "default": [],
          "description": "Extra arguments passed to the LSP server binary.",
          "scope": "machine-overridable"
        },
        "vhs-analyzer.trace.server": {
          "type": "string",
          "enum": ["off", "messages", "verbose"],
          "default": "off",
          "description": "Trace level for LSP protocol communication.",
          "scope": "window"
        },
        "vhs-analyzer.preview.autoRefresh": {
          "type": "boolean",
          "default": true,
          "description": "Automatically refresh preview when the output file changes.",
          "scope": "resource"
        },
        "vhs-analyzer.codelens.enabled": {
          "type": "boolean",
          "default": true,
          "description": "Show inline ▶ Run buttons above tape files.",
          "scope": "resource"
        }
      }
    }
  }
}
```

## 9. TextMate Grammar Scope Mapping

| VHS Token Category | TextMate Scope | Examples |
| --- | --- | --- |
| Command keywords | `keyword.control.tape` | `Output`, `Type`, `Sleep`, `Enter`, `Set` |
| Modifier keywords | `keyword.operator.modifier.tape` | `Ctrl`, `Alt`, `Shift` |
| Setting name keywords | `support.type.property-name.tape` | `FontSize`, `Theme`, `Shell` |
| Wait scope keywords | `keyword.other.tape` | `Screen`, `Line` |
| String literals | `string.quoted.double.tape` / `string.quoted.single.tape` / `string.quoted.backtick.tape` | `"hello"`, `'world'` |
| Regex literals | `string.regexp.tape` | `/pattern/` |
| JSON literals | `meta.embedded.json.tape` | `{ "name": "Dracula" }` |
| Integer literals | `constant.numeric.integer.tape` | `42`, `14` |
| Float literals | `constant.numeric.float.tape` | `3.14`, `.5` |
| Time literals | `constant.numeric.time.tape` | `500ms`, `2s` |
| Boolean literals | `constant.language.boolean.tape` | `true`, `false` |
| Path literals | `string.unquoted.path.tape` | `demo.gif`, `./out/video.mp4` |
| Comments | `comment.line.number-sign.tape` | `# this is a comment` |
| `@` punctuation | `punctuation.definition.annotation.tape` | `@` |
| `+` punctuation | `punctuation.separator.modifier.tape` | `+` |
| `%` punctuation | `punctuation.definition.percent.tape` | `%` |
| Identifiers | `variable.other.tape` | bare words not matching keywords |

## 10. Language Configuration (language-configuration.json)

```json
{
  "comments": {
    "lineComment": "#"
  },
  "brackets": [
    ["{", "}"],
    ["[", "]"],
    ["(", ")"]
  ],
  "autoClosingPairs": [
    { "open": "\"", "close": "\"" },
    { "open": "'", "close": "'" },
    { "open": "`", "close": "`" },
    { "open": "{", "close": "}" },
    { "open": "/", "close": "/" }
  ],
  "surroundingPairs": [
    ["\"", "\""],
    ["'", "'"],
    ["`", "`"]
  ],
  "wordPattern": "[A-Za-z][A-Za-z0-9_]*"
}
```

## 11. Resolved Design Decisions

> All Freeze Candidates resolved through collaborative Architect–Orchestrator
> discussion on 2026-03-20.

### RD-CLI-01 — Binary Name Convention

**Decision:** The bundled binary MUST be named `vhs-analyzer` (not
`vhs-analyzer-lsp`).

**Rationale:** Follows the rust-analyzer precedent — the binary is named
`rust-analyzer`, not `rust-analyzer-lsp`. Verified via `cargo install
rust-analyzer` that crates.io hosts only a placeholder library crate (v0.0.1)
with no binaries, confirming no separate CLI tool exists. `vhs-analyzer` has
no standalone CLI tool and none is planned. The `-lsp` suffix adds no value
since the binary's sole purpose is to serve as an LSP server. Builder MUST
configure the Cargo `[[bin]]` target to produce `vhs-analyzer` as the output
binary name.

### RD-CLI-02 — TextMate Grammar Source Format

**Decision:** The TextMate grammar MUST be authored directly as JSON
(`syntaxes/tape.tmLanguage.json`).

**Rationale:** JSON is directly consumed by VSCode without build conversion.
The VHS grammar is moderately sized (~150 rules). JSON eliminates a
YAML-to-JSON build dependency and matches the standard practice of official
VSCode language extensions (TypeScript, Python, rust-analyzer).

### RD-CLI-03 — Server Restart on Setting Change

**Decision:** Changing `vhs-analyzer.server.path` or
`vhs-analyzer.server.args` MUST trigger an automatic server restart with a
single attempt. If the new path is invalid, the extension MUST show an error
notification with a "Restart Server" action button and remain in stopped state.

**Rationale:** Auto-restart with single attempt provides better UX than
manual restart (the rust-analyzer approach). The user modifying `server.path`
typically has a valid binary ready. On failure, the error notification with
action button gives the user explicit control for retry.

### RD-CLI-04 — No-Server Mode Notification Suppression

**Decision:** The "LSP server not found" notification MUST include a "Don't
show again" button. Clicking it MUST set a `globalState` flag
(`"noServerMessageDismissed": true`) to permanently suppress the notification.

**Rationale:** This is the standard VSCode extension pattern for one-time
notifications. `globalState` is scoped to the extension and survives restarts.
Explicit user opt-out is more respectful than per-session or auto-suppress
behavior.

### RD-CLI-05 — Platform Detection for Runtime Dependency Check

**Decision:** The runtime dependency check (CLI-007) MUST use the `which` npm
package (v6.x) for cross-platform executable path resolution.

**Rationale:** `which` (npm/node-which, 187M+ weekly downloads) is the de
facto standard for cross-platform executable resolution in Node.js. It handles
Windows PATHEXT, symlinks, and edge cases. esbuild inlines it into the bundle,
so there is no runtime `node_modules` dependency.

### Cross-Phase Note — `--stdio` Flag Compatibility

`vscode-languageclient` v9 automatically appends `--stdio` to the server
arguments when using `TransportKind.stdio`. The Phase 1 `vhs-analyzer` binary
SHOULD accept `--stdio` as a no-op flag (the default transport is already
stdio per LSP-001). Builder MUST verify this compatibility before Phase 3
integration testing.
