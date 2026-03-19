import path from "node:path";
import { vi } from "vitest";

type Listener<T> = (event: T) => unknown;

export class Disposable {
  constructor(private readonly callback: () => void = () => {}) {}

  dispose(): void {
    this.callback();
  }
}

export class EventEmitter<T> {
  private readonly listeners = new Set<Listener<T>>();

  readonly event = (listener: Listener<T>): Disposable => {
    this.listeners.add(listener);
    return new Disposable(() => {
      this.listeners.delete(listener);
    });
  };

  fire(event: T): void {
    for (const listener of [...this.listeners]) {
      listener(event);
    }
  }

  dispose(): void {
    this.listeners.clear();
  }
}

export class Position {
  constructor(
    public readonly line: number,
    public readonly character: number,
  ) {}
}

export class Range {
  readonly start: Position;
  readonly end: Position;

  constructor(
    startLineOrPosition: number | Position,
    startCharacterOrEnd: number | Position,
    endLine?: number,
    endCharacter?: number,
  ) {
    if (
      startLineOrPosition instanceof Position &&
      startCharacterOrEnd instanceof Position
    ) {
      this.start = startLineOrPosition;
      this.end = startCharacterOrEnd;
      return;
    }

    this.start = new Position(
      startLineOrPosition as number,
      startCharacterOrEnd as number,
    );
    this.end = new Position(endLine ?? 0, endCharacter ?? 0);
  }
}

export class ThemeColor {
  constructor(public readonly id: string) {}
}

export const ColorThemeKind = {
  Light: 1,
  Dark: 2,
  HighContrast: 3,
  HighContrastLight: 4,
} as const;

export class Uri {
  readonly authority: string = "";
  readonly fragment: string = "";

  constructor(
    public readonly scheme: string,
    public readonly fsPath: string,
    public readonly query = "",
  ) {}

  static file(filePath: string): Uri {
    return new Uri("file", path.resolve(filePath));
  }

  static parse(value: string): Uri {
    if (value.startsWith("file://")) {
      return new Uri("file", value.replace("file://", ""));
    }

    return new Uri("https", value);
  }

  get path(): string {
    return this.fsPath;
  }

  with(changes: { query?: string }): Uri {
    return new Uri(this.scheme, this.fsPath, changes.query ?? this.query);
  }

  toString(): string {
    const suffix = this.query ? `?${this.query}` : "";

    if (this.scheme === "file") {
      return `file://${this.fsPath}${suffix}`;
    }

    return `${this.fsPath}${suffix}`;
  }

  toJSON(): { fsPath: string; path: string; scheme: string } {
    return {
      fsPath: this.fsPath,
      path: this.path,
      scheme: this.scheme,
    };
  }
}

export const StatusBarAlignment = {
  Left: 1,
  Right: 2,
} as const;

export const ViewColumn = {
  Active: -1,
  Beside: 2,
} as const;

type ConfigurationStore = Record<string, unknown>;

const configurationValues: ConfigurationStore = {
  "vhs-analyzer.server.path": "",
  "vhs-analyzer.server.args": [],
  "vhs-analyzer.trace.server": "off",
  "vhs-analyzer.preview.autoRefresh": true,
  "vhs-analyzer.codelens.enabled": true,
};

const commandHandlers = new Map<string, (...args: unknown[]) => unknown>();
const commandContexts = new Map<string, unknown>();
const configurationEmitter = new EventEmitter<ConfigurationChangeEvent>();
const colorThemeEmitter = new EventEmitter<{ kind: number }>();
const createdFileSystemWatchers: Array<
  ReturnType<typeof createFileSystemWatcher>
> = [];
const createdWebviewPanels: Array<ReturnType<typeof createWebviewPanel>> = [];

function buildConfigurationKey(
  section: string | undefined,
  key: string,
): string {
  return section ? `${section}.${key}` : key;
}

function getConfigurationValue<T>(fullKey: string, fallback: T): T {
  return (configurationValues[fullKey] as T | undefined) ?? fallback;
}

export interface ConfigurationChangeEvent {
  affectsConfiguration(section: string): boolean;
}

export interface WorkspaceConfiguration {
  get<T>(key: string, defaultValue?: T): T;
  update(key: string, value: unknown): Promise<void>;
}

function createConfiguration(section?: string): WorkspaceConfiguration {
  return {
    get<T>(key: string, defaultValue?: T): T {
      return getConfigurationValue(
        buildConfigurationKey(section, key),
        defaultValue as T,
      );
    },
    async update(key: string, value: unknown): Promise<void> {
      configurationValues[buildConfigurationKey(section, key)] = value;
    },
  };
}

function createOutputChannel(name: string) {
  const lines: string[] = [];

  return {
    name,
    lines,
    append: vi.fn((value: string) => {
      lines.push(value);
    }),
    appendLine: vi.fn((value: string) => {
      lines.push(`${value}\n`);
    }),
    clear: vi.fn(() => {
      lines.length = 0;
    }),
    replace: vi.fn((value: string) => {
      lines.length = 0;
      lines.push(value);
    }),
    show: vi.fn(),
    hide: vi.fn(),
    dispose: vi.fn(),
  };
}

function createStatusBarItem() {
  return {
    text: "",
    tooltip: "",
    color: undefined as ThemeColor | undefined,
    command: undefined as string | undefined,
    show: vi.fn(),
    hide: vi.fn(),
    dispose: vi.fn(),
  };
}

function createFileSystemWatcher(pattern: string) {
  const changeEmitter = new EventEmitter<Uri>();
  const createEmitter = new EventEmitter<Uri>();
  const deleteEmitter = new EventEmitter<Uri>();

  return {
    pattern,
    ignoreCreateEvents: false,
    ignoreChangeEvents: false,
    ignoreDeleteEvents: false,
    onDidChange: changeEmitter.event,
    onDidCreate: createEmitter.event,
    onDidDelete: deleteEmitter.event,
    __fireDidChange: (uri: Uri) => {
      changeEmitter.fire(uri);
    },
    __fireDidCreate: (uri: Uri) => {
      createEmitter.fire(uri);
    },
    __fireDidDelete: (uri: Uri) => {
      deleteEmitter.fire(uri);
    },
    dispose: vi.fn(() => {
      changeEmitter.dispose();
      createEmitter.dispose();
      deleteEmitter.dispose();
    }),
  };
}

function createWebviewPanel(
  viewType: string,
  title: string,
  viewColumn: number,
  options: {
    enableScripts?: boolean;
    localResourceRoots?: Uri[];
    retainContextWhenHidden?: boolean;
  },
) {
  const disposeEmitter = new EventEmitter<void>();
  const receiveMessageEmitter = new EventEmitter<unknown>();
  const postedMessages: unknown[] = [];
  const webview = {
    cspSource: "mock-webview-source",
    html: "",
    options: {
      enableScripts: options.enableScripts ?? false,
      localResourceRoots: options.localResourceRoots ?? [],
    },
    asWebviewUri: vi.fn(
      (uri: Uri) => new Uri("webview", uri.fsPath, uri.query),
    ),
    postMessage: vi.fn(async (message: unknown) => {
      postedMessages.push(message);
      return true;
    }),
    onDidReceiveMessage: receiveMessageEmitter.event,
    __fireDidReceiveMessage: (message: unknown) => {
      receiveMessageEmitter.fire(message);
    },
    postedMessages,
  };
  const panel = {
    active: true,
    iconPath: undefined as Uri | undefined,
    onDidDispose: disposeEmitter.event,
    options,
    reveal: vi.fn((targetViewColumn?: number) => {
      panel.viewColumn = targetViewColumn ?? panel.viewColumn;
      panel.visible = true;
    }),
    title,
    viewColumn,
    viewType,
    visible: true,
    webview,
    dispose: vi.fn(() => {
      panel.visible = false;
      disposeEmitter.fire();
    }),
    __fireDidDispose: () => {
      panel.dispose();
    },
  };

  return panel;
}

export const workspace = {
  workspaceFolders: [] as Array<{ uri: Uri }>,
  getConfiguration: vi.fn((section?: string) => createConfiguration(section)),
  onDidChangeConfiguration: configurationEmitter.event,
  createFileSystemWatcher: vi.fn((pattern: string) => {
    const watcher = createFileSystemWatcher(pattern);
    createdFileSystemWatchers.push(watcher);
    return watcher;
  }),
};

export const window = {
  activeColorTheme: {
    kind: ColorThemeKind.Dark,
  } as { kind: number },
  createWebviewPanel: vi.fn(
    (
      viewType: string,
      title: string,
      showOptions: number,
      options: {
        enableScripts?: boolean;
        localResourceRoots?: Uri[];
        retainContextWhenHidden?: boolean;
      },
    ) => {
      const panel = createWebviewPanel(viewType, title, showOptions, options);
      createdWebviewPanels.push(panel);
      return panel;
    },
  ),
  createOutputChannel: vi.fn((name: string) => createOutputChannel(name)),
  createStatusBarItem: vi.fn(() => createStatusBarItem()),
  onDidChangeActiveColorTheme: colorThemeEmitter.event,
  showInformationMessage: vi.fn(
    async (_message: string, ...items: string[]) => items[0],
  ),
  showErrorMessage: vi.fn(
    async (_message: string, ...items: string[]) => items[0],
  ),
  showQuickPick: vi.fn(
    async (items: Array<string | { label: string }>) => items[0],
  ),
};

export const env = {
  openExternal: vi.fn(async () => true),
};

export const commands = {
  registerCommand: vi.fn(
    (command: string, handler: (...args: unknown[]) => unknown) => {
      commandHandlers.set(command, handler);
      return new Disposable(() => {
        commandHandlers.delete(command);
      });
    },
  ),
  executeCommand: vi.fn(async (command: string, ...args: unknown[]) => {
    if (command === "setContext") {
      const [key, value] = args;
      commandContexts.set(String(key), value);
      return;
    }

    return commandHandlers.get(command)?.(...args);
  }),
};

export function createMockMemento(initialValues: Record<string, unknown> = {}) {
  const values = new Map(Object.entries(initialValues));

  return {
    get<T>(key: string, defaultValue?: T): T {
      return (values.get(key) as T | undefined) ?? (defaultValue as T);
    },
    async update(key: string, value: unknown): Promise<void> {
      values.set(key, value);
    },
  };
}

export function createMockExtensionContext(
  overrides: {
    extensionPath?: string;
    globalState?: Record<string, unknown>;
  } = {},
) {
  return {
    extensionPath: overrides.extensionPath ?? "/mock-extension",
    subscriptions: [] as Disposable[],
    globalState: createMockMemento(overrides.globalState),
  };
}

export function __setConfigurationValue(key: string, value: unknown): void {
  configurationValues[key] = value;
}

export function __fireConfigurationChange(keys: string[]): void {
  configurationEmitter.fire({
    affectsConfiguration(section: string): boolean {
      return keys.some(
        (key) => key === section || key.startsWith(`${section}.`),
      );
    },
  });
}

export function __setWorkspaceFolders(paths: string[]): void {
  workspace.workspaceFolders = paths.map((workspacePath) => ({
    uri: Uri.file(workspacePath),
  }));
}

export function __getCommandContext(key: string): unknown {
  return commandContexts.get(key);
}

export function __getCreatedFileSystemWatchers(): Array<
  ReturnType<typeof createFileSystemWatcher>
> {
  return [...createdFileSystemWatchers];
}

export function __getCreatedWebviewPanels(): Array<
  ReturnType<typeof createWebviewPanel>
> {
  return [...createdWebviewPanels];
}

export function __getLastCreatedWebviewPanel():
  | ReturnType<typeof createWebviewPanel>
  | undefined {
  return createdWebviewPanels.at(-1);
}

export function __setActiveColorTheme(kind: number): void {
  window.activeColorTheme.kind = kind;
  colorThemeEmitter.fire({
    kind,
  });
}

export function __resetMockVscode(): void {
  workspace.workspaceFolders = [];
  configurationEmitter.dispose();
  colorThemeEmitter.dispose();

  for (const [key, value] of Object.entries({
    "vhs-analyzer.server.path": "",
    "vhs-analyzer.server.args": [],
    "vhs-analyzer.trace.server": "off",
    "vhs-analyzer.preview.autoRefresh": true,
    "vhs-analyzer.codelens.enabled": true,
  })) {
    configurationValues[key] = value;
  }

  commandHandlers.clear();
  commandContexts.clear();
  createdFileSystemWatchers.length = 0;
  createdWebviewPanels.length = 0;
  window.activeColorTheme.kind = ColorThemeKind.Dark;

  workspace.getConfiguration.mockClear();
  workspace.createFileSystemWatcher.mockClear();
  window.createWebviewPanel.mockClear();
  window.createOutputChannel.mockClear();
  window.createStatusBarItem.mockClear();
  window.showInformationMessage.mockClear();
  window.showErrorMessage.mockClear();
  window.showQuickPick.mockClear();
  env.openExternal.mockClear();
  commands.registerCommand.mockClear();
  commands.executeCommand.mockClear();
}
