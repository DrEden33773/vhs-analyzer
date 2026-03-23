import process from "node:process";

import {
  type Disposable,
  type ExtensionContext,
  type FileSystemWatcher,
  type OutputChannel,
  Position,
  type TextDocumentChangeEvent,
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

interface SuggestTriggerContext {
  insertedText: string;
  linePrefix: string;
  lineSuffix: string;
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

export function shouldTriggerTargetedSuggest(
  context: SuggestTriggerContext,
): boolean {
  if (
    context.insertedText === " " &&
    /^\s*Set\s+Theme $/u.test(context.linePrefix)
  ) {
    return true;
  }

  if (
    isThemeQuotedValueContext(context.linePrefix, context.lineSuffix) &&
    isThemeValueEdit(context.insertedText)
  ) {
    return true;
  }

  return (
    isDurationSlotContext(context.linePrefix, context.lineSuffix) &&
    isDurationValueEdit(context.insertedText)
  );
}

function isThemeQuotedValueContext(
  linePrefix: string,
  lineSuffix: string,
): boolean {
  const match = /^\s*Set\s+Theme\s+(["'])([^"'`]*)$/u.exec(linePrefix);
  if (match === null) {
    return false;
  }

  const quote = match[1];
  return quote !== undefined && lineSuffix.startsWith(quote);
}

function isThemeValueEdit(insertedText: string): boolean {
  return (
    insertedText === '"' ||
    insertedText === "'" ||
    insertedText === '""' ||
    insertedText === "''" ||
    /^[A-Za-z0-9 +_-]$/u.test(insertedText)
  );
}

function isDurationSlotContext(
  linePrefix: string,
  lineSuffix: string,
): boolean {
  if (/^[A-Za-z0-9_]$/u.test(lineSuffix.charAt(0))) {
    return false;
  }

  return (
    /^\s*Sleep\s+(?:\d+(?:\.\d+)?|\.\d+)(?:ms|m|s)?$/u.test(linePrefix) ||
    /^\s*Type\s*@\s*(?:\d+(?:\.\d+)?|\.\d+)(?:ms|m|s)?$/u.test(linePrefix) ||
    /^\s*Set\s+TypingSpeed\s+(?:\d+(?:\.\d+)?|\.\d+)(?:ms|m|s)?$/u.test(
      linePrefix,
    )
  );
}

function isDurationValueEdit(insertedText: string): boolean {
  return /^[0-9ms]$/u.test(insertedText);
}

function advancePosition(position: Position, text: string): Position {
  let line = position.line;
  let character = position.character;

  for (let index = 0; index < text.length; index += 1) {
    const current = text[index];
    if (current === undefined) {
      break;
    }

    if (current === "\r") {
      line += 1;
      character = 0;
      if (text[index + 1] === "\n") {
        index += 1;
      }
      continue;
    }

    if (current === "\n") {
      line += 1;
      character = 0;
      continue;
    }

    character += 1;
  }

  return new Position(line, character);
}

function lineContextAtPosition(
  documentText: string,
  position: Position,
): { linePrefix: string; lineSuffix: string } | null {
  const lines = documentText.split(/\r?\n/u);
  const line = lines[position.line];
  if (line === undefined) {
    return null;
  }

  return {
    linePrefix: line.slice(0, position.character),
    lineSuffix: line.slice(position.character),
  };
}

function suggestTriggerContextForChange(
  documentText: string,
  change: TextDocumentChangeEvent["contentChanges"][number],
): SuggestTriggerContext | null {
  const candidatePositions = [advancePosition(change.range.start, change.text)];

  if (change.text === '""' || change.text === "''") {
    candidatePositions.unshift(
      advancePosition(change.range.start, change.text.slice(0, 1)),
    );
  }

  for (const position of candidatePositions) {
    const lineContext = lineContextAtPosition(documentText, position);
    if (lineContext === null) {
      continue;
    }

    const context = {
      insertedText: change.text,
      linePrefix: lineContext.linePrefix,
      lineSuffix: lineContext.lineSuffix,
    };
    if (shouldTriggerTargetedSuggest(context)) {
      return context;
    }
  }

  return null;
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
      workspace.onDidChangeTextDocument((event) => {
        void this.maybeTriggerTargetedSuggest(event);
      }),
    );

    await this.startLanguageClient(false);

    void this.dependencies.checkRuntimeDependencies({
      resolveExecutable: resolveBinaryOnPath,
    });
  }

  private async maybeTriggerTargetedSuggest(
    event: TextDocumentChangeEvent,
  ): Promise<void> {
    const activeEditor = window.activeTextEditor;
    if (
      activeEditor === undefined ||
      activeEditor.document.languageId !== "tape" ||
      activeEditor.document.uri.toString() !== event.document.uri.toString()
    ) {
      return;
    }

    const latestChange = event.contentChanges.at(-1);
    if (latestChange === undefined || latestChange.text === "") {
      return;
    }

    const suggestContext = suggestTriggerContextForChange(
      event.document.getText(),
      latestChange,
    );
    if (suggestContext === null) {
      return;
    }

    await commands.executeCommand("editor.action.triggerSuggest");
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
