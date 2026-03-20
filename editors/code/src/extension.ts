import process from "node:process";

import {
  type Disposable,
  type ExtensionContext,
  type FileSystemWatcher,
  type OutputChannel,
  Uri,
  commands,
  env,
  languages,
  window,
  workspace,
} from "vscode";
import type {
  ErrorHandler,
  InitializationFailedHandler,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";
import {
  VhsCodeLensProvider,
  bindExecutionStateToStatusBar,
  registerCodeLensCommands,
} from "./codelens";
import {
  type ExtensionConfiguration,
  type ServerTraceLevel,
  getExtensionConfiguration,
  registerConfigurationChangeHandler,
} from "./config";
import {
  type CheckRuntimeDependenciesOptions,
  checkRuntimeDependencies,
} from "./dependencies";
import { ExecutionManager } from "./execution";
import { PreviewManager } from "./preview";
import {
  RestartBudget,
  createClientErrorHandler,
  createInitializationFailedHandler,
  createServerOptions,
  discoverServerBinary,
  isExecutableFile,
  resolveBinaryOnPath,
} from "./server";
import { StatusController } from "./status";

const extensionId = "vhs-analyzer";
const extensionName = "VHS Analyzer";
const noServerMessageKey = "noServerMessageDismissed";
const noServerInstallUrl =
  "https://github.com/DrEden33773/vhs-analyzer/releases";
const statusCommandId = "vhs-analyzer.statusActions";

export interface LanguageClientLike {
  setTrace(value: ServerTraceLevel): Promise<void>;
  start(): Promise<void>;
  stop(): Promise<void>;
}

export interface ExtensionDependencies {
  checkRuntimeDependencies: (
    options: CheckRuntimeDependenciesOptions,
  ) => Promise<void>;
  createLanguageClient: (
    serverOptions: ServerOptions,
    clientOptions: LanguageClientOptions,
  ) => Promise<LanguageClientLike>;
  discoverServerBinary: (
    configuration: ExtensionConfiguration,
    extensionPath: string,
  ) => Promise<string | null>;
  isExecutableFile: (filePath: string) => Promise<boolean>;
  scheduleRestart: (callback: () => void, delayMs: number) => void;
}

export function buildLanguageClientOptions(options: {
  errorHandler: ErrorHandler;
  fileWatcher: FileSystemWatcher;
  initializationFailedHandler: InitializationFailedHandler;
  outputChannel: OutputChannel;
  traceOutputChannel: OutputChannel;
}): LanguageClientOptions {
  return {
    documentSelector: [{ scheme: "file", language: "tape" }],
    errorHandler: options.errorHandler,
    initializationFailedHandler: options.initializationFailedHandler,
    outputChannel: options.outputChannel,
    synchronize: {
      fileEvents: options.fileWatcher,
    },
    traceOutputChannel: options.traceOutputChannel,
  };
}

async function createDefaultLanguageClient(
  serverOptions: ServerOptions,
  clientOptions: LanguageClientOptions,
): Promise<LanguageClientLike> {
  const { LanguageClient } = await import("vscode-languageclient/node");
  return new LanguageClient(
    extensionId,
    extensionName,
    serverOptions,
    clientOptions,
  );
}

export function createDefaultDependencies(): ExtensionDependencies {
  return {
    checkRuntimeDependencies,
    createLanguageClient: createDefaultLanguageClient,
    async discoverServerBinary(configuration, extensionPath) {
      return discoverServerBinary({
        configuredPath: configuration.serverPath,
        extensionPath,
        isExecutable: isExecutableFile,
        platform: process.platform,
        resolveOnPath: resolveBinaryOnPath,
      });
    },
    isExecutableFile,
    scheduleRestart(callback, delayMs) {
      setTimeout(callback, delayMs);
    },
  };
}

export class ExtensionController {
  private client: LanguageClientLike | undefined;
  private readonly dependencies: ExtensionDependencies;
  private readonly executionManager: ExecutionManager;
  private readonly fileWatcher: FileSystemWatcher;
  private readonly outputChannel: OutputChannel;
  private readonly previewManager: PreviewManager;
  private readonly restartBudget = new RestartBudget();
  private readonly runOutputChannel: OutputChannel;
  private readonly status: StatusController;
  private readonly traceOutputChannel: OutputChannel;
  private readonly vhsCodeLensProvider: VhsCodeLensProvider;

  constructor(
    private readonly context: ExtensionContext,
    dependencies: Partial<ExtensionDependencies> = {},
  ) {
    this.dependencies = {
      ...createDefaultDependencies(),
      ...dependencies,
    };
    this.outputChannel = window.createOutputChannel(extensionName);
    this.runOutputChannel = window.createOutputChannel(`${extensionName}: Run`);
    this.traceOutputChannel = window.createOutputChannel(
      `${extensionName} Trace`,
    );
    this.fileWatcher = workspace.createFileSystemWatcher("**/*.tape");
    this.status = new StatusController(
      {
        onRestartServer: async () => {
          await this.restartServer(false, true);
        },
        onShowOutput: () => {
          this.outputChannel.show();
        },
        onShowTrace: () => {
          this.traceOutputChannel.show();
        },
      },
      statusCommandId,
    );
    this.executionManager = new ExecutionManager({
      outputChannel: this.runOutputChannel,
    });
    this.previewManager = new PreviewManager({
      executionManager: this.executionManager,
      extensionPath: context.extensionPath,
      workspaceFolders: (workspace.workspaceFolders ?? []).map(
        (folder) => folder.uri,
      ),
    });
    this.vhsCodeLensProvider = new VhsCodeLensProvider({
      executionManager: this.executionManager,
    });
  }

  async activate(): Promise<void> {
    this.context.subscriptions.push(
      this.executionManager,
      this.outputChannel,
      this.previewManager,
      this.runOutputChannel,
      this.traceOutputChannel,
      this.fileWatcher,
      this.status,
      this.vhsCodeLensProvider,
      bindExecutionStateToStatusBar({
        executionManager: this.executionManager,
        status: this.status,
      }),
      languages.registerCodeLensProvider(
        { language: "tape" },
        this.vhsCodeLensProvider,
      ),
      ...registerCodeLensCommands({
        executionManager: this.executionManager,
        previewManager: this.previewManager,
      }),
      commands.registerCommand(statusCommandId, async () => {
        await this.status.showActions();
      }),
      registerConfigurationChangeHandler({
        onImmediateConfigurationChange: () => {},
        onRestartRequired: async () => {
          await this.restartServer(true, true);
        },
        onTraceLevelChange: async (traceLevel) => {
          if (this.client !== undefined) {
            await this.client.setTrace(traceLevel);
          }
        },
      }),
    );

    await this.startLanguageClient(false);

    void this.dependencies.checkRuntimeDependencies({
      resolveExecutable: resolveBinaryOnPath,
    });
  }

  async deactivate(): Promise<void> {
    await this.stopClient();
  }

  private async restartServer(
    strictConfiguredPath: boolean,
    resetRetryBudget = false,
  ): Promise<void> {
    if (resetRetryBudget) {
      this.restartBudget.reset();
    }

    await this.stopClient();
    await this.startLanguageClient(strictConfiguredPath);
  }

  private async startLanguageClient(
    strictConfiguredPath: boolean,
  ): Promise<void> {
    const configuration = getExtensionConfiguration();
    const configuredPath = configuration.serverPath.trim();

    if (
      strictConfiguredPath &&
      configuredPath !== "" &&
      !(await this.dependencies.isExecutableFile(configuredPath))
    ) {
      await this.showConfigurationError(configuredPath);
      return;
    }

    const binaryPath = await this.dependencies.discoverServerBinary(
      configuration,
      this.context.extensionPath,
    );

    if (binaryPath === null) {
      await this.enterNoServerMode();
      return;
    }

    const errorHandler = createClientErrorHandler({
      budget: this.restartBudget,
      onRestartScheduled: () => {
        this.status.setServerStatus("starting");
      },
      onRetryExhausted: async () => {
        this.status.setServerStatus("failed");
        const selection = await window.showErrorMessage(
          "VHS Analyzer server crashed too many times.",
          "Restart Server",
        );

        if (selection === "Restart Server") {
          await this.restartServer(false, true);
        }
      },
      scheduleRestart: (delayMs) => {
        this.dependencies.scheduleRestart(() => {
          void this.restartServer(false);
        }, delayMs);
      },
    });

    const initializationFailedHandler = createInitializationFailedHandler(
      (error) => {
        void window.showErrorMessage(
          `Failed to initialize the VHS Analyzer server: ${String(error)}`,
        );
      },
    );

    const clientOptions = buildLanguageClientOptions({
      errorHandler,
      fileWatcher: this.fileWatcher,
      initializationFailedHandler,
      outputChannel: this.outputChannel,
      traceOutputChannel: this.traceOutputChannel,
    });

    this.status.setServerStatus("starting");

    const client = await this.dependencies.createLanguageClient(
      createServerOptions(binaryPath, configuration.serverArgs),
      clientOptions,
    );

    this.client = client;

    try {
      await client.start();
      await client.setTrace(configuration.traceServer);
      this.status.setServerStatus("running");
    } catch (error) {
      this.client = undefined;
      this.status.setServerStatus("failed");
      await window.showErrorMessage(
        `Failed to start the VHS Analyzer server: ${String(error)}`,
      );
    }
  }

  private async enterNoServerMode(): Promise<void> {
    this.status.setServerStatus("failed");

    if (
      this.context.globalState.get<boolean>(noServerMessageKey, false) === true
    ) {
      return;
    }

    const selection = await window.showInformationMessage(
      "VHS Analyzer LSP server not found. Install the platform-specific extension for full language support.",
      "Install",
      "Don't show again",
    );

    if (selection === "Install") {
      await env.openExternal(Uri.parse(noServerInstallUrl));
      return;
    }

    if (selection === "Don't show again") {
      await this.context.globalState.update(noServerMessageKey, true);
    }
  }

  private async showConfigurationError(configuredPath: string): Promise<void> {
    this.status.setServerStatus("failed");

    const selection = await window.showErrorMessage(
      `Configured VHS Analyzer server not found at ${configuredPath}.`,
      "Restart Server",
    );

    if (selection === "Restart Server") {
      await this.restartServer(true, true);
    }
  }

  private async stopClient(): Promise<void> {
    const client = this.client;
    this.client = undefined;

    if (client !== undefined) {
      await client.stop();
    }
  }
}

let extensionController: ExtensionController | undefined;

export async function activate(context: ExtensionContext): Promise<void> {
  extensionController = new ExtensionController(context);
  await extensionController.activate();
}

export async function deactivate(): Promise<void> {
  await extensionController?.deactivate();
  extensionController = undefined;
}
