# SPEC_CODELENS.md — CodeLens & Commands

**Phase:** 3 — VSCode Extension Client
**Work Stream:** WS-3 (CodeLens)
**Status:** Stage B (CONTRACT_FROZEN)
**Owner:** Architect
**Depends On:** WS-1 (SPEC_CLIENT.md — extension activation, configuration schema), WS-2 (SPEC_PREVIEW.md — preview panel, VHS execution)
**Last Updated:** 2026-03-20
**Frozen By:** Architect (Claude) — Stage B

---

> **CONTRACT_FROZEN** — This specification is frozen as of 2026-03-20.
> All Freeze Candidates have been resolved. No changes without explicit user approval.

---

## 1. Purpose

Define the CodeLens provider that places inline run buttons above tape file
directives, the command registry for VHS execution, the execution state
machine, and the integration between CodeLens triggers and the Preview panel.
CodeLens is a client-side feature — it does NOT require the LSP server and
MUST work in no-server mode (SPEC_CLIENT.md CLI-009).

## 2. Cross-Phase Dependencies

| Phase 1/2 Contract | Usage in This Spec |
| --- | --- |
| SPEC_PARSER.md — §4 (Node Kind Enumeration) | `OUTPUT_COMMAND` node positions inform CodeLens placement at Output directives |
| SPEC_PARSER.md — §5 (Ungrammar: OutputCommand) | `OutputCommand = 'Output' Path` — CodeLens extracts the output path for display |
| SPEC_LEXER.md — §4 (Token kinds: OUTPUT_KW) | CodeLens regex-based fallback matches `Output` keyword at line start |

| Phase 3 Spec | Integration |
| --- | --- |
| SPEC_CLIENT.md — CLI-006 (Configuration) | `vhs-analyzer.codelens.enabled` toggles CodeLens visibility |
| SPEC_CLIENT.md — CLI-009 (No-server mode) | CodeLens MUST work without LSP server |
| SPEC_PREVIEW.md — PRV-003 (VHS CLI invocation) | CodeLens triggers share the VHS execution engine with Preview |
| SPEC_PREVIEW.md — §9 (PreviewManager) | CodeLens "▶ Run & Preview" delegates to `PreviewManager.runAndPreview()` |

## 3. References

| Source | Role |
| --- | --- |
| [VSCode CodeLens API](https://code.visualstudio.com/api/language-extensions/programmatic-language-features#codelens-show-actionable-context-information-within-source-code) | `CodeLensProvider`, `provideCodeLenses`, `resolveCodeLens` |
| [VSCode API — CodeLens](https://vscode-api.js.org/interfaces/vscode.CodeLensProvider.html) | TypeScript interface reference |
| VHS Recording skill | VHS execution workflow |

## 4. Requirements

### CLS-001 — CodeLens Placement Strategy

| Field | Value |
| --- | --- |
| **ID** | CLS-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The CodeLens provider MUST place lenses at two positions: (1) **File-level**: Line 0 (or the first non-comment, non-empty line) with "▶ Run this tape" and "▶ Run & Preview". (2) **Output-level**: Above each `Output` directive line with "▶ Preview {filename}" (e.g., "▶ Preview demo.gif"). If the file has no `Output` directive, only the file-level CodeLens MUST appear. Output-level CodeLenses MUST extract the file name from the Output path for display. |
| **Verification** | File with 2 Output directives → 4 CodeLenses (2 at line 0, 1 above each Output). File with 0 Output directives → 2 CodeLenses at line 0. |

### CLS-002 — Command Registry

| Field | Value |
| --- | --- |
| **ID** | CLS-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST register the following commands in `contributes.commands`: (1) `vhs-analyzer.runTape` — execute VHS CLI on the current/specified tape file (no preview), (2) `vhs-analyzer.previewTape` — execute VHS CLI and open/refresh the preview panel, (3) `vhs-analyzer.stopRunning` — cancel any in-progress VHS execution for the current file. Commands MUST be executable via: CodeLens click, Command Palette, keyboard shortcut, and editor context menu (for `.tape` files). |
| **Verification** | Each command appears in Command Palette. CodeLens click triggers the correct command. Context menu on `.tape` file shows "Run Tape" and "Preview Tape". |

### CLS-003 — Execution State Machine

| Field | Value |
| --- | --- |
| **ID** | CLS-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | Each tape file MUST have an independent execution state tracked by the extension. States: `Idle`, `Running`, `Complete`, `Error`. Transitions: `Idle + run → Running`, `Running + exit(0) → Complete`, `Running + exit(non-0) → Error`, `Running + cancel → Idle`, `Complete/Error + run → Running`. The extension MUST enforce single-execution-per-file: triggering `runTape` while `Running` MUST cancel the current execution before starting a new one. |
| **Verification** | Run tape → state is Running. VHS completes → state is Complete. Run again → previous run cancels, new run starts. |

### CLS-004 — Status Bar Progress Indicator

| Field | Value |
| --- | --- |
| **ID** | CLS-004 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | During VHS execution, the extension SHOULD update the status bar item (SPEC_CLIENT.md CLI-011) to show: "$(sync~spin) VHS: Running {filename}...". On completion, the status bar SHOULD briefly flash "$(check) VHS: Done" for 3 seconds before reverting to the default server status. On error, the status bar SHOULD show "$(error) VHS: Failed" for 5 seconds. |
| **Verification** | Run tape → status bar shows spinner. VHS completes → "Done" flash. VHS fails → "Failed" flash. |

### CLS-005 — CodeLens Toggle via Configuration

| Field | Value |
| --- | --- |
| **ID** | CLS-005 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | When `vhs-analyzer.codelens.enabled` is `false`, the CodeLens provider MUST return an empty array (no lenses). Changing this setting MUST take effect immediately (fire `onDidChangeCodeLenses` event). The commands (CLS-002) MUST remain available in the Command Palette regardless of the CodeLens setting. |
| **Verification** | Set `codelens.enabled` to false → CodeLenses disappear. Set back to true → CodeLenses reappear. Commands still work via palette. |

### CLS-006 — CodeLens Provider Registration

| Field | Value |
| --- | --- |
| **ID** | CLS-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The CodeLens provider MUST be registered via `languages.registerCodeLensProvider({ language: "tape" }, provider)`. The provider MUST implement `CodeLensProvider` with: (1) `provideCodeLenses()` — scans the document text for Output directives and returns CodeLens objects at the appropriate line positions, (2) `onDidChangeCodeLenses` — an `EventEmitter` that fires when the document changes or when execution state changes (to update lens titles). |
| **Verification** | Open `.tape` file → CodeLenses appear. Edit file (add/remove Output) → CodeLenses update. |

### CLS-007 — Execution Output Channel

| Field | Value |
| --- | --- |
| **ID** | CLS-007 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | VHS CLI stdout and stderr SHOULD be forwarded to a dedicated "VHS Analyzer: Run" output channel. Each execution SHOULD be prefixed with a header: `[{timestamp}] Running: vhs {filename}`. On completion, a footer: `[{timestamp}] Completed in {duration}s (exit code: {code})`. The output channel SHOULD be revealed on error but NOT on success (to avoid focus stealing). |
| **Verification** | Run tape → output appears in "VHS Analyzer: Run" channel. Error → channel auto-reveals. Success → channel does not auto-reveal. |

### CLS-008 — Keyboard Shortcut Binding

| Field | Value |
| --- | --- |
| **ID** | CLS-008 |
| **Priority** | P2 (MAY) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MAY register default keybindings: (1) `Ctrl+Shift+R` (or `Cmd+Shift+R` on macOS) for `vhs-analyzer.runTape`, (2) `Ctrl+Shift+P` is reserved by VSCode. Keybindings MUST include a `when` clause: `editorLangId == 'tape'` to avoid conflicts with other extensions. |
| **Verification** | Open `.tape` file, press `Ctrl+Shift+R` → tape runs. Open `.js` file, press `Ctrl+Shift+R` → no action. |

### CLS-009 — Editor Context Menu

| Field | Value |
| --- | --- |
| **ID** | CLS-009 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | The extension SHOULD contribute editor context menu items for `.tape` files via `contributes.menus.editor/context`: (1) "VHS: Run Tape" → `vhs-analyzer.runTape`, (2) "VHS: Run & Preview" → `vhs-analyzer.previewTape`, (3) "VHS: Stop" → `vhs-analyzer.stopRunning` (visible only when Running). Menu items MUST have `when: "editorLangId == 'tape'"`. |
| **Verification** | Right-click in `.tape` editor → "VHS: Run Tape" and "VHS: Run & Preview" appear. Right-click in `.js` editor → menu items do not appear. |

### CLS-010 — Explorer Context Menu

| Field | Value |
| --- | --- |
| **ID** | CLS-010 |
| **Priority** | P2 (MAY) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MAY contribute explorer context menu items via `contributes.menus.explorer/context` for `.tape` files: (1) "VHS: Run Tape" → `vhs-analyzer.runTape` with the file URI argument. Menu items MUST have `when: "resourceExtname == '.tape'"`. |
| **Verification** | Right-click `.tape` file in Explorer → "VHS: Run Tape" appears. Right-click `.js` file → not shown. |

## 5. Design Options Analysis

### 5.1 CodeLens Placement Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: File-level only (line 0)** | Single CodeLens at the top of the file: "▶ Run this tape" | Minimal; clean; always present | No per-Output context; user cannot distinguish outputs in multi-Output files |
| **B: Output-level only** | CodeLens above each `Output` directive: "▶ Preview demo.gif" | Contextual; shows which output; clickable per-artifact | No CodeLens if file has no Output; user may not find the run button |
| **C: Combined (file-level + Output-level)** | Line 0: "▶ Run" + "▶ Run & Preview". Each Output: "▶ Preview {file}" | Complete UX; handles zero-Output files; per-artifact preview | More visual noise; more CodeLens items |

**Recommended: Option C (Combined).** VHS tape files are typically short
(<100 lines), so CodeLens visual noise is minimal. The file-level lens
provides a guaranteed entry point (even for files without Output). The
Output-level lens provides per-artifact preview context. This matches
the pattern used by Test Explorer (file-level "Run All" + per-test "Run").

### 5.2 Output Directive Detection Method

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Regex on document text** | `/^Output\s+(.+)/gm` on `document.getText()` | Fast; no AST dependency; works in no-server mode; same regex as Preview | May match inside comments if VHS ever supports inline comments |
| **B: LSP custom request** | Request AST node positions from the server | Accurate; uses real parser | Requires server; does not work in no-server mode |
| **C: TextMate token inspection** | Use `vscode.languages.getTokenInformationAtPosition()` (experimental) | Uses TextMate grammar | Experimental API; not stable; performance unknown |

**Recommended: Option A (Regex on document text).** CodeLens MUST work
in no-server mode (CLS-001 + CLI-009). The `Output` directive is line-anchored
and has trivial syntax. VHS does not support inline comments (LEX-004 specifies
`#` comments at line start only), so a line-anchored regex is robust. The
same regex pattern is reused by SPEC_PREVIEW.md PRV-004.

### 5.3 Execution Engine Architecture

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Shared ExecutionManager** | Singleton `ExecutionManager` manages per-file execution state and child processes; shared by CodeLens commands and Preview | Single source of truth; no duplicate processes; consistent state | Requires careful coordination between CodeLens and Preview |
| **B: Independent execution per trigger** | CodeLens and Preview each manage their own child processes | Simpler per-module code | Duplicate executions possible; state inconsistency |
| **C: Command-only (no manager)** | Commands directly spawn processes; no persistent state tracking | Simplest | No cancellation; no state awareness; no duplicate prevention |

**Recommended: Option A (Shared ExecutionManager).** A tape file should have
at most one VHS process running at a time. The ExecutionManager tracks per-file
state (Idle/Running/Complete/Error) and the active ChildProcess reference.
Both CodeLens commands and Preview delegate to this single manager. This
prevents duplicate executions and ensures consistent cancellation behavior.

### 5.4 CodeLens Title During Execution

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Static titles** | "▶ Run this tape" always, regardless of execution state | Simple; stable UI | No execution feedback in CodeLens |
| **B: Dynamic titles** | "▶ Run this tape" when Idle, "⏹ Running..." when Running, "✓ Done — ▶ Re-run" when Complete | Rich feedback; state awareness | Requires `onDidChangeCodeLenses` event firing on state change; minor flicker |
| **C: Dual lens** | Always show "▶ Run" + conditional "⏹ Stop" during execution | Clear separate actions | Two items change to three during execution; more visual noise |

**Recommended: Option B (Dynamic titles).** CodeLens is the primary visual
indicator for per-file execution state. Dynamic titles provide immediate
feedback without requiring the user to check the status bar. The
`onDidChangeCodeLenses` event is lightweight (triggers a re-query of
`provideCodeLenses`) and is the standard VSCode pattern for stateful
CodeLens.

## 6. Command Definitions (package.json fragment)

```json
{
  "contributes": {
    "commands": [
      {
        "command": "vhs-analyzer.runTape",
        "title": "VHS: Run Tape",
        "icon": "$(play)",
        "category": "VHS Analyzer"
      },
      {
        "command": "vhs-analyzer.previewTape",
        "title": "VHS: Run & Preview",
        "icon": "$(open-preview)",
        "category": "VHS Analyzer"
      },
      {
        "command": "vhs-analyzer.stopRunning",
        "title": "VHS: Stop",
        "icon": "$(debug-stop)",
        "category": "VHS Analyzer",
        "enablement": "vhs-analyzer.isRunning"
      }
    ],
    "menus": {
      "editor/context": [
        {
          "command": "vhs-analyzer.runTape",
          "when": "editorLangId == 'tape'",
          "group": "vhs@1"
        },
        {
          "command": "vhs-analyzer.previewTape",
          "when": "editorLangId == 'tape'",
          "group": "vhs@2"
        },
        {
          "command": "vhs-analyzer.stopRunning",
          "when": "editorLangId == 'tape' && vhs-analyzer.isRunning",
          "group": "vhs@3"
        }
      ],
      "explorer/context": [
        {
          "command": "vhs-analyzer.runTape",
          "when": "resourceExtname == '.tape'",
          "group": "vhs@1"
        }
      ]
    },
    "keybindings": [
      {
        "command": "vhs-analyzer.runTape",
        "key": "ctrl+shift+r",
        "mac": "cmd+shift+r",
        "when": "editorLangId == 'tape'"
      }
    ]
  }
}
```

## 7. Execution State Machine Diagram

```text
          ┌──────────────────────────────────────┐
          │                                      │
          ▼                                      │
    ┌──────────┐   runTape    ┌──────────┐       │
    │   Idle   │─────────────▶│ Running  │       │
    └──────────┘              └──────────┘       │
         ▲                     │  │   │          │
         │                     │  │   │          │
         │    cancel           │  │   │ exit(0)  │
         ├─────────────────────┘  │   │          │
         │                        │   ▼          │
         │         exit(≠0)  ┌──────────┐        │
         │         ┌─────────│ Complete │────────┘
         │         │         └──────────┘  runTape
         │         ▼
         │   ┌──────────┐
         └───│  Error   │──── runTape ────▶ Running
             └──────────┘
```

## 8. ExecutionManager Architecture

```typescript
class ExecutionManager {
    executions: Map<string, ExecutionState>  // key: tape file URI

    getState(uri: string): "idle" | "running" | "complete" | "error"

    async run(tapeUri: Uri, preview: boolean): Promise<void>
        // Cancel any running execution for this file
        this.cancel(tapeUri)

        state = { status: "running", process: null, startTime: Date.now() }
        this.executions.set(tapeUri, state)
        this.fireStateChange(tapeUri)

        // Spawn VHS
        const process = spawn("vhs", [tapeUri.fsPath], { cwd })
        state.process = process

        // Stream stderr to output channel and preview
        process.stderr.on("data", chunk => ...)

        process.on("exit", code =>
            if code === 0:
                state.status = "complete"
                if preview: previewManager.showPreview(tapeUri)
            else:
                state.status = "error"
            this.fireStateChange(tapeUri)
        )

    cancel(tapeUri: Uri): void
        state = this.executions.get(tapeUri)
        if state?.status === "running" and state.process:
            state.process.kill("SIGTERM")
            setTimeout(() => state.process?.kill("SIGKILL"), 3000)
            state.status = "idle"
            this.fireStateChange(tapeUri)

    onStateChange: EventEmitter<Uri>  // triggers CodeLens refresh
}
```

## 9. CodeLensProvider Implementation Outline

```typescript
class VhsCodeLensProvider implements CodeLensProvider {
    onDidChangeCodeLenses: Event<void>
    private changeEmitter = new EventEmitter<void>()

    constructor(executionManager: ExecutionManager):
        this.onDidChangeCodeLenses = this.changeEmitter.event
        executionManager.onStateChange(() => this.changeEmitter.fire())
        workspace.onDidChangeConfiguration(e =>
            if e.affectsConfiguration("vhs-analyzer.codelens"):
                this.changeEmitter.fire()
        )

    provideCodeLenses(document: TextDocument): CodeLens[]
        if !config.get("vhs-analyzer.codelens.enabled"):
            return []

        const lenses: CodeLens[] = []
        const state = executionManager.getState(document.uri)

        // File-level CodeLens at line 0
        const firstLine = findFirstNonTrivialLine(document)
        const runTitle = state === "running"
            ? "$(sync~spin) Running..."
            : "▶ Run this tape"
        lenses.push(new CodeLens(firstLine.range, {
            title: runTitle,
            command: "vhs-analyzer.runTape",
            arguments: [document.uri]
        }))
        lenses.push(new CodeLens(firstLine.range, {
            title: "▶ Run & Preview",
            command: "vhs-analyzer.previewTape",
            arguments: [document.uri]
        }))

        // Output-level CodeLenses
        const outputRegex = /^Output\s+(?:["'](.+?)["']|(\S+))/gm
        let match
        while (match = outputRegex.exec(document.getText())):
            const line = document.positionAt(match.index).line
            const outputPath = match[1] || match[2]
            const fileName = path.basename(outputPath)
            lenses.push(new CodeLens(
                new Range(line, 0, line, 0),
                {
                    title: `▶ Preview ${fileName}`,
                    command: "vhs-analyzer.previewTape",
                    arguments: [document.uri]
                }
            ))

        return lenses
}
```

## 10. Resolved Design Decisions

> All Freeze Candidates resolved through collaborative Architect–Orchestrator
> discussion on 2026-03-20.

### RD-CLS-01 — File-Level CodeLens Position

**Decision:** The file-level CodeLens MUST be placed at the first non-trivial
line — skip leading blank lines and `#` comment lines. If the file contains
only comments or is empty, fall back to line 0.

**Rationale:** VHS tape files may start with descriptive comments. Placing the
CodeLens at the first directive line (e.g., `Output`, `Set`) creates a tighter
visual association between the run button and the executable code. This matches
the Test Explorer pattern (file-level "Run All" above the first test).

### RD-CLS-02 — Output-Level CodeLens Action

**Decision:** Output-level CodeLens MUST trigger "Run & Preview" (execute VHS
and open the preview panel for that specific output artifact).

**Rationale:** The `▶` icon implies execution. For first-time files where the
output does not yet exist, a "Preview only" action would show an empty panel —
a poor first experience. "Run & Preview" is always actionable. The file-level
CodeLens already provides a "Run this tape" (no preview) option for
differentiation.

### RD-CLS-03 — Execution Context Variable

**Decision:** The extension MUST use
`vscode.commands.executeCommand("setContext", "vhs-analyzer.isRunning",
true/false)` to control menu item visibility. The ExecutionManager MUST update
this context variable on every state change.

**Rationale:** CLS-003 enforces single-execution-per-file, so at most one tape
is running at any time. A global boolean context variable is sufficient.
`setContext` is the official VSCode mechanism for `when`-clause menu visibility
control.

### RD-CLS-04 — Multiple Output Preview Target

**Decision:** When a file has multiple `Output` directives, the file-level
"Run & Preview" CodeLens MUST show the first `Output` directive's artifact.
Users MAY preview specific outputs via the Output-level CodeLens placed above
each `Output` directive.

**Rationale:** The first `Output` is the most predictable default. Most tape
files have only one Output. Per-Output CodeLens (RD-CLS-02) provides access to
any specific artifact. A Quick Pick selector would add unnecessary interaction
for the common single-Output case.
