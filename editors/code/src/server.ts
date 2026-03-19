import { constants, access } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

import type {
  CloseAction,
  ErrorAction,
  ErrorHandler,
  InitializationFailedHandler,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";
import which from "which";

export interface BinaryDiscoveryOptions {
  configuredPath: string;
  extensionPath: string;
  isExecutable: (filePath: string) => Promise<boolean>;
  platform?: NodeJS.Platform;
  resolveOnPath: (binaryName: string) => Promise<string | null>;
}

export const ClientCloseAction = {
  DoNotRestart: 1,
  Restart: 2,
} as const;

export const ClientErrorAction = {
  Continue: 1,
  Shutdown: 2,
} as const;

export const ClientTransportKind = {
  stdio: 0,
} as const;

export class RestartBudget {
  private readonly attempts: number[] = [];

  constructor(
    private readonly now: () => number = Date.now,
    private readonly delaysMs: readonly number[] = [
      1000, 2000, 4000, 8000, 8000,
    ],
    private readonly windowMs = 3 * 60 * 1000,
  ) {}

  nextDelay(): number | null {
    const currentTime = this.now();
    const cutoff = currentTime - this.windowMs;

    while (this.attempts[0] !== undefined && this.attempts[0] < cutoff) {
      this.attempts.shift();
    }

    if (this.attempts.length >= this.delaysMs.length) {
      return null;
    }

    this.attempts.push(currentTime);
    return this.delaysMs[this.attempts.length - 1] ?? null;
  }

  reset(): void {
    this.attempts.length = 0;
  }
}

export interface ClientErrorHandlerOptions {
  budget?: RestartBudget;
  onRestartScheduled?: () => void;
  onRetryExhausted: () => void | Promise<void>;
  scheduleRestart: (delayMs: number) => void;
}

export async function discoverServerBinary(
  options: BinaryDiscoveryOptions,
): Promise<string | null> {
  const configuredPath = options.configuredPath.trim();
  if (configuredPath && (await options.isExecutable(configuredPath))) {
    return configuredPath;
  }

  const extension = options.platform === "win32" ? ".exe" : "";
  const bundledPath = path.join(
    options.extensionPath,
    "server",
    `vhs-analyzer${extension}`,
  );

  if (await options.isExecutable(bundledPath)) {
    return bundledPath;
  }

  return options.resolveOnPath("vhs-analyzer");
}

export function createServerOptions(
  command: string,
  args: readonly string[],
): ServerOptions {
  return {
    command,
    args: [...args],
    options: {
      env: process.env,
    },
    transport: ClientTransportKind.stdio as TransportKind,
  };
}

export function createClientErrorHandler(
  options: ClientErrorHandlerOptions,
): ErrorHandler {
  const budget = options.budget ?? new RestartBudget();

  return {
    error(): { action: ErrorAction; handled: boolean } {
      return {
        action: ClientErrorAction.Shutdown as ErrorAction,
        handled: true,
      };
    },
    async closed(): Promise<{ action: CloseAction; handled: boolean }> {
      const delayMs = budget.nextDelay();

      if (delayMs === null) {
        await options.onRetryExhausted();
        return {
          action: ClientCloseAction.DoNotRestart as CloseAction,
          handled: true,
        };
      }

      options.onRestartScheduled?.();
      options.scheduleRestart(delayMs);
      return {
        action: ClientCloseAction.DoNotRestart as CloseAction,
        handled: true,
      };
    },
  };
}

export function createInitializationFailedHandler(
  reportInitializationFailure: (error: unknown) => void,
): InitializationFailedHandler {
  return (error) => {
    reportInitializationFailure(error);
    return false;
  };
}

export async function isExecutableFile(
  filePath: string,
  platform: NodeJS.Platform = process.platform,
): Promise<boolean> {
  const mode = platform === "win32" ? constants.F_OK : constants.X_OK;

  try {
    await access(filePath, mode);
    return true;
  } catch {
    return false;
  }
}

export async function resolveBinaryOnPath(
  binaryName: string,
): Promise<string | null> {
  return which(binaryName, { nothrow: true });
}
