import { afterEach, describe, expect, it, vi } from "vitest";
import type { ExtensionContext } from "vscode";

import {
  Position,
  type ThemeColor,
  Uri,
  __fireConfigurationChange,
  __fireTextDocumentChange,
  __getRegisteredCodeLensProviders,
  __resetMockVscode,
  __setActiveTextEditor,
  __setConfigurationValue,
  commands,
  createMockExtensionContext,
  languages,
  window,
  workspace,
} from "./__mocks__/vscode";
import {
  previewTapeCommandId,
  runTapeCommandId,
  stopRunningCommandId,
} from "./codelens";
import {
  ExtensionController,
  type LanguageClientLike,
  buildLanguageClientOptions,
} from "./extension";
import { ClientTransportKind } from "./server";
import { StatusController } from "./status";

function flushMicrotasks(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

describe("buildLanguageClientOptions", () => {
  afterEach(() => {
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("language_client_options_include_batch1_contract_values", () => {
    const fileWatcher = workspace.createFileSystemWatcher(
      "**/*.tape",
    ) as unknown as import("vscode").FileSystemWatcher;
    const outputChannel = window.createOutputChannel("output");
    const traceOutputChannel = window.createOutputChannel("trace");
    const errorHandler = {
      error: vi.fn(),
      closed: vi.fn(),
    } as never;
    const initializationFailedHandler = vi.fn(() => false);

    const options = buildLanguageClientOptions({
      errorHandler,
      fileWatcher,
      initializationFailedHandler,
      outputChannel,
      traceOutputChannel,
    });

    expect(options.documentSelector).toEqual([
      { scheme: "file", language: "tape" },
    ]);
    expect(options.synchronize).toEqual({
      fileEvents: fileWatcher,
    });
    expect(options.outputChannel).toBe(outputChannel);
    expect(options.traceOutputChannel).toBe(traceOutputChannel);
  });
});

describe("StatusController", () => {
  afterEach(() => {
    vi.useRealTimers();
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("status_bar_indicator_updates_for_server_states", () => {
    const status = new StatusController({
      onRestartServer: vi.fn(),
      onShowOutput: vi.fn(),
      onShowTrace: vi.fn(),
    });
    const item = status.statusBarItem;

    status.setServerStatus("running");
    expect(item.text).toBe("VHS $(check)");
    expect((item.color as ThemeColor).id).toBe("charts.green");

    status.setServerStatus("starting");
    expect(item.text).toBe("VHS $(warning)");
    expect((item.color as ThemeColor).id).toBe("charts.yellow");

    status.setServerStatus("failed");
    expect(item.text).toBe("VHS $(error)");
    expect((item.color as ThemeColor).id).toBe("charts.red");

    status.setExecutionStatus("demo.tape");
    expect(item.text).toBe("$(sync~spin) VHS: Running demo.tape...");
  });

  it("status_bar_quick_pick_dispatches_selected_action", async () => {
    const onRestartServer = vi.fn();
    const onShowOutput = vi.fn();
    const onShowTrace = vi.fn();
    const status = new StatusController({
      onRestartServer,
      onShowOutput,
      onShowTrace,
    });

    window.showQuickPick.mockResolvedValueOnce("Restart Server");
    await status.showActions();
    expect(onRestartServer).toHaveBeenCalledTimes(1);

    window.showQuickPick.mockResolvedValueOnce("Show Output");
    await status.showActions();
    expect(onShowOutput).toHaveBeenCalledTimes(1);

    window.showQuickPick.mockResolvedValueOnce("Show Trace");
    await status.showActions();
    expect(onShowTrace).toHaveBeenCalledTimes(1);
  });

  it("status_bar_execution_flashes_restore_the_previous_server_state", async () => {
    vi.useFakeTimers();
    const status = new StatusController({
      onRestartServer: vi.fn(),
      onShowOutput: vi.fn(),
      onShowTrace: vi.fn(),
    });
    const item = status.statusBarItem;

    status.setServerStatus("running");
    status.setExecutionStatus("demo.tape");
    expect(item.text).toBe("$(sync~spin) VHS: Running demo.tape...");

    status.flashExecutionComplete();
    expect(item.text).toBe("$(check) VHS: Done");
    await vi.advanceTimersByTimeAsync(3000);
    expect(item.text).toBe("VHS $(check)");

    status.setExecutionStatus("demo.tape");
    status.flashExecutionFailed();
    expect(item.text).toBe("$(error) VHS: Failed");
    await vi.advanceTimersByTimeAsync(5000);
    expect(item.text).toBe("VHS $(check)");
    vi.useRealTimers();
  });
});

describe("ExtensionController", () => {
  afterEach(() => {
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("activation_starts_client_when_binary_is_available", async () => {
    __setConfigurationValue("vhs-analyzer.server.args", ["--log=debug"]);
    __setConfigurationValue("vhs-analyzer.trace.server", "verbose");
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const client = createMockClient();
    const checkDependencies = vi.fn().mockResolvedValue(undefined);
    const createLanguageClient = vi.fn().mockResolvedValue(client);

    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: checkDependencies,
      createLanguageClient,
      discoverServerBinary: vi
        .fn()
        .mockResolvedValue("/extension/server/vhs-analyzer"),
      isExecutableFile: vi.fn().mockResolvedValue(true),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();

    expect(createLanguageClient).toHaveBeenCalledTimes(1);
    expect(createLanguageClient.mock.calls[0]?.[0]).toEqual({
      command: "/extension/server/vhs-analyzer",
      args: ["--log=debug"],
      options: {
        env: {
          ...process.env,
          NO_COLOR: "1",
        },
      },
      transport: ClientTransportKind.stdio,
    });
    expect(client.start).toHaveBeenCalledTimes(1);
    expect(client.setTrace).toHaveBeenCalledWith("verbose");
    expect(checkDependencies).toHaveBeenCalledTimes(1);
    expect(window.createStatusBarItem.mock.results[0]?.value.text).toBe(
      "VHS $(check)",
    );
  });

  it("activation_enters_no_server_mode_when_binary_is_missing", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const createLanguageClient = vi.fn();
    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient,
      discoverServerBinary: vi.fn().mockResolvedValue(null),
      isExecutableFile: vi.fn().mockResolvedValue(false),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();

    expect(createLanguageClient).not.toHaveBeenCalled();
    expect(window.showInformationMessage).toHaveBeenCalledWith(
      "VHS Analyzer LSP server not found. Install the platform-specific extension for full language support.",
      "Install",
      "Don't show again",
    );
    expect(window.createStatusBarItem.mock.results[0]?.value.text).toBe(
      "VHS $(error)",
    );
  });

  it("activation_registers_codelens_provider_and_commands_in_no_server_mode", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn(),
      discoverServerBinary: vi.fn().mockResolvedValue(null),
      isExecutableFile: vi.fn().mockResolvedValue(false),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();

    expect(languages.registerCodeLensProvider).toHaveBeenCalledWith(
      { language: "tape" },
      expect.anything(),
    );
    expect(__getRegisteredCodeLensProviders()).toHaveLength(1);
    expect(commands.registerCommand).toHaveBeenCalledWith(
      runTapeCommandId,
      expect.any(Function),
    );
    expect(commands.registerCommand).toHaveBeenCalledWith(
      previewTapeCommandId,
      expect.any(Function),
    );
    expect(commands.registerCommand).toHaveBeenCalledWith(
      stopRunningCommandId,
      expect.any(Function),
    );
  });

  it("no_server_notification_is_suppressed_after_dismissal", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    window.showInformationMessage.mockResolvedValueOnce("Don't show again");

    const firstController = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn(),
      discoverServerBinary: vi.fn().mockResolvedValue(null),
      isExecutableFile: vi.fn().mockResolvedValue(false),
      scheduleRestart: vi.fn(),
    });

    await firstController.activate();

    window.showInformationMessage.mockClear();

    const secondController = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn(),
      discoverServerBinary: vi.fn().mockResolvedValue(null),
      isExecutableFile: vi.fn().mockResolvedValue(false),
      scheduleRestart: vi.fn(),
    });

    await secondController.activate();

    expect(window.showInformationMessage).not.toHaveBeenCalled();
  });

  it("deactivate_stops_the_running_client", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const client = createMockClient();
    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn().mockResolvedValue(client),
      discoverServerBinary: vi
        .fn()
        .mockResolvedValue("/extension/server/vhs-analyzer"),
      isExecutableFile: vi.fn().mockResolvedValue(true),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();
    await controller.deactivate();

    expect(client.stop).toHaveBeenCalledTimes(1);
  });

  it("config_change_to_invalid_server_path_keeps_the_client_stopped", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const client = createMockClient();
    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn().mockResolvedValue(client),
      discoverServerBinary: vi
        .fn()
        .mockResolvedValue("/extension/server/vhs-analyzer"),
      isExecutableFile: vi.fn().mockResolvedValue(false),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();
    client.stop.mockClear();

    __setConfigurationValue(
      "vhs-analyzer.server.path",
      "/missing/vhs-analyzer",
    );
    window.showErrorMessage.mockResolvedValueOnce(undefined);
    __fireConfigurationChange(["vhs-analyzer.server.path"]);
    await flushMicrotasks();

    expect(client.stop).toHaveBeenCalledTimes(1);
    expect(window.showErrorMessage).toHaveBeenCalledWith(
      "Configured VHS Analyzer server not found at /missing/vhs-analyzer.",
      "Restart Server",
    );
    expect(window.createStatusBarItem.mock.results[0]?.value.text).toBe(
      "VHS $(error)",
    );
  });

  it("trace_change_updates_the_running_client_without_restart", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const client = createMockClient();
    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn().mockResolvedValue(client),
      discoverServerBinary: vi
        .fn()
        .mockResolvedValue("/extension/server/vhs-analyzer"),
      isExecutableFile: vi.fn().mockResolvedValue(true),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();
    client.setTrace.mockClear();
    client.start.mockClear();

    __setConfigurationValue("vhs-analyzer.trace.server", "messages");
    __fireConfigurationChange(["vhs-analyzer.trace.server"]);
    await flushMicrotasks();

    expect(client.setTrace).toHaveBeenCalledWith("messages");
    expect(client.start).not.toHaveBeenCalled();
  });

  it("targeted_suggest_triggers_after_set_theme_space", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const client = createMockClient();
    const uri = Uri.file("/workspace/demo.tape");
    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn().mockResolvedValue(client),
      discoverServerBinary: vi
        .fn()
        .mockResolvedValue("/extension/server/vhs-analyzer"),
      isExecutableFile: vi.fn().mockResolvedValue(true),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();
    commands.executeCommand.mockClear();
    __setActiveTextEditor({
      getText: () => "Set Theme ",
      languageId: "tape",
      selection: new Position(0, 10),
      uri,
    });

    __fireTextDocumentChange({
      contentChanges: [{ text: " " }],
      languageId: "tape",
      uri,
    });

    expect(commands.executeCommand).toHaveBeenCalledWith(
      "editor.action.triggerSuggest",
      { auto: true },
    );
  });

  it("targeted_suggest_triggers_inside_empty_theme_quotes", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const client = createMockClient();
    const uri = Uri.file("/workspace/demo.tape");
    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn().mockResolvedValue(client),
      discoverServerBinary: vi
        .fn()
        .mockResolvedValue("/extension/server/vhs-analyzer"),
      isExecutableFile: vi.fn().mockResolvedValue(true),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();
    commands.executeCommand.mockClear();
    __setActiveTextEditor({
      getText: () => 'Set Theme ""',
      languageId: "tape",
      selection: new Position(0, 11),
      uri,
    });

    __fireTextDocumentChange({
      contentChanges: [{ text: '"' }],
      languageId: "tape",
      uri,
    });

    expect(commands.executeCommand).toHaveBeenCalledWith(
      "editor.action.triggerSuggest",
      { auto: true },
    );
  });

  it("targeted_suggest_triggers_after_first_time_digit_in_supported_contexts", async () => {
    const context = createTypedContext({
      extensionPath: "/extension",
    });
    const client = createMockClient();
    const uri = Uri.file("/workspace/demo.tape");
    const controller = new ExtensionController(context, {
      checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
      createLanguageClient: vi.fn().mockResolvedValue(client),
      discoverServerBinary: vi
        .fn()
        .mockResolvedValue("/extension/server/vhs-analyzer"),
      isExecutableFile: vi.fn().mockResolvedValue(true),
      scheduleRestart: vi.fn(),
    });

    await controller.activate();
    commands.executeCommand.mockClear();

    __setActiveTextEditor({
      getText: () => "Sleep 1",
      languageId: "tape",
      selection: new Position(0, 7),
      uri,
    });
    __fireTextDocumentChange({
      contentChanges: [{ text: "1" }],
      languageId: "tape",
      uri,
    });

    __setActiveTextEditor({
      getText: () => 'Type@1 "x"',
      languageId: "tape",
      selection: new Position(0, 6),
      uri,
    });
    __fireTextDocumentChange({
      contentChanges: [{ text: "1" }],
      languageId: "tape",
      uri,
    });

    __setActiveTextEditor({
      getText: () => "Set TypingSpeed 1",
      languageId: "tape",
      selection: new Position(0, 17),
      uri,
    });
    __fireTextDocumentChange({
      contentChanges: [{ text: "1" }],
      languageId: "tape",
      uri,
    });

    expect(commands.executeCommand).toHaveBeenCalledTimes(3);
    expect(commands.executeCommand).toHaveBeenNthCalledWith(
      1,
      "editor.action.triggerSuggest",
      { auto: true },
    );
    expect(commands.executeCommand).toHaveBeenNthCalledWith(
      2,
      "editor.action.triggerSuggest",
      { auto: true },
    );
    expect(commands.executeCommand).toHaveBeenNthCalledWith(
      3,
      "editor.action.triggerSuggest",
      { auto: true },
    );
  });
});

function createMockClient(): LanguageClientLike & {
  setTrace: ReturnType<typeof vi.fn>;
  start: ReturnType<typeof vi.fn>;
  stop: ReturnType<typeof vi.fn>;
} {
  return {
    setTrace: vi.fn().mockResolvedValue(undefined),
    start: vi.fn().mockResolvedValue(undefined),
    stop: vi.fn().mockResolvedValue(undefined),
  };
}

function createTypedContext(overrides: {
  extensionPath?: string;
  globalState?: Record<string, unknown>;
}): ExtensionContext {
  return createMockExtensionContext(overrides) as unknown as ExtensionContext;
}
