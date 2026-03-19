import {
  type Disposable,
  StatusBarAlignment,
  type StatusBarItem,
  ThemeColor,
  window,
} from "vscode";

export type ServerStatus = "failed" | "running" | "starting";

export interface StatusActionHandlers {
  onRestartServer: () => Promise<void> | void;
  onShowOutput: () => void;
  onShowTrace: () => void;
}

export class StatusController implements Disposable {
  private readonly item: StatusBarItem;

  constructor(
    private readonly handlers: StatusActionHandlers,
    command = "vhs-analyzer.statusActions",
  ) {
    this.item = window.createStatusBarItem(StatusBarAlignment.Left);
    this.item.command = command;
    this.item.tooltip = "VHS Analyzer";
    this.setServerStatus("starting");
    this.item.show();
  }

  setServerStatus(status: ServerStatus): void {
    switch (status) {
      case "running":
        this.item.text = "VHS $(check)";
        this.item.color = new ThemeColor("charts.green");
        return;
      case "failed":
        this.item.text = "VHS $(error)";
        this.item.color = new ThemeColor("charts.red");
        return;
      case "starting":
        this.item.text = "VHS $(warning)";
        this.item.color = new ThemeColor("charts.yellow");
        return;
    }
  }

  setExecutionStatus(fileName: string): void {
    this.item.text = `$(sync~spin) VHS: Running ${fileName}...`;
    this.item.color = undefined;
  }

  async showActions(): Promise<void> {
    const selection = await window.showQuickPick([
      "Restart Server",
      "Show Output",
      "Show Trace",
    ]);

    switch (selection) {
      case "Restart Server":
        await this.handlers.onRestartServer();
        return;
      case "Show Output":
        this.handlers.onShowOutput();
        return;
      case "Show Trace":
        this.handlers.onShowTrace();
        return;
    }
  }

  get statusBarItem(): StatusBarItem {
    return this.item;
  }

  dispose(): void {
    this.item.dispose();
  }
}
