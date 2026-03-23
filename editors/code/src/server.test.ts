import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("vscode", async () => import("./__mocks__/vscode.js"));

import {
  ClientCloseAction,
  ClientErrorAction,
  ClientTransportKind,
  RestartBudget,
  createClientErrorHandler,
  createInitializationFailedHandler,
  createServerOptions,
  discoverServerBinary,
} from "./server";

describe("discoverServerBinary", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("binary_discovery_returns_null_when_no_binary", async () => {
    const result = await discoverServerBinary({
      configuredPath: "",
      extensionPath: "/extension",
      isExecutable: vi.fn().mockResolvedValue(false),
      platform: "linux",
      resolveOnPath: vi.fn().mockResolvedValue(null),
    });

    expect(result).toBeNull();
  });

  it("binary_discovery_returns_user_configured_path", async () => {
    const isExecutable = vi
      .fn<BinaryDiscoveryPredicate>()
      .mockResolvedValueOnce(true);
    const resolveOnPath = vi.fn().mockResolvedValue("/usr/bin/vhs-analyzer");

    const result = await discoverServerBinary({
      configuredPath: "/custom/vhs-analyzer",
      extensionPath: "/extension",
      isExecutable,
      platform: "linux",
      resolveOnPath,
    });

    expect(result).toBe("/custom/vhs-analyzer");
    expect(resolveOnPath).not.toHaveBeenCalled();
  });

  it("binary_discovery_falls_back_to_bundled_binary", async () => {
    const isExecutable = vi
      .fn<BinaryDiscoveryPredicate>()
      .mockResolvedValueOnce(false)
      .mockResolvedValueOnce(true);

    const result = await discoverServerBinary({
      configuredPath: "/custom/vhs-analyzer",
      extensionPath: "/extension",
      isExecutable,
      platform: "linux",
      resolveOnPath: vi.fn().mockResolvedValue(null),
    });

    expect(result).toBe("/extension/server/vhs-analyzer");
  });

  it("binary_discovery_falls_back_to_path_binary", async () => {
    const resolveOnPath = vi.fn().mockResolvedValue("/usr/bin/vhs-analyzer");

    const result = await discoverServerBinary({
      configuredPath: "",
      extensionPath: "/extension",
      isExecutable: vi.fn().mockResolvedValue(false),
      platform: "linux",
      resolveOnPath,
    });

    expect(result).toBe("/usr/bin/vhs-analyzer");
    expect(resolveOnPath).toHaveBeenCalledWith("vhs-analyzer");
  });

  it("server_options_use_stdio_transport_and_disable_ansi_by_default", () => {
    const options = createServerOptions("/bin/vhs-analyzer", ["--log=debug"]);

    expect(options).toEqual({
      command: "/bin/vhs-analyzer",
      args: ["--log=debug"],
      options: {
        env: {
          ...process.env,
          NO_COLOR: "1",
        },
      },
      transport: ClientTransportKind.stdio,
    });
  });

  it("client_error_handler_schedules_restart_with_exponential_backoff", async () => {
    let now = 0;
    const scheduled: number[] = [];

    const handler = createClientErrorHandler({
      budget: new RestartBudget(() => now),
      onRetryExhausted: vi.fn(),
      onRestartScheduled: vi.fn(),
      scheduleRestart: (delayMs) => {
        scheduled.push(delayMs);
      },
    });

    const firstClose = await handler.closed();
    now += 1_000;
    const secondClose = await handler.closed();

    expect(firstClose).toEqual({
      action: ClientCloseAction.DoNotRestart,
      handled: true,
    });
    expect(secondClose).toEqual({
      action: ClientCloseAction.DoNotRestart,
      handled: true,
    });
    expect(handler.error(new Error("boom"), undefined, 1)).toEqual({
      action: ClientErrorAction.Shutdown,
      handled: true,
    });
    expect(scheduled).toEqual([1000, 2000]);
  });

  it("client_error_handler_stops_after_retry_exhaustion", async () => {
    let now = 0;
    const exhausted = vi.fn();
    const scheduled: number[] = [];

    const handler = createClientErrorHandler({
      budget: new RestartBudget(() => now),
      onRetryExhausted: exhausted,
      scheduleRestart: (delayMs) => {
        scheduled.push(delayMs);
      },
    });

    for (let attempt = 0; attempt < 5; attempt += 1) {
      await handler.closed();
      now += 1_000;
    }

    const result = await handler.closed();

    expect(result).toEqual({
      action: ClientCloseAction.DoNotRestart,
      handled: true,
    });
    expect(exhausted).toHaveBeenCalledTimes(1);
    expect(scheduled).toEqual([1000, 2000, 4000, 8000, 8000]);
  });

  it("initialization_failure_handler_reports_error_and_stops_retrying", () => {
    const reportInitializationFailure = vi.fn();
    const handler = createInitializationFailedHandler(
      reportInitializationFailure,
    );

    expect(handler(new Error("init failed"))).toBe(false);
    expect(reportInitializationFailure).toHaveBeenCalledTimes(1);
  });
});

type BinaryDiscoveryPredicate = (filePath: string) => Promise<boolean>;
