import { afterEach, describe, expect, it, vi } from "vitest";
import { Uri, ViewColumn } from "vscode";

vi.mock("vscode", async () => import("./__mocks__/vscode.js"));

import {
  ColorThemeKind,
  EventEmitter,
  __getCreatedFileSystemWatchers,
  __getLastCreatedWebviewPanel,
  __resetMockVscode,
  __setActiveColorTheme,
  __setConfigurationValue,
} from "./__mocks__/vscode";
import {
  PreviewManager,
  createArtifactMarkup,
  createPreviewHtml,
} from "./preview";

describe("PreviewManager", () => {
  afterEach(() => {
    vi.useRealTimers();
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("preview_panel_opens_beside_editor", async () => {
    const manager = new PreviewManager({
      executionManager: createMockExecutionManager(),
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.showPreview(Uri.file("/workspace/demo.tape"));

    const panel = __getLastCreatedWebviewPanel();
    expect(panel?.viewType).toBe("vhs-preview");
    expect(panel?.title).toBe("VHS Preview: demo.tape");
    expect(panel?.viewColumn).toBe(ViewColumn.Beside);
    expect(panel?.options.enableScripts).toBe(true);
    expect(panel?.options.retainContextWhenHidden).toBe(true);
    expect(panel?.options.localResourceRoots).toEqual(
      expect.arrayContaining([
        Uri.file("/workspace"),
        Uri.file("/extension/media"),
      ]),
    );
  });

  it("same_file_preview_reuses_existing_panel", async () => {
    const manager = new PreviewManager({
      executionManager: createMockExecutionManager(),
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });
    const tapeUri = Uri.file("/workspace/demo.tape");

    await manager.showPreview(tapeUri);
    const panel = __getLastCreatedWebviewPanel();

    await manager.showPreview(tapeUri);

    expect(panel?.reveal).toHaveBeenCalledWith(ViewColumn.Beside);
    expect(panel?.dispose).not.toHaveBeenCalled();
  });

  it("preview_run_posts_render_start_and_render_complete", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const executionManager = createMockExecutionManager({
      run: vi.fn().mockResolvedValue({
        artifactPath: "/workspace/demo.gif",
        format: "gif",
        tapeUri,
      }),
    });
    const manager = new PreviewManager({
      executionManager,
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.runAndPreview(tapeUri);

    const panel = __getLastCreatedWebviewPanel();
    expect(executionManager.run).toHaveBeenCalledWith(tapeUri);
    expect(panel?.webview.postedMessages).toEqual([
      {
        tapeFile: "demo.tape",
        type: "renderStart",
      },
      {
        artifactUri: "/workspace/demo.gif",
        format: "gif",
        type: "renderComplete",
      },
    ]);
  });

  it("output_changes_trigger_cache_busting_refresh_when_auto_refresh_is_enabled", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-20T05:17:00Z"));
    __setConfigurationValue("vhs-analyzer.preview.autoRefresh", true);

    const tapeUri = Uri.file("/workspace/demo.tape");
    const executionManager = createMockExecutionManager({
      run: vi.fn().mockResolvedValue({
        artifactPath: "/workspace/demo.gif",
        format: "gif",
        tapeUri,
      }),
    });
    const manager = new PreviewManager({
      executionManager,
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.runAndPreview(tapeUri);

    const watcher = __getCreatedFileSystemWatchers()[0];
    const panel = __getLastCreatedWebviewPanel();
    watcher?.__fireDidChange(Uri.file("/workspace/demo.gif"));
    await vi.advanceTimersByTimeAsync(500);

    expect(panel?.webview.postedMessages.at(-1)).toEqual({
      artifactUri: `/workspace/demo.gif?t=${Date.now()}`,
      format: "gif",
      type: "renderComplete",
    });
    vi.useRealTimers();
  });

  it("auto_refresh_disabled_skips_file_watcher_creation", async () => {
    __setConfigurationValue("vhs-analyzer.preview.autoRefresh", false);

    const tapeUri = Uri.file("/workspace/demo.tape");
    const manager = new PreviewManager({
      executionManager: createMockExecutionManager({
        run: vi.fn().mockResolvedValue({
          artifactPath: "/workspace/demo.gif",
          format: "gif",
          tapeUri,
        }),
      }),
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.runAndPreview(tapeUri);

    expect(__getCreatedFileSystemWatchers()).toHaveLength(0);
  });

  it("theme_changes_post_theme_change_messages_to_the_webview", async () => {
    const manager = new PreviewManager({
      executionManager: createMockExecutionManager(),
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.showPreview(Uri.file("/workspace/demo.tape"));

    __setActiveColorTheme(ColorThemeKind.Light);

    const panel = __getLastCreatedWebviewPanel();
    expect(panel?.webview.postedMessages.at(-1)).toEqual({
      kind: "light",
      type: "themeChange",
    });
  });

  it("missing_vhs_posts_install_friendly_error_without_starting_execution", async () => {
    const executionManager = createMockExecutionManager();
    const manager = new PreviewManager({
      executionManager,
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue(null),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.runAndPreview(Uri.file("/workspace/demo.tape"));

    const panel = __getLastCreatedWebviewPanel();
    expect(executionManager.run).not.toHaveBeenCalled();
    expect(panel?.webview.postedMessages.at(-1)).toEqual({
      cancelled: false,
      message: "VHS is not installed. Preview requires the VHS CLI tool.",
      type: "renderError",
    });
  });

  it("execution_failures_post_non_cancelled_error_messages", async () => {
    const executionManager = createMockExecutionManager({
      run: vi.fn().mockRejectedValue(new Error("Render failed.")),
    });
    const manager = new PreviewManager({
      executionManager,
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.runAndPreview(Uri.file("/workspace/demo.tape"));

    const panel = __getLastCreatedWebviewPanel();
    expect(panel?.webview.postedMessages.at(-1)).toEqual({
      cancelled: false,
      message: "Render failed.",
      type: "renderError",
    });
  });

  it("webview_cancel_requests_execution_cancellation_and_reports_cancelled_error", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const deferred = createDeferred<{
      artifactPath: string;
      format: "gif";
      tapeUri: Uri;
    }>();
    const executionManager = createMockExecutionManager({
      cancel: vi.fn().mockResolvedValue(true),
      run: vi.fn().mockReturnValue(deferred.promise),
    });
    const manager = new PreviewManager({
      executionManager,
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    const runPromise = manager.runAndPreview(tapeUri);
    await flushMicrotasks();
    const panel = __getLastCreatedWebviewPanel();

    panel?.webview.__fireDidReceiveMessage({ type: "cancel" });
    deferred.reject(
      Object.assign(new Error("Execution cancelled."), {
        cancelled: true,
      }),
    );
    await runPromise;

    expect(executionManager.cancel).toHaveBeenCalledWith(tapeUri);
    expect(panel?.webview.postedMessages.at(-1)).toEqual({
      cancelled: true,
      message: "Execution cancelled.",
      type: "renderError",
    });
  });

  it("closing_the_panel_cancels_any_running_execution", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const executionManager = createMockExecutionManager();
    const manager = new PreviewManager({
      executionManager,
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.showPreview(tapeUri);

    const panel = __getLastCreatedWebviewPanel();
    panel?.__fireDidDispose();

    expect(executionManager.cancel).toHaveBeenCalledWith(tapeUri);
  });

  it("stderr_progress_is_forwarded_to_the_webview", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const deferred = createDeferred<{
      artifactPath: string;
      format: "gif";
      tapeUri: Uri;
    }>();
    const executionManager = createMockExecutionManager({
      run: vi.fn().mockReturnValue(deferred.promise),
    });
    const manager = new PreviewManager({
      executionManager,
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    const runPromise = manager.runAndPreview(tapeUri);
    await flushMicrotasks();

    executionManager.__fireProgress({
      line: "Rendering frame 1",
      tapeUri,
    });

    const panel = __getLastCreatedWebviewPanel();
    expect(panel?.webview.postedMessages).toContainEqual({
      line: "Rendering frame 1",
      type: "renderProgress",
    });

    deferred.resolve({
      artifactPath: "/workspace/demo.gif",
      format: "gif",
      tapeUri,
    });
    await runPromise;
  });

  it("webview_ready_and_rerun_messages_use_the_typed_protocol", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const executionManager = createMockExecutionManager({
      run: vi
        .fn()
        .mockResolvedValueOnce({
          artifactPath: "/workspace/demo.gif",
          format: "gif",
          tapeUri,
        })
        .mockResolvedValueOnce({
          artifactPath: "/workspace/demo.gif",
          format: "gif",
          tapeUri,
        }),
    });
    const manager = new PreviewManager({
      executionManager,
      extensionPath: "/extension",
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      resolveExecutable: vi.fn().mockResolvedValue("/usr/bin/vhs"),
      workspaceFolders: [Uri.file("/workspace")],
    });

    await manager.showPreview(tapeUri);

    const panel = __getLastCreatedWebviewPanel();
    panel?.webview.__fireDidReceiveMessage({ type: "ready" });
    panel?.webview.__fireDidReceiveMessage({ type: "rerun" });
    await flushMicrotasks();

    expect(panel?.webview.postedMessages).toContainEqual({
      kind: "dark",
      type: "themeChange",
    });
    expect(executionManager.run).toHaveBeenCalledWith(tapeUri);
  });
});

describe("preview html helpers", () => {
  it("preview_html_contains_csp_state_containers_and_message_handling", () => {
    const html = createPreviewHtml({
      cspSource: "mock-webview-source",
      nonce: "nonce123",
      stylesheetUri: "/preview.css",
    });

    expect(html).toContain("Content-Security-Policy");
    expect(html).toContain("acquireVsCodeApi()");
    expect(html).toContain('addEventListener("message"');
    expect(html).toContain('id="prompt"');
    expect(html).toContain('id="loading"');
    expect(html).toContain('id="complete"');
    expect(html).toContain('id="error"');
    expect(html).toContain('id="retry-btn"');
    expect(html).toContain('id="install-vhs-link"');
  });

  it("artifact_markup_uses_img_for_gif_and_video_for_video_formats", () => {
    expect(createArtifactMarkup("/workspace/demo.gif", "gif")).toContain(
      "<img",
    );
    expect(createArtifactMarkup("/workspace/demo.mp4", "mp4")).toContain(
      "<video controls autoplay loop",
    );
    expect(createArtifactMarkup("/workspace/demo.webm", "webm")).toContain(
      "<video controls autoplay loop",
    );
  });
});

function createMockExecutionManager(
  overrides: Partial<{
    cancel: ReturnType<typeof vi.fn>;
    getState: ReturnType<typeof vi.fn>;
    run: ReturnType<typeof vi.fn>;
  }> = {},
) {
  const progressEmitter = new EventEmitter<{
    line: string;
    tapeUri: Uri;
  }>();

  return {
    cancel: overrides.cancel ?? vi.fn().mockResolvedValue(false),
    getState: overrides.getState ?? vi.fn(() => ({ kind: "idle" as const })),
    onDidWriteProgress: progressEmitter.event,
    run: overrides.run ?? vi.fn(),
    __fireProgress: (event: { line: string; tapeUri: Uri }) => {
      progressEmitter.fire(event);
    },
  };
}

function flushMicrotasks(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function createDeferred<T>() {
  let reject!: (reason?: unknown) => void;
  let resolve!: (value: T | PromiseLike<T>) => void;
  const promise = new Promise<T>((innerResolve, innerReject) => {
    reject = innerReject;
    resolve = innerResolve;
  });

  return {
    promise,
    reject,
    resolve,
  };
}
