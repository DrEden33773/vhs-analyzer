# SPEC_TEST_MATRIX.md — Phase 3 Acceptance Tests

**Phase:** 3 — VSCode Extension Client
**Status:** CONTRACT_FROZEN
**Last Updated:** 2026-03-20

---

## 1. Testing Boundary Guidance

- **Unit tests (Vitest):** Mock `vscode` API, test pure logic — binary
  discovery, message protocol serialization, CodeLens computation,
  configuration schema validation, Output-path regex, execution state machine.
- **Integration tests:** Test with real LSP binary using
  `vscode-languageclient` in a test harness — activate, send request, verify
  response.
- **E2E tests (`@vscode/test-electron`):** OPTIONAL (MAY). Mark as MAY in
  scenarios below.

## 2. Client Tests (T-CLI)

| ID | Description | Input | Expected Output | Level |
| --- | --- | --- | --- | --- |
| T-CLI-001 | Extension activates on `.tape` file | Open a `.tape` file in VSCode | Extension activates, `activate()` called, status bar shows green indicator | Integration |
| T-CLI-002 | Binary discovery — user-configured path | Set `vhs-analyzer.server.path` to valid binary path | Client connects via that binary; LSP handshake succeeds | Unit + Integration |
| T-CLI-003 | Binary discovery — bundled binary | Install platform-specific VSIX; clear `server.path` | Client finds `{extensionPath}/server/vhs-analyzer[.exe]` and connects | Integration |
| T-CLI-004 | Binary discovery — system PATH | Remove bundled binary; add `vhs-analyzer` to PATH | Client discovers PATH binary and connects | Integration |
| T-CLI-005 | Binary discovery — no binary found | Remove all binary sources | Extension enters no-server mode (CLI-009); red status bar | Unit + Integration |
| T-CLI-006 | LSP Initialize handshake | Valid binary available | `InitializeResult` contains completionProvider, hoverProvider, diagnostics capabilities from Phase 2 | Integration |
| T-CLI-007 | Configuration schema in Settings UI | Open VSCode Settings, search "VHS Analyzer" | All 5 settings visible: `server.path`, `server.args`, `trace.server`, `preview.autoRefresh`, `codelens.enabled` | E2E (MAY) |
| T-CLI-008 | Config change triggers server restart | Change `server.path` to another valid binary | Old server stops, new server starts; handshake succeeds | Unit |
| T-CLI-009 | Config change — invalid path | Change `server.path` to nonexistent file | Error notification with "Restart Server" action; server remains stopped | Unit |
| T-CLI-010 | Trace level change without restart | Change `trace.server` from `"off"` to `"verbose"` | Trace output appears in "VHS Analyzer Trace" channel; server not restarted | Integration |
| T-CLI-011 | Runtime dependency detection — VHS missing | Remove `vhs` from PATH | Information message: "vhs not found" with Install button | Unit |
| T-CLI-012 | Runtime dependency detection — all present | `vhs`, `ttyd`, `ffmpeg` on PATH | No dependency warning messages | Unit |
| T-CLI-013 | TextMate grammar provides syntax highlighting | Open `.tape` file (no server) | Keywords colored as `keyword.control`, strings as `string.quoted`, comments as `comment.line` | E2E (MAY) |
| T-CLI-014 | No-server mode — LSP features unavailable | Enter no-server mode | Completion, diagnostics, hover, formatting return empty/null | Unit |
| T-CLI-015 | No-server mode — notification with "Don't show again" | No binary available; first activation | Information message with [Install] and [Don't show again] buttons | Unit |
| T-CLI-016 | No-server mode — notification suppressed | Click "Don't show again"; reactivate | No notification shown; `globalState` flag is `true` | Unit |
| T-CLI-017 | Crash recovery — exponential backoff | Kill server process once | Client restarts server within 1s | Integration |
| T-CLI-018 | Crash recovery — retry exhaustion | Kill server 5 times within 3 minutes | Error notification with "Restart Server" action button | Unit |
| T-CLI-019 | Graceful shutdown | Call `deactivate()` | `client.stop()` sends `shutdown` then `exit` per LSP-005 | Unit |
| T-CLI-020 | Status bar indicator states | Various server states | Green (running), yellow (starting), red (failed/no-server), spinner (VHS executing) | Unit |
| T-CLI-021 | `which` package resolves executables cross-platform | Call dependency check on various platforms | Correct binary paths returned; PATHEXT handled on Windows | Unit |
| T-CLI-022 | Targeted suggest after `Set Theme` trailing space | Type the trailing space in `Set Theme` | Extension executes `editor.action.triggerSuggest` in explicit mode | Unit |
| T-CLI-023 | Targeted suggest inside empty Theme quotes | Type `"` or `'` to produce `Set Theme ""` / `Set Theme ''` with cursor between quotes | Extension executes `editor.action.triggerSuggest` in explicit mode | Unit |
| T-CLI-023A | Targeted suggest while typing inside quoted Theme value | Type `D` in `Set Theme "D"` and type a space in `Set Theme "Catppuccin "` | Extension executes `editor.action.triggerSuggest` in explicit mode for both edits | Unit |
| T-CLI-024 | Targeted suggest after first and subsequent time digits | Type the first digit in `Sleep 1`, then continue to `Sleep 10`, and repeat for `Type@10` / `Set TypingSpeed 10` | Extension executes `editor.action.triggerSuggest` in explicit mode for the duration-slot digit edits | Unit |
| T-CLI-024A | Targeted suggest after partial time suffix characters | Type `m` or `s` in `Sleep 1000m`, `Type@1000m`, or `Set TypingSpeed 1000m` | Extension executes `editor.action.triggerSuggest` in explicit mode for the duration-slot suffix edits | Unit |

## 3. Preview Tests (T-PRV)

| ID | Description | Input | Expected Output | Level |
| --- | --- | --- | --- | --- |
| T-PRV-001 | Preview panel opens beside editor | Trigger `vhs-analyzer.previewTape` | Webview panel opens in `ViewColumn.Beside` with title "VHS Preview: {filename}" | Unit |
| T-PRV-002 | Same-file preview reuses panel | Trigger preview twice for same file | Existing panel reveals; no duplicate created | Unit |
| T-PRV-003 | VHS CLI invocation succeeds | Valid tape file with `Output demo.gif` | `spawn("vhs", [filePath])` called; exit code 0; `renderComplete` sent | Unit |
| T-PRV-004 | Output path — unquoted | File contains `Output demo.gif` | Regex extracts `demo.gif` | Unit |
| T-PRV-005 | Output path — quoted | File contains `Output "path with spaces.gif"` | Regex extracts `path with spaces.gif` (quotes stripped) | Unit |
| T-PRV-006 | Output path — no Output directive | File has no `Output` line | Default to `out.gif` in working directory | Unit |
| T-PRV-006A | Output path — nested tape with relative Output | Tape is in a nested directory, `Output demo.gif` | Preview targets the artifact relative to the tape directory | Unit |
| T-PRV-007 | Auto-refresh on output file change | `autoRefresh` enabled; output file changes externally | `renderComplete` with cache-busting URI sent to Webview | Unit |
| T-PRV-008 | Auto-refresh disabled | Set `preview.autoRefresh` to `false` | File watcher not active; no auto-refresh on external change | Unit |
| T-PRV-009 | Cancellation via Webview button | Click "Cancel" in Webview during VHS execution | SIGTERM sent to VHS process; `renderError` with `cancelled: true` | Unit |
| T-PRV-010 | Cancellation via panel close | Close preview panel during execution | VHS process terminated; resources disposed | Unit |
| T-PRV-011 | New execution cancels running one | Trigger run while VHS is executing | Previous process receives SIGTERM; new process starts | Unit |
| T-PRV-012 | SIGKILL fallback after SIGTERM timeout | VHS process ignores SIGTERM for 3+ seconds | SIGKILL sent after 3s timeout | Unit |
| T-PRV-013 | CSP enforcement | Webview HTML loaded | CSP meta tag present; external resource loading blocked | Unit |
| T-PRV-014 | Theme-aware styling | Switch VSCode theme | `themeChange` message sent; Webview updates background/foreground | Unit |
| T-PRV-015 | Loading state display | VHS execution in progress | Spinner visible; progress text shows raw VHS stderr lines | Unit |
| T-PRV-016 | Error state display | VHS exits with non-zero code | Error message shown; "Retry" button visible | Unit |
| T-PRV-017 | VHS missing graceful degradation | `vhs` not on PATH; trigger preview | "VHS is not installed" message with "Install VHS" link | Unit |
| T-PRV-018 | GIF display | VHS produces `.gif` output | `<img>` element with `asWebviewUri` source | Unit |
| T-PRV-019 | MP4/WebM display | VHS produces `.mp4` output | `<video controls autoplay loop>` element | Unit |
| T-PRV-020 | Messaging protocol — all message types | Various render lifecycle events | Correct discriminated union messages sent/received with proper `type` field | Unit |

## 4. CodeLens Tests (T-CLS)

| ID | Description | Input | Expected Output | Level |
| --- | --- | --- | --- | --- |
| T-CLS-001 | File-level CodeLens at first non-trivial line | File starts with `# comment` then `Output demo.gif` | CodeLens appears above `Output` line (not above comment) | Unit |
| T-CLS-002 | File-level CodeLens — no leading comments | File starts with `Output demo.gif` | CodeLens at line 0 | Unit |
| T-CLS-003 | Output-level CodeLens placement | File with 2 `Output` directives | 2 additional CodeLenses above each `Output` line, showing `▶ Preview {filename}` | Unit |
| T-CLS-004 | No Output directive | File with no `Output` line | Only file-level CodeLenses (2); no Output-level lenses | Unit |
| T-CLS-005 | Command registration | Extension activated | `vhs-analyzer.runTape`, `vhs-analyzer.previewTape`, `vhs-analyzer.stopRunning` in Command Palette | Integration |
| T-CLS-006 | runTape executes VHS | Trigger `runTape` on valid tape file | VHS process spawned; output in "VHS Analyzer: Run" channel | Unit |
| T-CLS-007 | previewTape opens preview | Trigger `previewTape` on valid tape file | VHS executes AND preview panel opens | Unit |
| T-CLS-008 | stopRunning cancels execution | Trigger `stopRunning` while VHS is running | SIGTERM sent; state transitions to Idle | Unit |
| T-CLS-009 | Execution state — Idle → Running | Trigger `runTape` from Idle | State becomes `Running`; `onStateChange` fires | Unit |
| T-CLS-010 | Execution state — Running → Complete | VHS exits code 0 | State becomes `Complete`; `onStateChange` fires | Unit |
| T-CLS-011 | Execution state — Running → Error | VHS exits non-zero | State becomes `Error`; `onStateChange` fires | Unit |
| T-CLS-012 | Single-execution-per-file | Trigger `runTape` while already Running | Previous execution cancelled; new one starts | Unit |
| T-CLS-013 | Status bar progress | VHS execution in progress | Status bar shows `$(sync~spin) VHS: Running {filename}...` | Unit |
| T-CLS-014 | Status bar completion flash | VHS completes successfully | Status bar shows `$(check) VHS: Done` for 3 seconds | Unit |
| T-CLS-015 | CodeLens toggle — disabled | Set `codelens.enabled` to `false` | `provideCodeLenses` returns empty array | Unit |
| T-CLS-016 | CodeLens toggle — re-enabled | Set `codelens.enabled` back to `true` | CodeLenses reappear; `onDidChangeCodeLenses` fires | Unit |
| T-CLS-017 | Dynamic CodeLens titles | Execution state changes | Title updates: "▶ Run this tape" (Idle) → "$(sync~spin) Running..." (Running) | Unit |
| T-CLS-018 | Context menu — `.tape` file | Right-click in `.tape` editor | "VHS: Run Tape" and "VHS: Run & Preview" visible | E2E (MAY) |
| T-CLS-019 | Context menu — non-`.tape` file | Right-click in `.js` editor | VHS menu items not visible | E2E (MAY) |
| T-CLS-020 | Context variable `isRunning` | Start/stop VHS execution | `vhs-analyzer.isRunning` context toggles; Stop command visibility updates | Unit |
| T-CLS-021 | Multiple Outputs — file-level shows first | File with `Output a.gif` then `Output b.mp4` | File-level "Run & Preview" opens preview for `a.gif` | Unit |

## 5. Packaging Tests (T-PKG)

| ID | Description | Input | Expected Output | Level |
| --- | --- | --- | --- | --- |
| T-PKG-001 | VSIX build — win32-x64 | CI matrix with `x86_64-pc-windows-msvc` binary | VSIX contains `server/vhs-analyzer.exe`; `vsce ls` confirms | CI |
| T-PKG-002 | VSIX build — darwin-arm64 | CI matrix with `aarch64-apple-darwin` binary | VSIX contains `server/vhs-analyzer`; macOS ARM64 binary | CI |
| T-PKG-003 | VSIX build — darwin-x64 | CI matrix with `x86_64-apple-darwin` binary | VSIX contains `server/vhs-analyzer`; macOS Intel binary | CI |
| T-PKG-004 | VSIX build — linux-x64 | CI matrix with `x86_64-unknown-linux-gnu` binary | VSIX contains `server/vhs-analyzer`; Linux x64 binary | CI |
| T-PKG-005 | VSIX build — linux-arm64 | CI matrix with `aarch64-unknown-linux-gnu` binary | VSIX contains `server/vhs-analyzer`; Linux ARM64 binary | CI |
| T-PKG-006 | VSIX build — alpine-x64 | CI matrix with `x86_64-unknown-linux-musl` binary | VSIX contains `server/vhs-analyzer`; musl-linked binary | CI |
| T-PKG-007 | Universal VSIX — no binary | `vsce package --no-dependencies` (no `--target`) | VSIX does NOT contain `server/` directory | CI |
| T-PKG-008 | Binary inclusion verification | Unzip platform VSIX | `server/vhs-analyzer` present, executable, correct architecture (`file` command) | CI |
| T-PKG-009 | Binary size check | Release build with `opt-level = "s"`, LTO, strip | Binary size < 15 MB per platform | CI |
| T-PKG-010 | esbuild bundle — single file | Run `pnpm run build` | `dist/extension.js` exists; single file; no `node_modules` at runtime | CI |
| T-PKG-011 | Bundle size check | Inspect `dist/extension.js` | Bundle size < 500 KB | CI |
| T-PKG-012 | CI lint pipeline | Run `pnpm run lint` (`biome check .`) | Zero errors | CI |
| T-PKG-013 | CI typecheck pipeline | Run `pnpm run typecheck` (`tsc --noEmit`) | Zero errors | CI |
| T-PKG-014 | CI test pipeline | Run `pnpm run test` (`vitest run`) | All tests pass | CI |
| T-PKG-015 | `.vscodeignore` excludes dev files | Inspect VSIX contents | No `src/`, `node_modules/`, `*.ts`, `tsconfig.json`, `biome.json` in VSIX | CI |
| T-PKG-016 | Pre-release tag detection | Push tag `v0.1.0-beta.1` | CI publishes with `--pre-release` flag | CI |
| T-PKG-017 | Stable tag detection | Push tag `v0.1.0` | CI publishes as stable release | CI |
| T-PKG-018 | Dual publishing — Marketplace + Open VSX | Release workflow completes | VSIX published to both registries via `vsce publish` and `npx ovsx publish` | CI |
| T-PKG-019 | GitHub Release assets | Release workflow completes | All 7 VSIX files attached as release assets | CI |
| T-PKG-020 | pnpm version consistency | CI environment | `pnpm --version` outputs `10.32.1` (matches `packageManager` field) | CI |

## 6. Integration Tests (T-INT3)

| ID | Description | Input | Expected Output | Level |
| --- | --- | --- | --- | --- |
| T-INT3-001 | Full activation → LSP handshake → hover | Open `.tape` file with bundled binary | Extension activates, server connects, hover on `Type` shows documentation | Integration |
| T-INT3-002 | CodeLens run → Preview shows result | Click "▶ Run & Preview" CodeLens | VHS executes; preview panel opens; GIF displayed | Integration |
| T-INT3-003 | No-server mode → CodeLens + Preview work | Install universal VSIX (no binary) | Syntax highlighting works; CodeLens run triggers VHS; preview works; completion/hover do NOT work | Integration |
| T-INT3-004 | Platform VSIX install → bundled binary | Install platform-specific VSIX | Binary discovered at `server/vhs-analyzer`; full LSP features available | E2E (MAY) |
| T-INT3-005 | Universal VSIX → no-server mode | Install universal VSIX; no `vhs-analyzer` on PATH | Extension enters no-server mode; information message shown | E2E (MAY) |

## 7. Coverage Summary

| Domain | Unit | Integration | E2E (MAY) | CI | Total |
| --- | --- | --- | --- | --- | --- |
| Client (T-CLI) | 14 | 6 | 1 | 0 | 21 |
| Preview (T-PRV) | 19 | 0 | 0 | 1 | 20 |
| CodeLens (T-CLS) | 17 | 1 | 2 | 0 | 21 (1 overlap) |
| Packaging (T-PKG) | 0 | 0 | 0 | 20 | 20 |
| Integration (T-INT3) | 0 | 3 | 2 | 0 | 5 |
| **Total** | **50** | **10** | **5** | **21** | **87** |
