import path from "node:path";

import {
  type CodeLens,
  type Disposable,
  type Event,
  EventEmitter,
  Range,
  type TextDocument,
  type Uri,
  commands,
  window,
  workspace,
} from "vscode";

import { getExtensionConfiguration } from "./config";
import type { ExecutionState, ExecutionStateChangeEvent } from "./execution";
import { OUTPUT_DIRECTIVE_REGEX } from "./utils";

export const runTapeCommandId = "vhs-analyzer.runTape";
export const previewTapeCommandId = "vhs-analyzer.previewTape";
export const stopRunningCommandId = "vhs-analyzer.stopRunning";

export interface CodeLensExecutionManagerLike {
  cancel?(tapeUri: Uri): Promise<boolean>;
  getState(tapeUri: Uri): ExecutionState;
  onDidChangeState: Event<ExecutionStateChangeEvent>;
  run?(tapeUri: Uri): Promise<unknown>;
}

export interface CodeLensPreviewManagerLike {
  runAndPreview(tapeUri: Uri, previewArtifactPath?: string): Promise<unknown>;
}

export interface ExecutionStatusIndicatorLike {
  clearExecutionStatus(): void;
  flashExecutionComplete(): void;
  flashExecutionFailed(): void;
  setExecutionStatus(fileName: string): void;
}

export class VhsCodeLensProvider {
  private readonly changeEmitter = new EventEmitter<void>();
  private readonly disposables: Disposable[] = [];
  readonly onDidChangeCodeLenses = this.changeEmitter.event;

  constructor(
    private readonly dependencies: {
      executionManager: CodeLensExecutionManagerLike;
      getConfiguration?: typeof getExtensionConfiguration;
    },
  ) {
    this.disposables.push(
      this.dependencies.executionManager.onDidChangeState(() => {
        this.changeEmitter.fire();
      }),
      workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration("vhs-analyzer.codelens")) {
          this.changeEmitter.fire();
        }
      }),
      workspace.onDidChangeTextDocument((event) => {
        if (event.document.languageId === "tape") {
          this.changeEmitter.fire();
        }
      }),
    );
  }

  provideCodeLenses(document: TextDocument): CodeLens[] {
    const configuration =
      this.dependencies.getConfiguration?.() ?? getExtensionConfiguration();
    if (!configuration.codelensEnabled) {
      return [];
    }

    const source = document.getText();
    const fileLevelLine = findFirstNonTrivialLine(source);
    const fileLevelRange = new Range(fileLevelLine, 0, fileLevelLine, 0);

    return [
      createCodeLens({
        arguments: [document.uri],
        command: runTapeCommandId,
        range: fileLevelRange,
        title: getRunTitle(
          this.dependencies.executionManager.getState(document.uri),
        ),
      }),
      createCodeLens({
        arguments: [document.uri],
        command: previewTapeCommandId,
        range: fileLevelRange,
        title: "▶ Run & Preview",
      }),
      ...collectOutputDirectiveLenses(source, document.uri),
    ];
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) {
      disposable.dispose();
    }
    this.changeEmitter.dispose();
  }
}

export function registerCodeLensCommands(options: {
  executionManager: CodeLensExecutionManagerLike;
  previewManager: CodeLensPreviewManagerLike;
}) {
  return [
    commands.registerCommand(
      runTapeCommandId,
      async (tapeUri?: Uri): Promise<void> => {
        const resolvedTapeUri = resolveCommandTapeUri(tapeUri);
        if (resolvedTapeUri === undefined) {
          return;
        }

        await options.executionManager.run?.(resolvedTapeUri);
      },
    ),
    commands.registerCommand(
      previewTapeCommandId,
      async (tapeUri?: Uri, previewArtifactPath?: string): Promise<void> => {
        const resolvedTapeUri = resolveCommandTapeUri(tapeUri);
        if (resolvedTapeUri === undefined) {
          return;
        }

        await options.previewManager.runAndPreview(
          resolvedTapeUri,
          previewArtifactPath,
        );
      },
    ),
    commands.registerCommand(
      stopRunningCommandId,
      async (tapeUri?: Uri): Promise<void> => {
        const resolvedTapeUri = resolveCommandTapeUri(tapeUri);
        if (resolvedTapeUri === undefined) {
          return;
        }

        await options.executionManager.cancel?.(resolvedTapeUri);
      },
    ),
  ];
}

export function bindExecutionStateToStatusBar(options: {
  executionManager: Pick<CodeLensExecutionManagerLike, "onDidChangeState">;
  status: ExecutionStatusIndicatorLike;
}): Disposable {
  return options.executionManager.onDidChangeState(({ state, tapeUri }) => {
    switch (state.kind) {
      case "running":
        options.status.setExecutionStatus(path.basename(tapeUri.fsPath));
        return;
      case "complete":
        options.status.flashExecutionComplete();
        return;
      case "error":
        options.status.flashExecutionFailed();
        return;
      case "idle":
        options.status.clearExecutionStatus();
        return;
    }
  });
}

function createCodeLens(options: {
  arguments: unknown[];
  command: string;
  range: Range;
  title: string;
}): CodeLens {
  return {
    command: {
      arguments: options.arguments,
      command: options.command,
      title: options.title,
    },
    isResolved: true,
    range: options.range,
  };
}

function collectOutputDirectiveLenses(
  source: string,
  tapeUri: Uri,
): CodeLens[] {
  return source.split(/\r?\n/u).flatMap((line, lineIndex) => {
    const match = line.match(OUTPUT_DIRECTIVE_REGEX);
    const outputPath = match?.[1] ?? match?.[2];
    if (outputPath === undefined) {
      return [];
    }

    return [
      createCodeLens({
        arguments: [tapeUri, outputPath],
        command: previewTapeCommandId,
        range: new Range(lineIndex, 0, lineIndex, 0),
        title: `▶ Preview ${path.basename(outputPath)}`,
      }),
    ];
  });
}

function findFirstNonTrivialLine(source: string): number {
  const lines = source.split(/\r?\n/u);
  const firstContentLine = lines.findIndex((line) => {
    const trimmed = line.trim();
    return trimmed !== "" && !trimmed.startsWith("#");
  });

  return firstContentLine === -1 ? 0 : firstContentLine;
}

function getRunTitle(state: ExecutionState): string {
  switch (state.kind) {
    case "running":
      return "$(sync~spin) Running...";
    case "complete":
      return "✓ Done — ▶ Re-run";
    default:
      return "▶ Run this tape";
  }
}

function resolveCommandTapeUri(tapeUri?: Uri): Uri | undefined {
  if (tapeUri !== undefined) {
    return tapeUri;
  }

  const activeEditor = window.activeTextEditor;
  if (activeEditor?.document.languageId !== "tape") {
    return undefined;
  }

  return activeEditor.document.uri;
}
