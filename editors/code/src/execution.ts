import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { createInterface } from "node:readline";

import {
  type Event,
  EventEmitter,
  type Uri,
  commands,
  workspace,
} from "vscode";

import { extractOutputPath } from "./utils";

const cancellationGracePeriodMs = 3000;
const defaultOutputFileName = "out.gif";
const runningContextKey = "vhs-analyzer.isRunning";

export type OutputFormat = "gif" | "mp4" | "webm";

export type ExecutionState =
  | { kind: "idle" }
  | { kind: "running"; artifactPath: string; tapeUri: Uri }
  | { kind: "complete"; artifactPath: string; format: OutputFormat }
  | { kind: "error"; cancelled: boolean; message: string };

export interface ExecutionDependencies {
  clearScheduled: (timer: TimerHandle) => void;
  getWorkspaceFolders: () => readonly Uri[];
  now: () => number;
  outputChannel?: RunOutputChannelLike;
  readTapeFile: (tapeUri: Uri) => Promise<string>;
  schedule: (callback: () => void, delayMs: number) => TimerHandle;
  setContext: (key: string, value: boolean) => Promise<unknown>;
  spawnProcess: (
    command: string,
    args: readonly string[],
    options: { cwd: string },
  ) => SpawnedProcess;
}

export interface ExecutionProgressEvent {
  line: string;
  tapeUri: Uri;
}

export interface ExecutionResult {
  artifactPath: string;
  format: OutputFormat;
  tapeUri: Uri;
}

export interface ExecutionStateChangeEvent {
  state: ExecutionState;
  tapeUri: Uri;
}

export interface SpawnedProcess {
  readonly stderr: NodeJS.ReadableStream | null;
  readonly stdout: NodeJS.ReadableStream | null;
  kill(signal?: NodeJS.Signals): boolean;
  on(event: "error", listener: (error: Error) => void): this;
  on(event: "exit", listener: (code: number | null) => void): this;
}

export interface RunOutputChannelLike {
  appendLine(value: string): void;
  show(preserveFocus?: boolean): void;
}

type TimerHandle = ReturnType<typeof setTimeout>;

interface ActiveExecution {
  cancelTimer?: TimerHandle;
  cancelled: boolean;
  process: SpawnedProcess;
  readonly settle: () => void;
  readonly settled: Promise<void>;
  stderrLines: string[];
}

/**
 * Shared VHS execution engine used by preview and CodeLens flows.
 */
export class ExecutionManager {
  private readonly dependencies: ExecutionDependencies;
  private readonly progressEmitter = new EventEmitter<ExecutionProgressEvent>();
  private readonly running = new Map<string, ActiveExecution>();
  private readonly stateEmitter = new EventEmitter<ExecutionStateChangeEvent>();
  private readonly states = new Map<string, ExecutionState>();
  readonly onDidChangeState: Event<ExecutionStateChangeEvent> =
    this.stateEmitter.event;
  readonly onDidWriteProgress: Event<ExecutionProgressEvent> =
    this.progressEmitter.event;

  constructor(dependencies: Partial<ExecutionDependencies> = {}) {
    this.dependencies = {
      clearScheduled: clearTimeout,
      getWorkspaceFolders: () =>
        (workspace.workspaceFolders ?? []).map((folder) => folder.uri),
      now: Date.now,
      readTapeFile: async (tapeUri) => readFile(tapeUri.fsPath, "utf8"),
      schedule: setTimeout,
      setContext: async (key, value) =>
        commands.executeCommand("setContext", key, value),
      spawnProcess: (command, args, options) =>
        spawn(command, [...args], { cwd: options.cwd }),
      ...dependencies,
    };
  }

  async cancel(tapeUri: Uri): Promise<boolean> {
    const active = this.running.get(tapeUri.fsPath);
    if (active === undefined) {
      return false;
    }

    if (!active.cancelled) {
      active.cancelled = true;
      active.process.kill("SIGTERM");

      // SIGTERM first, SIGKILL after 3s — matches POSIX graceful shutdown convention.
      active.cancelTimer = this.dependencies.schedule(() => {
        if (this.running.get(tapeUri.fsPath) === active) {
          active.process.kill("SIGKILL");
        }
      }, cancellationGracePeriodMs);
      this.setState(tapeUri, { kind: "idle" });
      await this.updateRunningContext();
    }

    return true;
  }

  dispose(): void {
    this.progressEmitter.dispose();
    this.stateEmitter.dispose();
  }

  getState(tapeUri: Uri): ExecutionState {
    return this.states.get(tapeUri.fsPath) ?? { kind: "idle" };
  }

  async run(tapeUri: Uri): Promise<ExecutionResult> {
    const existing = this.running.get(tapeUri.fsPath);
    if (existing !== undefined) {
      await this.cancel(tapeUri);
      await existing.settled;
    }

    const workingDirectory = this.resolveWorkingDirectory(tapeUri);
    const tapeSource = await this.dependencies.readTapeFile(tapeUri);
    const artifactPath = resolveArtifactPath(
      tapeUri,
      workingDirectory,
      tapeSource,
    );
    const startedAt = this.dependencies.now();
    this.writeOutputLine(
      `[${formatTimestamp(startedAt)}] Running: vhs ${path.basename(tapeUri.fsPath)}`,
    );
    const process = this.dependencies.spawnProcess("vhs", [tapeUri.fsPath], {
      cwd: workingDirectory,
    });

    return new Promise<ExecutionResult>((resolve, reject) => {
      let settled = false;
      let settleRun!: () => void;
      const active: ActiveExecution = {
        cancelled: false,
        process,
        settle: () => {
          if (!settled) {
            settled = true;
            settleRun();
          }
        },
        settled: new Promise<void>((resolveSettled) => {
          settleRun = resolveSettled;
        }),
        stderrLines: [],
      };

      this.running.set(tapeUri.fsPath, active);
      this.setState(tapeUri, {
        artifactPath,
        kind: "running",
        tapeUri,
      });
      void this.updateRunningContext();
      this.attachOutputReaders(tapeUri, active);

      process.on("error", (error) => {
        void this.finishFailure({
          active,
          error,
          reject,
          startedAt,
          tapeUri,
        });
      });
      process.on("exit", (code) => {
        if (code === 0 && !active.cancelled) {
          void this.finishSuccess({
            active,
            artifactPath,
            resolve,
            startedAt,
            tapeUri,
          });
          return;
        }

        void this.finishExit({
          active,
          code,
          reject,
          startedAt,
          tapeUri,
        });
      });
    });
  }

  private attachOutputReaders(tapeUri: Uri, active: ActiveExecution): void {
    this.attachLineReader(active.process.stdout, (line) => {
      this.writeOutputLine(line);
    });
    this.attachLineReader(active.process.stderr, (line) => {
      active.stderrLines.push(line);
      this.progressEmitter.fire({ line, tapeUri });
      this.writeOutputLine(line);
    });
  }

  private async finishExit(options: {
    active: ActiveExecution;
    code: number | null;
    reject: (reason: ExecutionFailure) => void;
    startedAt: number;
    tapeUri: Uri;
  }): Promise<void> {
    const message =
      options.active.stderrLines.join("\n").trim() ||
      `VHS exited with code ${options.code ?? "unknown"}.`;

    if (options.active.cancelled) {
      this.cleanupActiveExecution(options.tapeUri, options.active);
      this.setState(options.tapeUri, { kind: "idle" });
      await this.updateRunningContext();
      this.writeCompletionFooter(options.startedAt, options.code);
      options.reject(
        new ExecutionFailure("Execution cancelled.", {
          cancelled: true,
          exitCode: options.code,
          stderr: message,
        }),
      );
      return;
    }

    this.cleanupActiveExecution(options.tapeUri, options.active);
    this.setState(options.tapeUri, {
      cancelled: false,
      kind: "error",
      message,
    });
    await this.updateRunningContext();
    this.writeCompletionFooter(options.startedAt, options.code);
    this.dependencies.outputChannel?.show(true);
    options.reject(
      new ExecutionFailure(message, {
        cancelled: false,
        exitCode: options.code,
        stderr: message,
      }),
    );
  }

  private async finishFailure(options: {
    active: ActiveExecution;
    error: Error;
    reject: (reason: ExecutionFailure) => void;
    startedAt: number;
    tapeUri: Uri;
  }): Promise<void> {
    const message =
      options.active.stderrLines.join("\n").trim() || options.error.message;

    this.cleanupActiveExecution(options.tapeUri, options.active);
    this.setState(options.tapeUri, {
      cancelled: false,
      kind: "error",
      message,
    });
    await this.updateRunningContext();
    this.writeCompletionFooter(options.startedAt, null);
    this.dependencies.outputChannel?.show(true);
    options.reject(
      new ExecutionFailure(message, {
        cancelled: false,
        exitCode: null,
        stderr: message,
      }),
    );
  }

  private async finishSuccess(options: {
    active: ActiveExecution;
    artifactPath: string;
    resolve: (result: ExecutionResult) => void;
    startedAt: number;
    tapeUri: Uri;
  }): Promise<void> {
    const format = inferOutputFormat(options.artifactPath);

    this.cleanupActiveExecution(options.tapeUri, options.active);
    this.setState(options.tapeUri, {
      artifactPath: options.artifactPath,
      format,
      kind: "complete",
    });
    await this.updateRunningContext();
    this.writeCompletionFooter(options.startedAt, 0);
    options.resolve({
      artifactPath: options.artifactPath,
      format,
      tapeUri: options.tapeUri,
    });
  }

  private cleanupActiveExecution(tapeUri: Uri, active: ActiveExecution): void {
    if (active.cancelTimer !== undefined) {
      this.dependencies.clearScheduled(active.cancelTimer);
    }

    this.running.delete(tapeUri.fsPath);
    active.settle();
  }

  private resolveWorkingDirectory(tapeUri: Uri): string {
    const workspaceFolder = this.dependencies
      .getWorkspaceFolders()
      .find((folder) => isWithinDirectory(folder.fsPath, tapeUri.fsPath));

    return workspaceFolder?.fsPath ?? path.dirname(tapeUri.fsPath);
  }

  private setState(tapeUri: Uri, state: ExecutionState): void {
    this.states.set(tapeUri.fsPath, state);
    this.stateEmitter.fire({ state, tapeUri });
  }

  private async updateRunningContext(): Promise<void> {
    await this.dependencies.setContext(
      runningContextKey,
      [...this.running.values()].some((active) => !active.cancelled),
    );
  }

  private attachLineReader(
    stream: NodeJS.ReadableStream | null,
    onLine: (line: string) => void,
  ): void {
    if (stream === null) {
      return;
    }

    const reader = createInterface({
      crlfDelay: Number.POSITIVE_INFINITY,
      input: stream,
    });
    reader.on("line", onLine);
  }

  private writeCompletionFooter(
    startedAt: number,
    exitCode: number | null,
  ): void {
    const elapsedMs = this.dependencies.now() - startedAt;
    const formattedExitCode = exitCode ?? "unknown";
    this.writeOutputLine(
      `[${formatTimestamp(this.dependencies.now())}] Completed in ${(
        elapsedMs / 1000
      ).toFixed(1)}s (exit code: ${formattedExitCode})`,
    );
  }

  private writeOutputLine(line: string): void {
    this.dependencies.outputChannel?.appendLine(line);
  }
}

export class ExecutionFailure extends Error {
  readonly cancelled: boolean;
  readonly exitCode: number | null;
  readonly stderr: string;

  constructor(
    message: string,
    options: {
      cancelled: boolean;
      exitCode: number | null;
      stderr: string;
    },
  ) {
    super(message);
    this.cancelled = options.cancelled;
    this.exitCode = options.exitCode;
    this.name = "ExecutionFailure";
    this.stderr = options.stderr;
  }
}

export function inferOutputFormat(artifactPath: string): OutputFormat {
  switch (path.extname(artifactPath).toLowerCase()) {
    case ".mp4":
      return "mp4";
    case ".webm":
      return "webm";
    default:
      return "gif";
  }
}

export function resolveArtifactPath(
  tapeUri: Uri,
  workingDirectory: string,
  tapeSource: string,
): string {
  const outputPath = extractOutputPath(tapeSource);

  if (outputPath === null) {
    return path.join(workingDirectory, defaultOutputFileName);
  }

  return path.resolve(workingDirectory, outputPath);
}

function isWithinDirectory(directoryPath: string, filePath: string): boolean {
  const relativePath = path.relative(directoryPath, filePath);
  return (
    relativePath === "" ||
    (!relativePath.startsWith("..") && !path.isAbsolute(relativePath))
  );
}

function formatTimestamp(timestamp: number): string {
  return new Date(timestamp).toISOString();
}
