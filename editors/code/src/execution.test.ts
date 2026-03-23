import { EventEmitter } from "node:events";
import { PassThrough } from "node:stream";

import { afterEach, describe, expect, it, vi } from "vitest";
import { Uri } from "vscode";

vi.mock("vscode", async () => import("./__mocks__/vscode.js"));

import { __getCommandContext, __resetMockVscode } from "./__mocks__/vscode";
import { ExecutionManager } from "./execution";

function flushMicrotasks(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

describe("ExecutionManager", () => {
  afterEach(() => {
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("execution_run_transitions_to_complete_and_updates_running_context", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const childProcess = new MockChildProcess();
    const manager = new ExecutionManager({
      getWorkspaceFolders: () => [Uri.file("/workspace")],
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      spawnProcess: vi.fn().mockReturnValue(childProcess),
    });

    const runPromise = manager.run(tapeUri);
    await flushMicrotasks();

    expect(manager.getState(tapeUri)).toEqual({
      artifactPath: "/workspace/demo.gif",
      kind: "running",
      tapeUri,
    });
    expect(__getCommandContext("vhs-analyzer.isRunning")).toBe(true);

    childProcess.exit(0);

    await expect(runPromise).resolves.toEqual({
      artifactPath: "/workspace/demo.gif",
      format: "gif",
      tapeUri,
    });
    expect(manager.getState(tapeUri)).toEqual({
      artifactPath: "/workspace/demo.gif",
      format: "gif",
      kind: "complete",
    });
    expect(__getCommandContext("vhs-analyzer.isRunning")).toBe(false);
  });

  it("execution_cancel_sends_sigterm_then_sigkill_and_returns_to_idle", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const childProcess = new MockChildProcess();
    const scheduledCallbacks: Array<{ callback: () => void; delayMs: number }> =
      [];
    const timerHandle = setTimeout(() => {}, 0);
    clearTimeout(timerHandle);
    const clearScheduled = vi.fn();
    const manager = new ExecutionManager({
      clearScheduled,
      getWorkspaceFolders: () => [Uri.file("/workspace")],
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      schedule: (callback: () => void, delayMs: number) => {
        scheduledCallbacks.push({ callback, delayMs });
        return timerHandle;
      },
      spawnProcess: vi.fn().mockReturnValue(childProcess),
    });

    const runPromise = manager.run(tapeUri);
    await flushMicrotasks();

    await expect(manager.cancel(tapeUri)).resolves.toBe(true);
    expect(childProcess.kill).toHaveBeenCalledWith("SIGTERM");
    expect(scheduledCallbacks).toHaveLength(1);
    expect(scheduledCallbacks[0]?.delayMs).toBe(3000);
    expect(manager.getState(tapeUri)).toEqual({ kind: "idle" });
    expect(__getCommandContext("vhs-analyzer.isRunning")).toBe(false);

    scheduledCallbacks[0]?.callback();
    expect(childProcess.kill).toHaveBeenLastCalledWith("SIGKILL");

    childProcess.exit(1);

    await expect(runPromise).rejects.toMatchObject({
      cancelled: true,
      name: "ExecutionFailure",
    });
    expect(manager.getState(tapeUri)).toEqual({ kind: "idle" });
    expect(clearScheduled).toHaveBeenCalledWith(timerHandle);
  });

  it("execution_uses_tape_directory_cwd_and_defaults_to_out_gif_without_output_directive", async () => {
    const tapeUri = Uri.file("/workspace/nested/demo.tape");
    const childProcess = new MockChildProcess();
    const spawnProcess = vi.fn().mockReturnValue(childProcess);
    const manager = new ExecutionManager({
      getWorkspaceFolders: () => [Uri.file("/workspace")],
      readTapeFile: vi.fn().mockResolvedValue('Set Shell "bash"'),
      spawnProcess,
    });

    const runPromise = manager.run(tapeUri);
    await flushMicrotasks();

    expect(spawnProcess).toHaveBeenCalledWith("vhs", [tapeUri.fsPath], {
      cwd: "/workspace/nested",
    });
    expect(manager.getState(tapeUri)).toEqual({
      artifactPath: "/workspace/nested/out.gif",
      kind: "running",
      tapeUri,
    });

    childProcess.exit(0);

    await expect(runPromise).resolves.toEqual({
      artifactPath: "/workspace/nested/out.gif",
      format: "gif",
      tapeUri,
    });
  });

  it("execution_resolves_relative_output_paths_against_the_tape_directory", async () => {
    const tapeUri = Uri.file("/workspace/nested/demo.tape");
    const childProcess = new MockChildProcess();
    const spawnProcess = vi.fn().mockReturnValue(childProcess);
    const manager = new ExecutionManager({
      getWorkspaceFolders: () => [Uri.file("/workspace")],
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      spawnProcess,
    });

    const runPromise = manager.run(tapeUri);
    await flushMicrotasks();

    expect(spawnProcess).toHaveBeenCalledWith("vhs", [tapeUri.fsPath], {
      cwd: "/workspace/nested",
    });
    expect(manager.getState(tapeUri)).toEqual({
      artifactPath: "/workspace/nested/demo.gif",
      kind: "running",
      tapeUri,
    });

    childProcess.exit(0);

    await expect(runPromise).resolves.toEqual({
      artifactPath: "/workspace/nested/demo.gif",
      format: "gif",
      tapeUri,
    });
  });

  it("running_the_same_file_again_cancels_the_previous_process_before_restarting", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const firstProcess = new MockChildProcess();
    const secondProcess = new MockChildProcess();
    const spawnProcess = vi
      .fn()
      .mockReturnValueOnce(firstProcess)
      .mockReturnValueOnce(secondProcess);
    const manager = new ExecutionManager({
      getWorkspaceFolders: () => [Uri.file("/workspace")],
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      spawnProcess,
    });

    const firstRun = manager.run(tapeUri);
    await flushMicrotasks();

    const secondRun = manager.run(tapeUri);
    await flushMicrotasks();

    expect(firstProcess.kill).toHaveBeenCalledWith("SIGTERM");
    expect(spawnProcess).toHaveBeenCalledTimes(1);

    const firstRunAssertion = expect(firstRun).rejects.toMatchObject({
      cancelled: true,
      name: "ExecutionFailure",
    });

    firstProcess.exit(1);
    await flushMicrotasks();

    secondProcess.exit(0);

    await firstRunAssertion;
    await expect(secondRun).resolves.toEqual({
      artifactPath: "/workspace/demo.gif",
      format: "gif",
      tapeUri,
    });
    expect(spawnProcess).toHaveBeenCalledTimes(2);
  });

  it("execution_writes_timestamped_output_logs_and_reveals_channel_on_error", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const childProcess = new MockChildProcess();
    const outputChannel = createMockOutputChannel();
    const manager = new ExecutionManager({
      getWorkspaceFolders: () => [Uri.file("/workspace")],
      now: () => Date.parse("2026-03-20T17:00:00Z"),
      outputChannel,
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      spawnProcess: vi.fn().mockReturnValue(childProcess),
    });

    const runPromise = manager.run(tapeUri);
    await flushMicrotasks();
    childProcess.stdout.write("stdout line\n");
    childProcess.stderr.write("stderr line\n");
    childProcess.exit(1);

    await expect(runPromise).rejects.toMatchObject({
      cancelled: false,
      name: "ExecutionFailure",
    });

    expect(outputChannel.appendLine).toHaveBeenCalledWith(
      "[2026-03-20T17:00:00.000Z] Running: vhs demo.tape",
    );
    expect(outputChannel.appendLine).toHaveBeenCalledWith("stdout line");
    expect(outputChannel.appendLine).toHaveBeenCalledWith("stderr line");
    expect(outputChannel.appendLine).toHaveBeenCalledWith(
      "[2026-03-20T17:00:00.000Z] Completed in 0.0s (exit code: 1)",
    );
    expect(outputChannel.show).toHaveBeenCalledTimes(1);
  });

  it("execution_keeps_the_output_channel_hidden_on_success", async () => {
    const tapeUri = Uri.file("/workspace/demo.tape");
    const childProcess = new MockChildProcess();
    const outputChannel = createMockOutputChannel();
    const manager = new ExecutionManager({
      getWorkspaceFolders: () => [Uri.file("/workspace")],
      now: () => Date.parse("2026-03-20T17:00:00Z"),
      outputChannel,
      readTapeFile: vi.fn().mockResolvedValue("Output demo.gif"),
      spawnProcess: vi.fn().mockReturnValue(childProcess),
    });

    const runPromise = manager.run(tapeUri);
    await flushMicrotasks();
    childProcess.exit(0);

    await expect(runPromise).resolves.toEqual({
      artifactPath: "/workspace/demo.gif",
      format: "gif",
      tapeUri,
    });
    expect(outputChannel.show).not.toHaveBeenCalled();
  });

  it("execution_can_reveal_the_output_channel_without_stealing_focus", () => {
    const outputChannel = createMockOutputChannel();
    const manager = new ExecutionManager({
      outputChannel,
    });

    manager.revealOutput(true);

    expect(outputChannel.show).toHaveBeenCalledWith(true);
  });
});

class MockChildProcess extends EventEmitter {
  readonly stderr = new PassThrough();
  readonly stdout = new PassThrough();
  readonly kill = vi.fn(() => true);

  exit(code: number | null): void {
    this.stderr.end();
    this.stdout.end();
    this.emit("exit", code);
  }
}

function createMockOutputChannel() {
  return {
    appendLine: vi.fn(),
    show: vi.fn(),
  };
}
