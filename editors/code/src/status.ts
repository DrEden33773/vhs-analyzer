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
  private executionActive = false;
  private readonly item: StatusBarItem;
  private restoreTimer: ReturnType<typeof setTimeout> | undefined;
  private serverStatus: ServerStatus = "starting";

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
    this.serverStatus = status;
    if (this.executionActive || this.restoreTimer !== undefined) {
      return;
    }

    this.renderServerStatus();
  }

  setExecutionStatus(fileName: string): void {
    this.clearRestoreTimer();
    this.executionActive = true;
    this.item.text = `$(sync~spin) VHS: Running ${fileName}...`;
    this.item.color = undefined;
  }

  flashExecutionComplete(): void {
    this.flashExecutionState(
      "$(check) VHS: Done",
      new ThemeColor("charts.green"),
      3000,
    );
  }

  flashExecutionFailed(): void {
    this.flashExecutionState(
      "$(error) VHS: Failed",
      new ThemeColor("charts.red"),
      5000,
    );
  }

  clearExecutionStatus(): void {
    this.clearRestoreTimer();
    this.executionActive = false;
    this.renderServerStatus();
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
    this.clearRestoreTimer();
    this.item.dispose();
  }

  private flashExecutionState(
    text: string,
    color: ThemeColor,
    durationMs: number,
  ): void {
    this.clearRestoreTimer();
    this.executionActive = false;
    this.item.text = text;
    this.item.color = color;
    this.restoreTimer = setTimeout(() => {
      this.restoreTimer = undefined;
      this.renderServerStatus();
    }, durationMs);
  }

  private clearRestoreTimer(): void {
    if (this.restoreTimer === undefined) {
      return;
    }

    clearTimeout(this.restoreTimer);
    this.restoreTimer = undefined;
  }

  private renderServerStatus(): void {
    switch (this.serverStatus) {
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
}
