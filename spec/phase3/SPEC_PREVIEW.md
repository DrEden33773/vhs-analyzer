# SPEC_PREVIEW.md — Side-by-Side Live Preview Webview

**Phase:** 3 — VSCode Extension Client
**Work Stream:** WS-2 (Preview)
**Status:** Stage A (Exploratory Design)
**Owner:** Architect
**Depends On:** WS-1 (SPEC_CLIENT.md — extension activation, configuration schema)
**Last Updated:** 2026-03-19

---

## 1. Purpose

Define the Webview-based live preview panel that renders VHS output artifacts
(GIF/MP4/WebM) side-by-side with the `.tape` editor. The preview invokes the
native VHS CLI, captures its output, and displays the result in a theme-aware
Webview with loading states, error display, and auto-refresh on file changes.

## 2. Cross-Phase Dependencies

| Phase 1/2 Contract | Usage in This Spec |
| --- | --- |
| SPEC_PARSER.md — §5 (Ungrammar: OutputCommand) | Preview extracts `Output` directive path from the tape file to determine which artifact to display |
| SPEC_CLIENT.md — CLI-006 (Configuration) | `vhs-analyzer.preview.autoRefresh` setting controls auto-refresh behavior |
| SPEC_CLIENT.md — CLI-007 (Runtime dependency detection) | VHS CLI availability is checked on activation; preview gracefully degrades if VHS is missing |

| External Dependency | Usage |
| --- | --- |
| VHS CLI (`vhs`) | Invoked as a child process to render `.tape` → GIF/MP4/WebM |
| `ttyd` | Required by VHS for terminal emulation (transitive) |
| `ffmpeg` | Required by VHS for video encoding (transitive) |

## 3. References

| Source | Role |
| --- | --- |
| [VSCode Webview API](https://code.visualstudio.com/api/extension-guides/webview) | `createWebviewPanel`, `retainContextWhenHidden`, `asWebviewUri` |
| [VHS README](https://github.com/charmbracelet/vhs?tab=readme-ov-file) | CLI invocation, `--output` flag, output formats |
| TypeScript Advanced Types skill | Discriminated union message protocol design |
| VHS Recording skill | VHS execution workflow and troubleshooting |

## 4. Requirements

### PRV-001 — Webview Panel Creation

| Field | Value |
| --- | --- |
| **ID** | PRV-001 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST create a Webview panel via `window.createWebviewPanel()` with: (1) `viewType: "vhs-preview"`, (2) `title: "VHS Preview: {filename}"` where `{filename}` is the base name of the `.tape` file, (3) `showOptions: ViewColumn.Beside` for side-by-side layout, (4) `options: { enableScripts: true, retainContextWhenHidden: true, localResourceRoots: [...] }`. If a preview panel for the same file already exists, the extension MUST reveal it instead of creating a duplicate. |
| **Verification** | Trigger preview → panel opens beside editor. Trigger again → existing panel reveals. Close panel → clean disposal. |

### PRV-002 — Webview Messaging Protocol

| Field | Value |
| --- | --- |
| **ID** | PRV-002 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The Extension ↔ Webview communication MUST use a typed messaging protocol defined as TypeScript discriminated unions (see §6). All messages MUST have a `type` discriminant field. Extension → Webview messages: `renderStart`, `renderProgress`, `renderComplete`, `renderError`, `themeChange`. Webview → Extension messages: `rerun`, `cancel`, `ready`. The Webview MUST call `acquireVsCodeApi()` once and use `postMessage()` / `addEventListener('message', ...)` for communication. |
| **Verification** | Trigger render → Webview receives `renderStart` then `renderProgress` then `renderComplete`. Click "Re-run" in Webview → Extension receives `rerun` message. |

### PRV-003 — VHS CLI Invocation

| Field | Value |
| --- | --- |
| **ID** | PRV-003 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST invoke VHS via `child_process.spawn("vhs", [tapeFilePath])`. The working directory MUST be the workspace folder containing the `.tape` file, or the file's parent directory if no workspace is open. The VHS process stdout and stderr MUST be captured: stderr lines SHOULD be forwarded to the Webview as `renderProgress` messages. On process exit code 0, the extension MUST send `renderComplete` with the output artifact URI. On non-zero exit, the extension MUST send `renderError` with the captured stderr. |
| **Verification** | Valid tape file → VHS runs, GIF appears in preview. Invalid tape file → error message shown in Webview. |

### PRV-004 — Output Artifact Discovery

| Field | Value |
| --- | --- |
| **ID** | PRV-004 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST determine the output artifact path by: (1) Parsing the `.tape` file text to find the first `Output` directive and extracting its path argument. (2) Resolving the path relative to the `.tape` file's directory. (3) If no `Output` directive is found, using the VHS default: `out.gif` in the working directory. The extension MUST support all VHS output formats: `.gif`, `.mp4`, `.webm`. |
| **Verification** | File with `Output demo.gif` → preview shows `demo.gif`. File without `Output` → preview shows `out.gif`. |

### PRV-005 — Auto-Refresh on Output File Change

| Field | Value |
| --- | --- |
| **ID** | PRV-005 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | When `vhs-analyzer.preview.autoRefresh` is `true`, the extension SHOULD watch the output artifact file using `workspace.createFileSystemWatcher()`. On file change, the extension SHOULD send a `renderComplete` message with a cache-busting URI (appending `?t={timestamp}`) to force the Webview to reload the artifact. A debounce of 500ms SHOULD be applied to avoid flickering during VHS rendering (VHS writes output incrementally). |
| **Verification** | Run VHS externally → preview auto-refreshes when GIF changes. Disable `autoRefresh` setting → preview does not auto-refresh. |

### PRV-006 — Execution Cancellation

| Field | Value |
| --- | --- |
| **ID** | PRV-006 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The extension MUST support cancelling an in-progress VHS execution. Cancellation MUST be triggered by: (1) Webview "Cancel" button (sends `cancel` message), (2) closing the preview panel during execution, (3) triggering a new execution for the same file while one is running. Cancellation MUST send `SIGTERM` to the VHS process, wait 3 seconds, then `SIGKILL` if still running. On cancellation, the Webview MUST receive a `renderError` message with `cancelled: true`. |
| **Verification** | Start VHS on long tape → click Cancel → process terminates, Webview shows "Cancelled". Start new run while running → previous run cancels. |

### PRV-007 — Content Security Policy

| Field | Value |
| --- | --- |
| **ID** | PRV-007 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The Webview HTML MUST include a CSP meta tag restricting resource loading: `default-src 'none'; img-src ${webview.cspSource}; media-src ${webview.cspSource}; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';`. The `localResourceRoots` MUST include: (1) the workspace folder, (2) the output artifact's parent directory (if outside workspace), (3) the extension's `media/` directory (for icons and stylesheets). All local resources MUST be loaded via `webview.asWebviewUri()`. |
| **Verification** | Preview loads GIF from workspace. Attempt to load resource outside localResourceRoots → fails (CSP enforced). |

### PRV-008 — Theme-Aware Webview Styling

| Field | Value |
| --- | --- |
| **ID** | PRV-008 |
| **Priority** | P1 (SHOULD) |
| **Owner** | Architect → Builder |
| **Statement** | The Webview SHOULD respect VSCode's current color theme. The HTML template SHOULD use CSS custom properties from the VSCode Webview API (`--vscode-editor-background`, `--vscode-editor-foreground`, `--vscode-button-background`, etc.). On theme change, the extension SHOULD send a `themeChange` message to update the Webview styling. The Webview MUST be visually usable in both light and dark themes. |
| **Verification** | Switch VSCode theme from dark to light → Webview background updates. Preview is readable in both themes. |

### PRV-009 — Loading and Error States

| Field | Value |
| --- | --- |
| **ID** | PRV-009 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | The Webview MUST display three distinct visual states: (1) **Loading**: animated spinner with progress text from VHS stderr, (2) **Complete**: rendered artifact (GIF as `<img>`, MP4/WebM as `<video controls>`), (3) **Error**: error message from VHS stderr with a "Retry" button. The initial state on panel creation MUST be a "Click ▶ Run or press the button to preview" prompt. |
| **Verification** | Open preview without running → prompt state. Run VHS → loading spinner appears with progress. VHS completes → artifact displayed. VHS fails → error with Retry button. |

### PRV-010 — VHS Missing Graceful Degradation

| Field | Value |
| --- | --- |
| **ID** | PRV-010 |
| **Priority** | P0 (MUST) |
| **Owner** | Architect → Builder |
| **Statement** | If the `vhs` CLI is not found on the system PATH, the preview panel MUST display a user-friendly error: "VHS is not installed. Preview requires the VHS CLI tool." with an "Install VHS" button linking to `https://github.com/charmbracelet/vhs#installation`. The extension MUST NOT attempt to spawn a process that will fail with ENOENT. |
| **Verification** | Remove `vhs` from PATH → open preview → installation prompt appears. Click "Install VHS" → browser opens installation page. |

## 5. Design Options Analysis

### 5.1 Webview Persistence Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: retainContextWhenHidden** | Set `retainContextWhenHidden: true` on panel creation | Webview preserves state when hidden behind other tabs; instant reveal; messages can be sent while hidden | Higher memory usage; Webview stays alive in memory |
| **B: Serialize/Deserialize** | Use `WebviewPanelSerializer` to save/restore state | Lower memory; survives extension restarts | Complex serialization; preview content (GIF binary) cannot be serialized; must re-render |
| **C: Always recreate** | Dispose and recreate Webview on every reveal | Simplest; lowest memory | Loses preview state; re-render required on every reveal; poor UX |

**Recommended: Option A (retainContextWhenHidden).** Preview content is an
image/video that should persist when the user switches tabs and returns.
Re-rendering a VHS tape takes 5-30 seconds; losing the preview on tab
switch is unacceptable UX. The memory cost is minimal (a single img/video
element). `WebviewPanelSerializer` (Option B) is overkill for this use
case since the preview state is just a URI reference.

### 5.2 VHS Output Path Discovery

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Parse tape file text with regex** | Simple regex `/^Output\s+(.+)/m` on the raw file text | Fast; no AST dependency; works in no-server mode | May match inside comments; fragile if VHS syntax evolves |
| **B: Use LSP custom request** | Send a custom `vhs/outputPath` request to the LSP server | Accurate; uses the real parser; AST-based | Requires server running; doesn't work in no-server mode |
| **C: Hybrid (regex fallback)** | Try LSP request first; fall back to regex if server unavailable | Best of both; works in all modes | Two code paths to maintain |

**Recommended: Option A (Parse tape file text with regex).** The `Output`
directive syntax is trivial (`Output <path>`) and stable. A line-anchored
regex is robust against comments (which start with `#` on their own line
in VHS). The preview must work in no-server mode (CLI-009), so it cannot
depend on the LSP server. A single regex eliminates the need for a custom
LSP request protocol. If the regex proves fragile, it can be upgraded to
Option C in a future iteration.

### 5.3 VHS CLI Invocation Method

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: Direct spawn** | `child_process.spawn("vhs", [tapeFilePath])` | Simple; VHS reads Output directive from file; works with all VHS versions | Cannot override output path without `--output` flag |
| **B: Spawn with --output override** | `child_process.spawn("vhs", [tapeFilePath, "-o", tempPath])` | Controls output location; avoids polluting workspace | Overrides user's intended output path; may conflict with multiple Output directives |
| **C: Temp file with modified Output** | Copy tape to temp, rewrite Output path, run VHS on temp | Full control over output | Complex; may break Source/Require relative paths |

**Recommended: Option A (Direct spawn).** VHS reads the `Output` directive
from the tape file and produces the artifact at the declared path. The
extension then reads that path for preview. This respects the user's
intended output and avoids side effects. The extension already knows the
output path from PRV-004 (parsing the Output directive). No temporary
files or path rewrites are needed.

### 5.4 Artifact Display Strategy

| Option | Description | Pros | Cons |
| --- | --- | --- | --- |
| **A: img/video element with asWebviewUri** | GIF → `<img src="${uri}">`, MP4/WebM → `<video src="${uri}" controls>` | Native browser rendering; hardware-accelerated video; simple | Cache busting needed for updates |
| **B: Embedded binary (base64 data URI)** | Read artifact, encode as base64, embed in `src="data:..."` | No CSP/localResourceRoots concerns | Large files cause Webview memory issues; slow encoding; 33% size overhead |
| **C: Local HTTP server** | Serve artifacts via a localhost HTTP server | Clean URLs; easy streaming | Extra server process; port management; firewall concerns |

**Recommended: Option A (img/video element with asWebviewUri).** This is
the VSCode-recommended approach for displaying local resources in Webviews.
`asWebviewUri()` handles the protocol conversion and security. Cache
busting via `?t={timestamp}` query parameter forces reload on artifact
update. No extra dependencies or processes.

## 6. Messaging Protocol Type Definitions

```typescript
// Extension → Webview messages
interface RenderStartMessage {
  type: "renderStart";
  tapeFile: string;
}

interface RenderProgressMessage {
  type: "renderProgress";
  line: string;
}

interface RenderCompleteMessage {
  type: "renderComplete";
  artifactUri: string;
  format: "gif" | "mp4" | "webm";
}

interface RenderErrorMessage {
  type: "renderError";
  message: string;
  cancelled: boolean;
}

interface ThemeChangeMessage {
  type: "themeChange";
  kind: "light" | "dark" | "high-contrast";
}

type ExtensionToWebviewMessage =
  | RenderStartMessage
  | RenderProgressMessage
  | RenderCompleteMessage
  | RenderErrorMessage
  | ThemeChangeMessage;

// Webview → Extension messages
interface RerunMessage {
  type: "rerun";
}

interface CancelMessage {
  type: "cancel";
}

interface ReadyMessage {
  type: "ready";
}

type WebviewToExtensionMessage =
  | RerunMessage
  | CancelMessage
  | ReadyMessage;
```

## 7. Webview HTML Template Structure

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy"
    content="default-src 'none';
             img-src ${cspSource};
             media-src ${cspSource};
             style-src ${cspSource} 'unsafe-inline';
             script-src 'nonce-${nonce}';">
  <title>VHS Preview</title>
  <style>
    /* Theme-aware CSS using VSCode custom properties */
    body {
      background: var(--vscode-editor-background);
      color: var(--vscode-editor-foreground);
      display: flex;
      align-items: center;
      justify-content: center;
      height: 100vh;
      margin: 0;
      font-family: var(--vscode-font-family);
    }
    /* ... loading, complete, error state styles ... */
  </style>
</head>
<body>
  <div id="container">
    <!-- States: prompt | loading | complete | error -->
    <div id="prompt">Click ▶ Run to preview this tape.</div>
    <div id="loading" hidden>
      <div class="spinner"></div>
      <p id="progress-text">Rendering...</p>
    </div>
    <div id="complete" hidden>
      <!-- img or video element injected here -->
    </div>
    <div id="error" hidden>
      <p id="error-text"></p>
      <button id="retry-btn">Retry</button>
    </div>
  </div>
  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    // Message handling and state transitions
  </script>
</body>
</html>
```

## 8. Preview Panel Lifecycle State Machine

```text
States: Prompt → Loading → Complete | Error
                           ↑         ↑
                           └── Rerun ─┘

Transitions:
  Prompt  + runTape command   → Loading
  Loading + renderComplete    → Complete
  Loading + renderError       → Error
  Loading + cancel            → Error (cancelled: true)
  Complete + rerun            → Loading
  Error    + rerun/retry      → Loading
  Any      + panel.onDispose  → (disposed, cancel running process)
```

## 9. Preview Manager Architecture

```typescript
class PreviewManager {
    panels: Map<string, PreviewPanel>  // key: tape file URI

    showPreview(tapeUri: Uri): void
        if panels.has(tapeUri):
            panels.get(tapeUri).reveal()
        else:
            panel = new PreviewPanel(tapeUri)
            panels.set(tapeUri, panel)
            panel.onDispose(() => panels.delete(tapeUri))

    runAndPreview(tapeUri: Uri): void
        showPreview(tapeUri)
        panels.get(tapeUri).startRender()
}

class PreviewPanel {
    webviewPanel: WebviewPanel
    process: ChildProcess | null
    watcher: FileSystemWatcher | null

    constructor(tapeUri: Uri):
        // Create WebviewPanel, set HTML, register message handler

    startRender(): void
        cancelRunning()
        send({ type: "renderStart", tapeFile: path.basename(tapeUri) })
        process = spawn("vhs", [tapeFilePath], { cwd: workDir })
        process.stderr.on("data", line =>
            send({ type: "renderProgress", line }))
        process.on("exit", code =>
            if code === 0:
                send({ type: "renderComplete", artifactUri, format })
            else:
                send({ type: "renderError", message: stderr, cancelled: false }))

    cancelRunning(): void
        if process:
            process.kill("SIGTERM")
            setTimeout(() => process?.kill("SIGKILL"), 3000)

    dispose(): void
        cancelRunning()
        watcher?.dispose()
}
```

## 10. Freeze Candidates

### FC-PRV-01 — Webview Panel: One Per File vs. Singleton

**Question:** Should there be one preview panel per tape file, or a single
shared preview panel that switches content?

**Analysis:** One per file allows side-by-side comparison of two tapes but
consumes more resources. A singleton is simpler and matches the "Preview"
pattern in VSCode (Markdown preview is singleton per file). Recommended:
one panel per file (matching PRV-001), but limited to a configurable max
(default 3). Excess panels auto-dispose the oldest.

**Leaning:** One per file, max 3 panels.

### FC-PRV-02 — VHS stderr Parsing for Progress

**Question:** Should the extension parse VHS stderr for structured progress
information, or pass raw lines through?

**Analysis:** VHS stderr output is unstructured text (e.g., "Rendering GIF...",
timing info). Parsing for percentage or step progress is fragile and VHS-version
dependent. Raw passthrough is robust and always reflects actual VHS output.

**Leaning:** Raw passthrough. Display raw stderr lines in the Webview loading
state. If VHS adds structured progress output in the future, upgrade to parsing.

### FC-PRV-03 — Output Path Regex Robustness

**Question:** The regex `/^Output\s+(.+)/m` for extracting the output path —
should it handle quoted paths (e.g., `Output "path with spaces.gif"`)?

**Analysis:** VHS supports both quoted and unquoted paths in the Output
directive. The regex should strip surrounding quotes from the captured group.
Pattern: `/^Output\s+(?:["'](.+?)["']|(\S+))/m` — matches quoted or unquoted.

**Leaning:** Quote-aware regex as described above.

### FC-PRV-04 — Video Playback Controls

**Question:** For MP4/WebM output, should the Webview provide custom playback
controls or use the browser's native `<video controls>`?

**Analysis:** Native `<video controls>` provides play, pause, seek, volume,
fullscreen — all for free. Custom controls would add complexity without clear
benefit. Native controls are theme-aware in Chromium (VSCode's Webview engine).

**Leaning:** Native `<video controls autoplay loop>` for MP4/WebM. `<img>` for
GIF (which auto-loops natively).
