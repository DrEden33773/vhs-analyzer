import { readFileSync } from "node:fs";
import path from "node:path";

import { afterEach, describe, expect, it, vi } from "vitest";
import type { TextDocument } from "vscode";
import { Uri } from "vscode";

vi.mock("vscode", async () => import("./__mocks__/vscode.js"));

import {
  EventEmitter,
  __fireConfigurationChange,
  __fireTextDocumentChange,
  __resetMockVscode,
  __setConfigurationValue,
  commands,
  window,
} from "./__mocks__/vscode";
import {
  VhsCodeLensProvider,
  bindExecutionStateToStatusBar,
  previewTapeCommandId,
  registerCodeLensCommands,
  runTapeCommandId,
  stopRunningCommandId,
} from "./codelens";
import type { ExecutionState, ExecutionStateChangeEvent } from "./execution";

describe("VhsCodeLensProvider", () => {
  afterEach(() => {
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("codelens_placed_at_first_non_trivial_line", () => {
    const executionStateEmitter = new EventEmitter<ExecutionStateChangeEvent>();
    const provider = new VhsCodeLensProvider({
      executionManager: {
        getState: vi.fn(() => ({ kind: "idle" as const })),
        onDidChangeState: executionStateEmitter.event,
      },
    });
    const document = createMockDocument(
      '\n# leading comment\n\nOutput demo.gif\nType "hello"',
    );

    const lenses = provider.provideCodeLenses(document);

    expect(lenses).toHaveLength(3);
    expect(lenses[0]?.range.start.line).toBe(3);
    expect(lenses[1]?.range.start.line).toBe(3);
    expect(lenses.every((lens) => lens.command !== undefined)).toBe(true);
    expect(lenses.every((lens) => lens.isResolved !== false)).toBe(true);
  });

  it("codelens_defaults_to_line_zero_without_leading_comments", () => {
    const provider = createProvider();
    const document = createMockDocument('Output demo.gif\nType "hello"');

    const lenses = provider.provideCodeLenses(document);

    expect(lenses[0]?.range.start.line).toBe(0);
    expect(lenses[1]?.range.start.line).toBe(0);
  });

  it("codelens_adds_output_level_preview_lenses_for_each_output_directive", () => {
    const provider = createProvider();
    const document = createMockDocument(
      'Output first.gif\nType "one"\nOutput "nested/second demo.mp4"',
    );

    const lenses = provider.provideCodeLenses(document);

    expect(lenses).toHaveLength(4);
    expect(lenses[2]?.command?.title).toBe("▶ Preview first.gif");
    expect(lenses[2]?.range.start.line).toBe(0);
    expect(lenses[3]?.command?.title).toBe("▶ Preview second demo.mp4");
    expect(lenses[3]?.range.start.line).toBe(2);
    expect(lenses[3]?.command?.arguments).toEqual([
      Uri.file("/workspace/demo.tape"),
      "nested/second demo.mp4",
    ]);
  });

  it("codelens_returns_only_file_level_actions_when_output_is_missing", () => {
    const provider = createProvider();
    const document = createMockDocument('Set Shell "bash"\nType "hello"');

    const lenses = provider.provideCodeLenses(document);

    expect(lenses).toHaveLength(2);
  });

  it("codelens_returns_no_lenses_when_disabled", () => {
    __setConfigurationValue("vhs-analyzer.codelens.enabled", false);
    const provider = createProvider();

    expect(
      provider.provideCodeLenses(createMockDocument("Output demo.gif")),
    ).toEqual([]);
  });

  it("codelens_titles_follow_execution_state", () => {
    const document = createMockDocument("Output demo.gif");
    const runningProvider = createProvider({
      getState: () => ({
        artifactPath: "/workspace/demo.gif",
        kind: "running",
        tapeUri: document.uri,
      }),
    });
    const completeProvider = createProvider({
      getState: () => ({
        artifactPath: "/workspace/demo.gif",
        format: "gif",
        kind: "complete",
      }),
    });

    expect(runningProvider.provideCodeLenses(document)[0]?.command?.title).toBe(
      "$(sync~spin) Running...",
    );
    expect(
      completeProvider.provideCodeLenses(document)[0]?.command?.title,
    ).toBe("✓ Done — ▶ Re-run");
  });

  it("codelens_change_event_fires_for_execution_state_and_configuration_changes", () => {
    const executionStateEmitter = new EventEmitter<ExecutionStateChangeEvent>();
    const provider = new VhsCodeLensProvider({
      executionManager: {
        getState: vi.fn(() => ({ kind: "idle" as const })),
        onDidChangeState: executionStateEmitter.event,
      },
    });
    const listener = vi.fn();

    provider.onDidChangeCodeLenses(listener);

    executionStateEmitter.fire({
      state: { kind: "idle" },
      tapeUri: Uri.file("/workspace/demo.tape"),
    });
    __fireConfigurationChange(["vhs-analyzer.codelens.enabled"]);

    expect(listener).toHaveBeenCalledTimes(2);
  });

  it("codelens_change_event_fires_when_tape_documents_change", () => {
    const provider = createProvider();
    const listener = vi.fn();

    provider.onDidChangeCodeLenses(listener);
    __fireTextDocumentChange({
      languageId: "tape",
      uri: Uri.file("/workspace/demo.tape"),
    });

    expect(listener).toHaveBeenCalledTimes(1);
  });

  it("registered_commands_delegate_to_execution_and_preview_managers", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const executionManager = {
      cancel: vi.fn().mockResolvedValue(true),
      getState: vi.fn(() => ({ kind: "idle" as const })),
      onDidChangeState: new EventEmitter<ExecutionStateChangeEvent>().event,
      run: vi.fn().mockResolvedValue(undefined),
    };
    const previewManager = {
      runAndPreview: vi.fn().mockResolvedValue(undefined),
    };

    registerCodeLensCommands({
      executionManager,
      previewManager,
    });

    await commands.executeCommand(runTapeCommandId, tapeUri);
    await commands.executeCommand(
      previewTapeCommandId,
      tapeUri,
      "nested/output.mp4",
    );
    await commands.executeCommand(stopRunningCommandId, tapeUri);

    expect(executionManager.run).toHaveBeenCalledWith(tapeUri);
    expect(previewManager.runAndPreview).toHaveBeenCalledWith(
      tapeUri,
      "nested/output.mp4",
    );
    expect(executionManager.cancel).toHaveBeenCalledWith(tapeUri);
  });

  it("registered_commands_fall_back_to_the_active_tape_editor", async () => {
    const tapeUri = Uri.file("/workspace/active-demo.tape");
    const executionManager = {
      cancel: vi.fn().mockResolvedValue(true),
      getState: vi.fn(() => ({ kind: "idle" as const })),
      onDidChangeState: new EventEmitter<ExecutionStateChangeEvent>().event,
      run: vi.fn().mockResolvedValue(undefined),
    };
    const previewManager = {
      runAndPreview: vi.fn().mockResolvedValue(undefined),
    };

    registerCodeLensCommands({
      executionManager,
      previewManager,
    });
    (
      window as typeof window & {
        activeTextEditor?: {
          document: { languageId: string; uri: Uri };
        };
      }
    ).activeTextEditor = {
      document: {
        languageId: "tape",
        uri: tapeUri,
      },
    };

    await commands.executeCommand(runTapeCommandId);
    await commands.executeCommand(previewTapeCommandId);
    await commands.executeCommand(stopRunningCommandId);

    expect(executionManager.run).toHaveBeenCalledWith(tapeUri);
    expect(previewManager.runAndPreview).toHaveBeenCalledWith(
      tapeUri,
      undefined,
    );
    expect(executionManager.cancel).toHaveBeenCalledWith(tapeUri);
  });

  it("execution_state_changes_drive_status_bar_feedback", () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const executionStateEmitter = new EventEmitter<ExecutionStateChangeEvent>();
    const status = {
      clearExecutionStatus: vi.fn(),
      flashExecutionComplete: vi.fn(),
      flashExecutionFailed: vi.fn(),
      setExecutionStatus: vi.fn(),
    };

    bindExecutionStateToStatusBar({
      executionManager: {
        onDidChangeState: executionStateEmitter.event,
      },
      status,
    });

    executionStateEmitter.fire({
      state: {
        artifactPath: "/workspace/demo.gif",
        kind: "running",
        tapeUri,
      },
      tapeUri,
    });
    executionStateEmitter.fire({
      state: {
        artifactPath: "/workspace/demo.gif",
        format: "gif",
        kind: "complete",
      },
      tapeUri,
    });
    executionStateEmitter.fire({
      state: {
        cancelled: false,
        kind: "error",
        message: "Boom",
      },
      tapeUri,
    });
    executionStateEmitter.fire({
      state: { kind: "idle" },
      tapeUri,
    });

    expect(status.setExecutionStatus).toHaveBeenCalledWith("demo.tape");
    expect(status.flashExecutionComplete).toHaveBeenCalledTimes(1);
    expect(status.flashExecutionFailed).toHaveBeenCalledTimes(1);
    expect(status.clearExecutionStatus).toHaveBeenCalledTimes(1);
  });

  it("package_manifest_declares_codelens_commands_menus_and_shortcut", () => {
    const packageJson = JSON.parse(
      readFileSync(path.join(process.cwd(), "package.json"), "utf8"),
    ) as {
      contributes: {
        commands: Array<{ command: string }>;
        keybindings: Array<{ command: string; when: string }>;
        menus: {
          "editor/context": Array<{ command: string; when: string }>;
          "explorer/context": Array<{ command: string; when: string }>;
        };
      };
    };

    expect(packageJson.contributes.commands).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ command: runTapeCommandId }),
        expect.objectContaining({ command: previewTapeCommandId }),
        expect.objectContaining({ command: stopRunningCommandId }),
      ]),
    );
    expect(packageJson.contributes.menus["editor/context"]).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          command: runTapeCommandId,
          when: "editorLangId == 'tape'",
        }),
        expect.objectContaining({
          command: previewTapeCommandId,
          when: "editorLangId == 'tape'",
        }),
      ]),
    );
    expect(packageJson.contributes.menus["explorer/context"]).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          command: runTapeCommandId,
          when: "resourceExtname == '.tape'",
        }),
      ]),
    );
    expect(packageJson.contributes.keybindings).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          command: runTapeCommandId,
          when: "editorLangId == 'tape'",
        }),
      ]),
    );
  });
});

function createMockDocument(source: string) {
  return {
    getText: () => source,
    uri: Uri.file("/workspace/demo.tape"),
  } as unknown as TextDocument;
}

function createProvider(
  overrides: Partial<{
    getState: (tapeUri: Uri) => ExecutionState;
  }> = {},
) {
  const executionStateEmitter = new EventEmitter<ExecutionStateChangeEvent>();

  return new VhsCodeLensProvider({
    executionManager: {
      getState: overrides.getState ?? createProviderExecutionState,
      onDidChangeState: executionStateEmitter.event,
    },
  });
}

function createProviderExecutionState() {
  return { kind: "idle" as const } satisfies ExecutionState;
}
