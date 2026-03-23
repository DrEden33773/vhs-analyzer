import { readFile } from "node:fs/promises";
import path from "node:path";

import {
  ColorThemeKind,
  type Disposable,
  type Event,
  Uri,
  ViewColumn,
  window,
  workspace,
} from "vscode";
import previewStyles from "../media/preview.css?raw";

import { getExtensionConfiguration } from "./config";
import {
  type ExecutionProgressEvent,
  type ExecutionResult,
  inferOutputFormat,
  resolveArtifactPath,
} from "./execution";
import { resolveBinaryOnPath } from "./server";

const missingVhsMessage =
  "VHS is not installed. Preview requires the VHS CLI tool.";
const previewAutoRefreshDebounceMs = 500;

export const previewViewType = "vhs-preview";

export interface PreviewExecutionManagerLike {
  cancel(tapeUri: Uri): Promise<boolean>;
  getState(tapeUri: Uri): unknown;
  onDidWriteProgress: Event<ExecutionProgressEvent>;
  revealOutput?(preserveFocus?: boolean): void;
  run(tapeUri: Uri): Promise<ExecutionResult>;
}

export interface RenderCompleteMessage {
  artifactUri: string;
  format: "gif" | "mp4" | "webm";
  type: "renderComplete";
}

export interface RenderErrorMessage {
  cancelled: boolean;
  message: string;
  type: "renderError";
}

export interface RenderProgressMessage {
  line: string;
  type: "renderProgress";
}

export interface RenderStartMessage {
  tapeFile: string;
  type: "renderStart";
}

export interface ThemeChangeMessage {
  kind: "dark" | "high-contrast" | "light";
  type: "themeChange";
}

export type ExtensionToWebviewMessage =
  | RenderCompleteMessage
  | RenderErrorMessage
  | RenderProgressMessage
  | RenderStartMessage
  | ThemeChangeMessage;

export type WebviewToExtensionMessage =
  | { type: "cancel" }
  | { type: "ready" }
  | { type: "rerun" };

export interface PreviewManagerDependencies {
  clearScheduled: (timer: TimerHandle) => void;
  createFileSystemWatcher: (globPattern: string) => FileSystemWatcherLike;
  createWebviewPanel: (
    viewType: string,
    title: string,
    showOptions: number,
    options: PreviewPanelOptions,
  ) => WebviewPanelLike;
  executionManager: PreviewExecutionManagerLike;
  extensionPath: string;
  getConfiguration: typeof getExtensionConfiguration;
  getThemeKind: () => number;
  now: () => number;
  onDidChangeTheme: (listener: (theme: { kind: number }) => void) => Disposable;
  readTapeFile: (tapeUri: Uri) => Promise<string>;
  resolveExecutable: (binaryName: string) => Promise<string | null>;
  schedule: (callback: () => void, delayMs: number) => TimerHandle;
  workspaceFolders: readonly Uri[];
}

export interface PreviewPanelOptions {
  enableScripts: boolean;
  localResourceRoots: readonly Uri[];
  retainContextWhenHidden: boolean;
}

export interface WebviewLike {
  cspSource: string;
  html: string;
  options?: {
    enableScripts?: boolean;
    localResourceRoots?: readonly Uri[];
  };
  asWebviewUri(localResource: Uri): Uri;
  onDidReceiveMessage(listener: (message: unknown) => void): Disposable;
  postMessage(message: unknown): PromiseLike<boolean>;
}

export interface WebviewPanelLike {
  dispose(): void;
  onDidDispose(listener: () => void): Disposable;
  reveal(viewColumn?: number): void;
  title: string;
  viewType: string;
  webview: WebviewLike;
}

export interface FileSystemWatcherLike {
  dispose(): void;
  onDidChange(listener: (uri: Uri) => void): Disposable;
  onDidCreate(listener: (uri: Uri) => void): Disposable;
  onDidDelete(listener: (uri: Uri) => void): Disposable;
}

type TimerHandle = ReturnType<typeof setTimeout>;

interface PreviewPanelDependencies {
  clearScheduled: PreviewManagerDependencies["clearScheduled"];
  createFileSystemWatcher: PreviewManagerDependencies["createFileSystemWatcher"];
  createWebviewPanel: PreviewManagerDependencies["createWebviewPanel"];
  executionManager: PreviewExecutionManagerLike;
  extensionPath: string;
  getConfiguration: PreviewManagerDependencies["getConfiguration"];
  getThemeKind: PreviewManagerDependencies["getThemeKind"];
  now: PreviewManagerDependencies["now"];
  onDidChangeTheme: PreviewManagerDependencies["onDidChangeTheme"];
  readTapeFile: PreviewManagerDependencies["readTapeFile"];
  resolveExecutable: PreviewManagerDependencies["resolveExecutable"];
  schedule: PreviewManagerDependencies["schedule"];
  tapeUri: Uri;
  workspaceFolders: readonly Uri[];
}

export class PreviewManager {
  private readonly dependencies: PreviewManagerDependencies;
  private readonly panels = new Map<string, PreviewPanel>();

  constructor(
    dependencies: Partial<PreviewManagerDependencies> & {
      executionManager: PreviewExecutionManagerLike;
      extensionPath: string;
    },
  ) {
    const { executionManager, extensionPath, ...optionalDependencies } =
      dependencies;

    this.dependencies = {
      clearScheduled: clearTimeout,
      createFileSystemWatcher: (globPattern) =>
        workspace.createFileSystemWatcher(globPattern),
      createWebviewPanel: (viewType, title, showOptions, options) =>
        window.createWebviewPanel(viewType, title, showOptions, options),
      executionManager,
      extensionPath,
      getConfiguration: getExtensionConfiguration,
      getThemeKind: () => window.activeColorTheme.kind,
      now: Date.now,
      onDidChangeTheme: (listener) =>
        window.onDidChangeActiveColorTheme(listener),
      readTapeFile: async (tapeUri) => readFile(tapeUri.fsPath, "utf8"),
      resolveExecutable: resolveBinaryOnPath,
      schedule: setTimeout,
      workspaceFolders: [],
      ...optionalDependencies,
    };
  }

  dispose(): void {
    for (const panel of this.panels.values()) {
      panel.dispose();
    }

    this.panels.clear();
  }

  async runAndPreview(
    tapeUri: Uri,
    previewArtifactPath?: string,
  ): Promise<PreviewPanel> {
    const panel = await this.showPreview(tapeUri);
    await panel.startRender(previewArtifactPath);
    return panel;
  }

  async showPreview(tapeUri: Uri): Promise<PreviewPanel> {
    const existing = this.panels.get(tapeUri.fsPath);
    if (existing !== undefined) {
      existing.reveal();
      return existing;
    }

    const panel = await PreviewPanel.create({
      clearScheduled: this.dependencies.clearScheduled,
      createFileSystemWatcher: this.dependencies.createFileSystemWatcher,
      createWebviewPanel: this.dependencies.createWebviewPanel,
      executionManager: this.dependencies.executionManager,
      extensionPath: this.dependencies.extensionPath,
      getConfiguration: this.dependencies.getConfiguration,
      getThemeKind: this.dependencies.getThemeKind,
      now: this.dependencies.now,
      onDidChangeTheme: this.dependencies.onDidChangeTheme,
      readTapeFile: this.dependencies.readTapeFile,
      resolveExecutable: this.dependencies.resolveExecutable,
      schedule: this.dependencies.schedule,
      tapeUri,
      workspaceFolders: this.dependencies.workspaceFolders,
    });

    this.panels.set(tapeUri.fsPath, panel);
    panel.onDidDispose(() => {
      this.panels.delete(tapeUri.fsPath);
    });
    return panel;
  }
}

export class PreviewPanel {
  static async create(
    dependencies: PreviewPanelDependencies,
  ): Promise<PreviewPanel> {
    const tapeSource = await dependencies.readTapeFile(dependencies.tapeUri);
    const workingDirectory = resolveWorkingDirectory(
      dependencies.tapeUri,
      dependencies.workspaceFolders,
    );
    const artifactPath = resolveArtifactPath(
      dependencies.tapeUri,
      workingDirectory,
      tapeSource,
    );
    const localResourceRoots = buildLocalResourceRoots({
      artifactPath,
      extensionPath: dependencies.extensionPath,
      tapeUri: dependencies.tapeUri,
      workspaceFolders: dependencies.workspaceFolders,
    });
    const panel = dependencies.createWebviewPanel(
      previewViewType,
      `VHS Preview: ${path.basename(dependencies.tapeUri.fsPath)}`,
      ViewColumn.Beside,
      {
        enableScripts: true,
        localResourceRoots,
        retainContextWhenHidden: true,
      },
    );

    panel.webview.html = createPreviewHtml({
      cspSource: panel.webview.cspSource,
      inlineStyles: previewStyles,
      nonce: createNonce(),
    });

    return new PreviewPanel(
      {
        clearScheduled: dependencies.clearScheduled,
        createFileSystemWatcher: dependencies.createFileSystemWatcher,
        executionManager: dependencies.executionManager,
        getConfiguration: dependencies.getConfiguration,
        getThemeKind: dependencies.getThemeKind,
        now: dependencies.now,
        onDidChangeTheme: dependencies.onDidChangeTheme,
        panel,
        resolveExecutable: dependencies.resolveExecutable,
        schedule: dependencies.schedule,
        tapeUri: dependencies.tapeUri,
        workspaceFolders: dependencies.workspaceFolders,
      },
      dependencies.tapeUri,
    );
  }

  constructor(
    private readonly dependencies: {
      clearScheduled: PreviewPanelDependencies["clearScheduled"];
      createFileSystemWatcher: PreviewPanelDependencies["createFileSystemWatcher"];
      executionManager: PreviewExecutionManagerLike;
      getConfiguration: PreviewPanelDependencies["getConfiguration"];
      getThemeKind: PreviewPanelDependencies["getThemeKind"];
      now: PreviewPanelDependencies["now"];
      onDidChangeTheme: PreviewPanelDependencies["onDidChangeTheme"];
      panel: WebviewPanelLike;
      resolveExecutable: PreviewPanelDependencies["resolveExecutable"];
      schedule: PreviewPanelDependencies["schedule"];
      tapeUri: Uri;
      workspaceFolders: readonly Uri[];
    },
    private readonly tapeUri: Uri,
  ) {
    this.disposables.push(
      this.dependencies.executionManager.onDidWriteProgress((event) => {
        if (event.tapeUri.fsPath !== this.tapeUri.fsPath) {
          return;
        }

        this.postMessage({
          line: event.line,
          type: "renderProgress",
        });
      }),
      this.dependencies.panel.onDidDispose(() => {
        void this.dependencies.executionManager.cancel(this.tapeUri);
        this.disposeResources();
      }),
      this.dependencies.onDidChangeTheme(() => {
        this.postCurrentTheme();
      }),
      this.dependencies.panel.webview.onDidReceiveMessage((message) => {
        void this.handleWebviewMessage(message);
      }),
    );
  }

  private readonly disposables: Disposable[] = [];
  private lastRenderResult: ExecutionResult | undefined;
  private refreshTimer: TimerHandle | undefined;
  private watcher: FileSystemWatcherLike | undefined;

  dispose(): void {
    this.dependencies.panel.dispose();
  }

  onDidDispose(listener: () => void): Disposable {
    return this.dependencies.panel.onDidDispose(listener);
  }

  reveal(): void {
    this.dependencies.panel.reveal(ViewColumn.Beside);
  }

  async startRender(previewArtifactPath?: string): Promise<void> {
    const vhsPath = await this.dependencies.resolveExecutable("vhs");
    if (vhsPath === null) {
      this.postMessage({
        cancelled: false,
        message: missingVhsMessage,
        type: "renderError",
      });
      return;
    }

    void vhsPath;
    this.dependencies.executionManager.revealOutput?.(true);
    this.postMessage({
      tapeFile: path.basename(this.tapeUri.fsPath),
      type: "renderStart",
    });

    try {
      const result = await this.dependencies.executionManager.run(this.tapeUri);
      const resolvedResult = resolvePreviewResult(
        this.tapeUri,
        result,
        previewArtifactPath,
        this.dependencies.workspaceFolders,
      );
      this.lastRenderResult = resolvedResult;
      this.ensureLocalResourceRoot(resolvedResult.artifactPath);
      this.configureAutoRefresh(resolvedResult);
      this.postRenderComplete(resolvedResult);
    } catch (error) {
      this.postMessage({
        cancelled: isCancelledExecution(error),
        message:
          error instanceof Error
            ? error.message
            : "Failed to render VHS preview.",
        type: "renderError",
      });
    }
  }

  private async handleWebviewMessage(message: unknown): Promise<void> {
    const typedMessage = message as Partial<WebviewToExtensionMessage>;

    switch (typedMessage.type) {
      case "cancel":
        await this.dependencies.executionManager.cancel(this.tapeUri);
        return;
      case "rerun":
        await this.startRender();
        return;
      case "ready":
        this.postCurrentTheme();
        return;
      default:
        return;
    }
  }

  private disposeResources(): void {
    this.disposeWatcher();

    if (this.refreshTimer !== undefined) {
      this.dependencies.clearScheduled(this.refreshTimer);
      this.refreshTimer = undefined;
    }

    for (const disposable of this.disposables.splice(0)) {
      disposable.dispose();
    }
  }

  private postMessage(message: ExtensionToWebviewMessage): void {
    void this.dependencies.panel.webview.postMessage(message);
  }

  private postCurrentTheme(): void {
    this.postMessage({
      kind: mapColorThemeKind(this.dependencies.getThemeKind()),
      type: "themeChange",
    });
  }

  private configureAutoRefresh(result: ExecutionResult): void {
    this.disposeWatcher();

    if (!this.dependencies.getConfiguration().previewAutoRefresh) {
      return;
    }

    this.watcher = this.dependencies.createFileSystemWatcher(
      result.artifactPath,
    );
    this.disposables.push(
      this.watcher.onDidChange(() => {
        this.scheduleRefresh();
      }),
      this.watcher.onDidCreate(() => {
        this.scheduleRefresh();
      }),
    );
  }

  private disposeWatcher(): void {
    this.watcher?.dispose();
    this.watcher = undefined;
  }

  private ensureLocalResourceRoot(artifactPath: string): void {
    const currentOptions = this.dependencies.panel.webview.options ?? {};
    const currentRoots = currentOptions.localResourceRoots ?? [];
    const artifactDirectory = Uri.file(path.dirname(artifactPath));
    if (
      currentRoots.some((root) =>
        isWithinDirectory(root.fsPath, artifactDirectory.fsPath),
      )
    ) {
      return;
    }

    this.dependencies.panel.webview.options = {
      ...currentOptions,
      localResourceRoots: [...currentRoots, artifactDirectory],
    };
  }

  private postRenderComplete(
    result: ExecutionResult,
    withCacheBust = false,
  ): void {
    const artifactUri = Uri.file(result.artifactPath).with({
      query: withCacheBust ? `t=${this.dependencies.now()}` : "",
    });
    const resolvedArtifactUri = this.dependencies.panel.webview
      .asWebviewUri(artifactUri)
      .toString();

    this.postMessage({
      artifactUri: resolvedArtifactUri,
      format: result.format,
      type: "renderComplete",
    });
  }

  private scheduleRefresh(): void {
    if (this.lastRenderResult === undefined) {
      return;
    }

    if (this.refreshTimer !== undefined) {
      this.dependencies.clearScheduled(this.refreshTimer);
    }

    this.refreshTimer = this.dependencies.schedule(() => {
      this.refreshTimer = undefined;
      if (this.lastRenderResult !== undefined) {
        this.postRenderComplete(this.lastRenderResult, true);
      }
    }, previewAutoRefreshDebounceMs);
  }
}

export function buildLocalResourceRoots(options: {
  artifactPath: string;
  extensionPath: string;
  tapeUri: Uri;
  workspaceFolders: readonly Uri[];
}): Uri[] {
  const roots = new Map<string, Uri>();

  for (const workspaceFolder of options.workspaceFolders) {
    roots.set(workspaceFolder.fsPath, workspaceFolder);
  }

  const artifactDirectory = Uri.file(path.dirname(options.artifactPath));
  if (
    !options.workspaceFolders.some((folder) =>
      isWithinDirectory(folder.fsPath, artifactDirectory.fsPath),
    )
  ) {
    roots.set(artifactDirectory.fsPath, artifactDirectory);
  }

  const tapeDirectory = Uri.file(path.dirname(options.tapeUri.fsPath));
  roots.set(tapeDirectory.fsPath, tapeDirectory);

  return [...roots.values()];
}

export function createPreviewHtml(options: {
  cspSource: string;
  inlineStyles: string;
  nonce: string;
}): string {
  const escapedMissingVhsMessage = JSON.stringify(missingVhsMessage);
  const escapedInlineStyles = escapeInlineStyle(options.inlineStyles);

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src ${options.cspSource}; media-src ${options.cspSource}; style-src ${options.cspSource} 'unsafe-inline'; script-src 'nonce-${options.nonce}';">
  <style>${escapedInlineStyles}</style>
  <title>VHS Preview</title>
</head>
<body>
  <div id="container">
    <div id="prompt">Click ▶ Run or press the button to preview.</div>
    <div id="loading" hidden>
      <div class="spinner"></div>
      <p id="progress-text">Rendering...</p>
      <button id="cancel-btn" type="button">Cancel</button>
    </div>
    <div id="complete" hidden></div>
    <div id="error" hidden>
      <p id="error-text"></p>
      <button id="retry-btn" type="button">Retry</button>
      <a id="install-vhs-link" href="https://github.com/charmbracelet/vhs#installation" hidden>Install VHS</a>
    </div>
  </div>
  <script nonce="${options.nonce}">
    const vscode = acquireVsCodeApi();
    const prompt = document.getElementById("prompt");
    const loading = document.getElementById("loading");
    const complete = document.getElementById("complete");
    const error = document.getElementById("error");
    const progressText = document.getElementById("progress-text");
    const errorText = document.getElementById("error-text");
    const cancelButton = document.getElementById("cancel-btn");
    const retryButton = document.getElementById("retry-btn");
    const installLink = document.getElementById("install-vhs-link");
    const missingVhsMessage = ${escapedMissingVhsMessage};

    const setVisibleState = (nextVisible) => {
      for (const element of [prompt, loading, complete, error]) {
        element.hidden = element !== nextVisible;
      }
    };

    const renderArtifact = (artifactUri, format) => {
      if (format === "gif") {
        return \`<img class="artifact-image" src="\${artifactUri}" alt="VHS preview output">\`;
      }

      return \`<video controls autoplay loop class="artifact-video" src="\${artifactUri}"></video>\`;
    };

    cancelButton.addEventListener("click", () => {
      vscode.postMessage({ type: "cancel" });
    });
    retryButton.addEventListener("click", () => {
      vscode.postMessage({ type: "rerun" });
    });

    window.addEventListener("message", (event) => {
      const message = event.data;

      switch (message.type) {
        case "renderStart":
          progressText.textContent = \`Rendering \${message.tapeFile}...\`;
          installLink.hidden = true;
          setVisibleState(loading);
          return;
        case "renderProgress":
          progressText.textContent = message.line;
          setVisibleState(loading);
          return;
        case "renderComplete":
          complete.innerHTML = renderArtifact(message.artifactUri, message.format);
          installLink.hidden = true;
          setVisibleState(complete);
          return;
        case "renderError":
          errorText.textContent = message.message;
          installLink.hidden = message.message !== missingVhsMessage;
          setVisibleState(error);
          return;
        case "themeChange":
          document.body.dataset.theme = message.kind;
          return;
        default:
          return;
      }
    });

    vscode.postMessage({ type: "ready" });
  </script>
</body>
</html>`;
}

export function createArtifactMarkup(
  artifactUri: string,
  format: "gif" | "mp4" | "webm",
): string {
  const escapedArtifactUri = escapeHtmlAttribute(artifactUri);

  if (format === "gif") {
    return `<img class="artifact-image" src="${escapedArtifactUri}" alt="VHS preview output">`;
  }

  return `<video controls autoplay loop class="artifact-video" src="${escapedArtifactUri}"></video>`;
}

export function createNonce(): string {
  return Math.random().toString(36).slice(2, 12);
}

function isCancelledExecution(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "cancelled" in error &&
    error.cancelled === true
  );
}

function mapColorThemeKind(kind: number): ThemeChangeMessage["kind"] {
  switch (kind) {
    case ColorThemeKind.Light:
      return "light";
    case ColorThemeKind.HighContrast:
    case ColorThemeKind.HighContrastLight:
      return "high-contrast";
    default:
      return "dark";
  }
}

function escapeHtmlAttribute(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function escapeInlineStyle(value: string): string {
  return value.replaceAll("</style", "<\\/style");
}

function isWithinDirectory(directoryPath: string, filePath: string): boolean {
  const relativePath = path.relative(directoryPath, filePath);
  return (
    relativePath === "" ||
    (!relativePath.startsWith("..") && !path.isAbsolute(relativePath))
  );
}

function resolveWorkingDirectory(
  tapeUri: Uri,
  _workspaceFolders: readonly Uri[],
): string {
  return path.dirname(tapeUri.fsPath);
}

function resolvePreviewResult(
  tapeUri: Uri,
  result: ExecutionResult,
  previewArtifactPath?: string,
  workspaceFolders: readonly Uri[] = [],
): ExecutionResult {
  if (previewArtifactPath === undefined) {
    return result;
  }

  const workingDirectory = resolveWorkingDirectory(tapeUri, workspaceFolders);
  const artifactPath = path.resolve(workingDirectory, previewArtifactPath);
  return {
    artifactPath,
    format: inferOutputFormat(artifactPath),
    tapeUri,
  };
}
