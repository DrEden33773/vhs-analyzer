import { type ChildProcessWithoutNullStreams, spawn } from "node:child_process";
import { constants } from "node:fs";
import {
  access,
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";

import { afterEach, describe, expect, it, vi } from "vitest";
import type { ExtensionContext } from "vscode";
import { Uri } from "vscode";

vi.mock("vscode", async () => import("./__mocks__/vscode.js"));

import {
  __getLastCreatedWebviewPanel,
  __getRegisteredCodeLensProviders,
  __resetMockVscode,
  __setActiveTextEditor,
  __setWorkspaceFolders,
  commands,
  createMockExtensionContext,
  window,
} from "./__mocks__/vscode";
import { previewTapeCommandId } from "./codelens";
import { ExtensionController, type LanguageClientLike } from "./extension";

interface HoverMarkupContent {
  kind: string;
  value: string;
}

interface HoverResult {
  contents:
    | HoverMarkupContent
    | HoverMarkupContent[]
    | string
    | Array<{ language: string; value: string } | string>;
}

interface InitializeResult {
  capabilities: {
    completionProvider?: unknown;
    documentFormattingProvider?: boolean | unknown;
    hoverProvider?: boolean | unknown;
  };
}

interface IntegrationFixture {
  dispose(): Promise<void>;
  extensionPath: string;
  tapeSource: string;
  tapeUri: Uri;
  workspacePath: string;
  workspaceUri: Uri;
}

type JsonRpcRequestId = number;

type PendingRequest = {
  reject: (reason: unknown) => void;
  resolve: (value: unknown) => void;
  timer: NodeJS.Timeout;
};

describe("Phase 3 integration", () => {
  afterEach(() => {
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("full_activation_starts_bundled_server_and_returns_hover_for_type", async () => {
    let controller: ExtensionController | undefined;
    let fixture: IntegrationFixture | undefined;
    let client: TestLspHarnessClient | undefined;

    try {
      fixture = await createIntegrationFixture();
      client = new TestLspHarnessClient(fixture.workspaceUri);
      controller = new ExtensionController(
        createTypedContext({
          extensionPath: fixture.extensionPath,
        }),
        {
          checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
          createLanguageClient: vi.fn(async (serverOptions) => {
            client?.configure(serverOptions);
            return client as LanguageClientLike;
          }),
        },
      );

      await controller.activate();

      expect(window.showErrorMessage).not.toHaveBeenCalled();
      expect(window.showInformationMessage).not.toHaveBeenCalledWith(
        "VHS Analyzer LSP server not found. Install the platform-specific extension for full language support.",
        "Install",
        "Don't show again",
      );
      expect(window.createStatusBarItem.mock.results[0]?.value.text).toBe(
        "VHS $(check)",
      );
      expect(client.initializeResult?.capabilities.hoverProvider).toBe(true);
      expect(
        client.initializeResult?.capabilities.completionProvider,
      ).toBeDefined();
      expect(
        client.initializeResult?.capabilities.documentFormattingProvider,
      ).toBe(true);

      await client.openDocument(fixture.tapeUri, fixture.tapeSource);
      const hover = await client.hover(fixture.tapeUri, {
        character: 0,
        line: 0,
      });
      const hoverMarkdown = extractHoverMarkdown(hover);

      expect(hoverMarkdown).toContain("Emulate typing text into the terminal.");
      expect(hoverMarkdown).toContain('Type[@<speed>] "<text>"');
    } finally {
      await controller?.deactivate();
      await client?.stop();
      await fixture?.dispose();
    }
  }, 30_000);

  it("codelens_run_and_preview_posts_render_complete_for_output_artifact", async () => {
    let controller: ExtensionController | undefined;
    let fakeCommands: FakeCommandEnvironment | undefined;
    let fixture: IntegrationFixture | undefined;
    let client: TestLspHarnessClient | undefined;
    const originalPath = process.env.PATH;

    try {
      fixture = await createIntegrationFixture({
        tapeSource: 'Output demo.gif\nType "hello"\n',
      });
      fakeCommands = await createFakeCommandEnvironment();
      process.env.PATH = joinPathEntries(fakeCommands.binPath, originalPath);
      client = new TestLspHarnessClient(fixture.workspaceUri);
      controller = new ExtensionController(
        createTypedContext({
          extensionPath: fixture.extensionPath,
        }),
        {
          checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
          createLanguageClient: vi.fn(async (serverOptions) => {
            client?.configure(serverOptions);
            return client as LanguageClientLike;
          }),
        },
      );

      await controller.activate();

      const runAndPreviewLens = getFileLevelRunAndPreviewLens({
        source: fixture.tapeSource,
        tapeUri: fixture.tapeUri,
      });
      await commands.executeCommand(
        runAndPreviewLens.command.command,
        ...(runAndPreviewLens.command.arguments ?? []),
      );

      const panel = __getLastCreatedWebviewPanel();
      expect(panel?.title).toBe("VHS Preview: demo.tape");
      expect(runAndPreviewLens.command.command).toBe(previewTapeCommandId);
      expect(panel?.webview.postedMessages).toEqual(
        expect.arrayContaining([
          {
            tapeFile: "demo.tape",
            type: "renderStart",
          },
          {
            line: "Rendering frame 1",
            type: "renderProgress",
          },
          {
            artifactUri: path.join(fixture.workspacePath, "demo.gif"),
            format: "gif",
            type: "renderComplete",
          },
        ]),
      );
    } finally {
      process.env.PATH = originalPath;
      await controller?.deactivate();
      await client?.stop();
      await fakeCommands?.dispose();
      await fixture?.dispose();
    }
  }, 30_000);

  it("no_server_mode_keeps_codelens_and_preview_available_without_language_client", async () => {
    let controller: ExtensionController | undefined;
    let fakeCommands: FakeCommandEnvironment | undefined;
    let fixture: IntegrationFixture | undefined;
    const createLanguageClient = vi.fn();
    const originalPath = process.env.PATH;

    try {
      fixture = await createIntegrationFixture({
        includeBundledServer: false,
        tapeSource: 'Output demo.gif\nType "hello"\n',
      });
      fakeCommands = await createFakeCommandEnvironment();
      process.env.PATH = fakeCommands.binPath;
      window.showInformationMessage.mockResolvedValueOnce(undefined);
      controller = new ExtensionController(
        createTypedContext({
          extensionPath: fixture.extensionPath,
        }),
        {
          checkRuntimeDependencies: vi.fn().mockResolvedValue(undefined),
          createLanguageClient,
        },
      );

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

      const runAndPreviewLens = getFileLevelRunAndPreviewLens({
        source: fixture.tapeSource,
        tapeUri: fixture.tapeUri,
      });
      await commands.executeCommand(
        runAndPreviewLens.command.command,
        ...(runAndPreviewLens.command.arguments ?? []),
      );

      const panel = __getLastCreatedWebviewPanel();
      expect(panel?.title).toBe("VHS Preview: demo.tape");
      expect(panel?.webview.postedMessages).toEqual(
        expect.arrayContaining([
          {
            tapeFile: "demo.tape",
            type: "renderStart",
          },
          {
            artifactUri: path.join(fixture.workspacePath, "demo.gif"),
            format: "gif",
            type: "renderComplete",
          },
        ]),
      );
    } finally {
      process.env.PATH = originalPath;
      await controller?.deactivate();
      await fakeCommands?.dispose();
      await fixture?.dispose();
    }
  }, 30_000);
});

class TestLspHarnessClient implements LanguageClientLike {
  initializeResult: InitializeResult | undefined;

  private nextRequestId: JsonRpcRequestId = 1;
  private readonly pendingRequests = new Map<
    JsonRpcRequestId,
    PendingRequest
  >();
  private process: ChildProcessWithoutNullStreams | undefined;
  private serverOptions:
    | {
        args: string[];
        command: string;
        env: NodeJS.ProcessEnv | undefined;
      }
    | undefined;
  private stderrOutput = "";
  private stdoutBuffer = Buffer.alloc(0);

  constructor(private readonly workspaceUri: Uri) {}

  configure(serverOptions: {
    args?: string[];
    command: string;
    options?: {
      env?: NodeJS.ProcessEnv;
    };
  }): void {
    this.serverOptions = {
      args: [...(serverOptions.args ?? [])],
      command: serverOptions.command,
      env: serverOptions.options?.env,
    };
  }

  async setTrace(value: "off" | "messages" | "verbose"): Promise<void> {
    if (this.process === undefined) {
      return;
    }

    await this.sendNotification("$/setTrace", { value });
  }

  async start(): Promise<void> {
    if (this.process !== undefined) {
      return;
    }

    if (this.serverOptions === undefined) {
      throw new Error("The integration LSP harness was not configured.");
    }

    const child = spawn(this.serverOptions.command, this.serverOptions.args, {
      env: this.serverOptions.env,
      stdio: "pipe",
    });

    this.process = child;
    child.stdout.on("data", (chunk: Buffer) => {
      this.consumeStdoutChunk(chunk);
    });
    child.stderr.on("data", (chunk: Buffer) => {
      this.stderrOutput += chunk.toString("utf8");
    });
    child.on("error", (error) => {
      this.rejectPendingRequests(
        new Error(
          `Failed to spawn the VHS Analyzer LSP server: ${String(error)}${this.formatStderr()}`,
        ),
      );
    });
    child.on("exit", (code, signal) => {
      this.process = undefined;
      this.rejectPendingRequests(
        new Error(
          `The VHS Analyzer LSP server exited before completing the request (code: ${String(code)}, signal: ${String(signal)}).${this.formatStderr()}`,
        ),
      );
    });

    this.initializeResult = await this.sendRequest<InitializeResult>(
      "initialize",
      {
        capabilities: {
          textDocument: {
            hover: {
              contentFormat: ["markdown"],
            },
          },
        },
        clientInfo: {
          name: "vitest",
        },
        processId: process.pid,
        rootUri: this.workspaceUri.toString(),
        trace: "off",
        workspaceFolders: [
          {
            name: path.basename(this.workspaceUri.fsPath),
            uri: this.workspaceUri.toString(),
          },
        ],
      },
    );
    await this.sendNotification("initialized", {});
  }

  async stop(): Promise<void> {
    const child = this.process;

    if (child === undefined) {
      return;
    }

    try {
      await this.sendRequest("shutdown", null, 2_000);
    } catch {}

    try {
      await this.sendNotification("exit");
    } catch {}

    await waitForProcessExit(child);
    this.process = undefined;
    this.rejectPendingRequests(
      new Error("The VHS Analyzer LSP server stopped."),
    );
  }

  async openDocument(uri: Uri, source: string): Promise<void> {
    await this.sendNotification("textDocument/didOpen", {
      textDocument: {
        languageId: "tape",
        text: source,
        uri: uri.toString(),
        version: 1,
      },
    });
  }

  async hover(
    uri: Uri,
    position: { character: number; line: number },
  ): Promise<HoverResult | null> {
    return this.sendRequest<HoverResult | null>("textDocument/hover", {
      position,
      textDocument: {
        uri: uri.toString(),
      },
    });
  }

  private consumeStdoutChunk(chunk: Buffer): void {
    this.stdoutBuffer = Buffer.concat([this.stdoutBuffer, chunk]);

    while (true) {
      const headerEndIndex = this.stdoutBuffer.indexOf("\r\n\r\n");
      if (headerEndIndex === -1) {
        return;
      }

      const header = this.stdoutBuffer
        .subarray(0, headerEndIndex)
        .toString("utf8");
      const contentLengthMatch = header.match(/Content-Length:\s*(\d+)/i);
      if (contentLengthMatch?.[1] === undefined) {
        throw new Error(
          `Missing Content-Length header in LSP response: ${header}`,
        );
      }

      const bodyLength = Number(contentLengthMatch[1]);
      const messageEndIndex = headerEndIndex + 4 + bodyLength;
      if (this.stdoutBuffer.length < messageEndIndex) {
        return;
      }

      const body = this.stdoutBuffer
        .subarray(headerEndIndex + 4, messageEndIndex)
        .toString("utf8");
      this.stdoutBuffer = this.stdoutBuffer.subarray(messageEndIndex);
      this.handleMessage(JSON.parse(body) as JsonRpcMessage);
    }
  }

  private handleMessage(message: JsonRpcMessage): void {
    if (typeof message.id !== "number") {
      return;
    }

    const pendingRequest = this.pendingRequests.get(message.id);
    if (pendingRequest === undefined) {
      return;
    }

    clearTimeout(pendingRequest.timer);
    this.pendingRequests.delete(message.id);

    if ("error" in message && message.error !== undefined) {
      pendingRequest.reject(
        new Error(
          `LSP request ${message.id} failed with ${message.error.code}: ${message.error.message}${this.formatStderr()}`,
        ),
      );
      return;
    }

    pendingRequest.resolve(message.result);
  }

  private rejectPendingRequests(reason: Error): void {
    for (const [requestId, pendingRequest] of this.pendingRequests.entries()) {
      clearTimeout(pendingRequest.timer);
      pendingRequest.reject(reason);
      this.pendingRequests.delete(requestId);
    }
  }

  private async sendNotification(
    method: string,
    params?: unknown,
  ): Promise<void> {
    this.writeMessage({
      jsonrpc: "2.0",
      method,
      params,
    });
  }

  private async sendRequest<T>(
    method: string,
    params?: unknown,
    timeoutMs = 5_000,
  ): Promise<T> {
    const requestId = this.nextRequestId++;

    return new Promise<T>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pendingRequests.delete(requestId);
        reject(
          new Error(
            `Timed out waiting for the ${method} response.${this.formatStderr()}`,
          ),
        );
      }, timeoutMs);

      this.pendingRequests.set(requestId, {
        reject,
        resolve: (value) => {
          resolve(value as T);
        },
        timer,
      });

      try {
        this.writeMessage({
          id: requestId,
          jsonrpc: "2.0",
          method,
          params,
        });
      } catch (error) {
        clearTimeout(timer);
        this.pendingRequests.delete(requestId);
        reject(error);
      }
    });
  }

  private writeMessage(message: Record<string, unknown>): void {
    const child = this.process;
    if (child === undefined || child.stdin.destroyed) {
      throw new Error("The VHS Analyzer LSP server process is not running.");
    }

    const payload = JSON.stringify(message);
    const frame = `Content-Length: ${Buffer.byteLength(payload, "utf8")}\r\n\r\n${payload}`;
    child.stdin.write(frame, "utf8");
  }

  private formatStderr(): string {
    if (this.stderrOutput.trim() === "") {
      return "";
    }

    return `\nServer stderr:\n${this.stderrOutput.trimEnd()}`;
  }
}

type JsonRpcMessage =
  | {
      error?: {
        code: number;
        message: string;
      };
      id: JsonRpcRequestId;
      jsonrpc: "2.0";
      result?: unknown;
    }
  | {
      id?: never;
      jsonrpc: "2.0";
      method: string;
      params?: unknown;
    };

async function createIntegrationFixture(
  options: Partial<{
    includeBundledServer: boolean;
    tapeSource: string;
  }> = {},
): Promise<IntegrationFixture> {
  const rootPath = await mkdtemp(
    path.join(os.tmpdir(), "vhs-analyzer-phase3-integration-"),
  );
  const workspacePath = path.join(rootPath, "workspace");
  const extensionPath = path.join(rootPath, "extension");
  const bundledBinaryPath = path.join(
    extensionPath,
    "server",
    bundledBinaryName(),
  );
  const tapePath = path.join(workspacePath, "demo.tape");
  const tapeSource = options.tapeSource ?? 'Type "hello"\n';

  await mkdir(workspacePath, {
    recursive: true,
  });

  if (options.includeBundledServer !== false) {
    await mkdir(path.dirname(bundledBinaryPath), {
      recursive: true,
    });
    await prepareBundledBinary(bundledBinaryPath);
  }

  await writeFile(tapePath, tapeSource, "utf8");

  __setWorkspaceFolders([workspacePath]);
  const tapeUri = Uri.file(tapePath);
  __setActiveTextEditor({
    languageId: "tape",
    uri: tapeUri,
  });

  return {
    async dispose() {
      await rm(rootPath, {
        force: true,
        recursive: true,
      });
    },
    extensionPath,
    tapeSource,
    tapeUri,
    workspacePath,
    workspaceUri: Uri.file(workspacePath),
  };
}

interface FakeCommandEnvironment {
  binPath: string;
  dispose(): Promise<void>;
}

async function createFakeCommandEnvironment(): Promise<FakeCommandEnvironment> {
  const rootPath = await mkdtemp(
    path.join(os.tmpdir(), "vhs-analyzer-fake-commands-"),
  );
  const binPath = path.join(rootPath, "bin");
  const vhsScriptPath = path.join(binPath, "vhs.js");

  await mkdir(binPath, {
    recursive: true,
  });
  await writeFile(
    vhsScriptPath,
    [
      'const fs = require("node:fs");',
      'const path = require("node:path");',
      "const tapePath = process.argv[2];",
      'const source = fs.readFileSync(tapePath, "utf8");',
      'const outputMatch = source.match(/^Output\\s+(?:"([^"]+)"|(\\S+))/mu);',
      'const outputPath = path.resolve(path.dirname(tapePath), outputMatch?.[1] ?? outputMatch?.[2] ?? "out.gif");',
      "fs.mkdirSync(path.dirname(outputPath), { recursive: true });",
      'fs.writeFileSync(outputPath, "GIF89a");',
      'console.error("Rendering frame 1");',
    ].join("\n"),
    "utf8",
  );
  await writeExecutableCommand({
    binPath,
    commandName: "vhs",
    scriptPath: vhsScriptPath,
  });

  return {
    binPath,
    async dispose() {
      await rm(rootPath, {
        force: true,
        recursive: true,
      });
    },
  };
}

async function prepareBundledBinary(destinationPath: string): Promise<void> {
  const releaseBinaryPath =
    process.env.VHS_ANALYZER_LSP_BINARY ??
    path.resolve(
      process.cwd(),
      "..",
      "..",
      "target",
      "release",
      releaseBinaryName(),
    );
  const accessMode =
    process.platform === "win32" ? constants.F_OK : constants.X_OK;

  try {
    await access(releaseBinaryPath, accessMode);
  } catch (error) {
    throw new Error(
      `Expected a built VHS Analyzer LSP binary at ${releaseBinaryPath}. Run \`cargo build --release -p vhs-analyzer-lsp --locked\` or set VHS_ANALYZER_LSP_BINARY.`,
      { cause: error },
    );
  }

  await copyFile(releaseBinaryPath, destinationPath);

  if (process.platform !== "win32") {
    await chmod(destinationPath, 0o755);
  }
}

function bundledBinaryName(): string {
  return process.platform === "win32" ? "vhs-analyzer.exe" : "vhs-analyzer";
}

function releaseBinaryName(): string {
  return process.platform === "win32" ? "vhs-analyzer.exe" : "vhs-analyzer";
}

function createTypedContext(overrides: {
  extensionPath?: string;
  globalState?: Record<string, unknown>;
}): ExtensionContext {
  return createMockExtensionContext(overrides) as unknown as ExtensionContext;
}

function createMockTapeDocument(options: { source: string; tapeUri: Uri }) {
  return {
    getText: () => options.source,
    uri: options.tapeUri,
  };
}

function getFileLevelRunAndPreviewLens(options: {
  source: string;
  tapeUri: Uri;
}): {
  command: {
    arguments?: unknown[];
    command: string;
    title: string;
  };
} {
  const registration = __getRegisteredCodeLensProviders()[0];
  if (registration === undefined) {
    throw new Error("Expected a registered VHS CodeLens provider.");
  }

  const provider = registration.provider as {
    provideCodeLenses(document: {
      getText(): string;
      uri: Uri;
    }): Array<{
      command?: {
        arguments?: unknown[];
        command: string;
        title: string;
      };
    }>;
  };
  const lenses = provider.provideCodeLenses(createMockTapeDocument(options));
  const runAndPreviewLens = lenses.find(
    (lens) => lens.command?.title === "▶ Run & Preview",
  );
  if (runAndPreviewLens?.command === undefined) {
    throw new Error('Expected a file-level "Run & Preview" CodeLens.');
  }

  return runAndPreviewLens as {
    command: {
      arguments?: unknown[];
      command: string;
      title: string;
    };
  };
}

function joinPathEntries(...entries: Array<string | undefined>): string {
  return entries
    .filter((entry): entry is string => entry !== undefined)
    .join(path.delimiter);
}

function extractHoverMarkdown(hover: HoverResult | null): string {
  if (hover === null) {
    throw new Error("Expected hover information for the Type command.");
  }

  if (typeof hover.contents === "string") {
    return hover.contents;
  }

  if (Array.isArray(hover.contents)) {
    return hover.contents
      .map((value) => {
        if (typeof value === "string") {
          return value;
        }

        return value.value;
      })
      .join("\n\n");
  }

  return hover.contents.value;
}

async function waitForProcessExit(
  child: ChildProcessWithoutNullStreams,
): Promise<void> {
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }

  await new Promise<void>((resolve) => {
    const timeout = setTimeout(() => {
      if (child.exitCode === null && child.signalCode === null) {
        child.kill("SIGKILL");
      }
    }, 2_000);

    child.once("exit", () => {
      clearTimeout(timeout);
      resolve();
    });
  });
}

async function writeExecutableCommand(options: {
  binPath: string;
  commandName: string;
  scriptPath: string;
}): Promise<void> {
  const commandPath = path.join(options.binPath, options.commandName);

  if (process.platform === "win32") {
    await writeFile(
      `${commandPath}.cmd`,
      `@echo off\r\n"${process.execPath}" "${options.scriptPath}" %*\r\n`,
      "utf8",
    );
    return;
  }

  await writeFile(
    commandPath,
    `#!/bin/sh\nexec "${process.execPath}" "${options.scriptPath}" "$@"\n`,
    "utf8",
  );
  await chmod(commandPath, 0o755);
}
