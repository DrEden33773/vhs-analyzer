declare module "*?raw" {
  const content: string;
  export default content;
}

declare module "which" {
  export interface WhichOptions {
    all?: boolean;
    nothrow?: boolean;
    path?: string;
    pathExt?: string;
  }

  export default function which(
    binaryName: string,
    options?: WhichOptions,
  ): Promise<string | null>;
}

declare module "vscode-languageclient/node" {
  import type { FileSystemWatcher, OutputChannel } from "vscode";

  export interface ErrorHandlerResult {
    action: number;
    handled?: boolean;
    message?: string;
  }

  export interface ErrorHandler {
    error(
      error: Error,
      message: unknown,
      count: number | undefined,
    ): ErrorHandlerResult | Promise<ErrorHandlerResult>;
    closed(): ErrorHandlerResult | Promise<ErrorHandlerResult>;
  }

  export type InitializationFailedHandler = (error: unknown) => boolean;
  export type CloseAction = number;
  export type ErrorAction = number;
  export type TransportKind = number;

  export interface ExecutableOptions {
    cwd?: string;
    detached?: boolean;
    env?: NodeJS.ProcessEnv;
    shell?: boolean;
  }

  export interface ServerOptions {
    args?: string[];
    command: string;
    options?: ExecutableOptions;
    transport?: TransportKind;
  }

  export interface LanguageClientOptions {
    documentSelector?: Array<{ language: string; scheme: string }> | string[];
    errorHandler?: ErrorHandler;
    initializationFailedHandler?: InitializationFailedHandler;
    outputChannel?: OutputChannel;
    synchronize?: {
      fileEvents?: FileSystemWatcher | FileSystemWatcher[];
    };
    traceOutputChannel?: OutputChannel;
  }

  export class LanguageClient {
    constructor(
      id: string,
      name: string,
      serverOptions: ServerOptions,
      clientOptions: LanguageClientOptions,
    );

    setTrace(value: "messages" | "off" | "verbose"): Promise<void>;
    start(): Promise<void>;
    stop(timeout?: number): Promise<void>;
  }
}
