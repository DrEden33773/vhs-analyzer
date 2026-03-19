import { afterEach, describe, expect, it, vi } from "vitest";

import { __resetMockVscode, __setConfigurationValue } from "./__mocks__/vscode";
import {
  createConfigurationChangeHandler,
  getExtensionConfiguration,
} from "./config";

describe("configuration", () => {
  afterEach(() => {
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("configuration_reader_returns_batch1_defaults", () => {
    expect(getExtensionConfiguration()).toEqual({
      codelensEnabled: true,
      previewAutoRefresh: true,
      serverArgs: [],
      serverPath: "",
      traceServer: "off",
    });
  });

  it("config_change_triggers_server_restart", async () => {
    const onRestartRequired = vi.fn().mockResolvedValue(undefined);
    const onTraceLevelChange = vi.fn();
    const onImmediateConfigurationChange = vi.fn();

    const handleConfigurationChange = createConfigurationChangeHandler({
      onImmediateConfigurationChange,
      onRestartRequired,
      onTraceLevelChange,
    });

    await handleConfigurationChange({
      affectsConfiguration(section: string): boolean {
        return (
          section === "vhs-analyzer" || section === "vhs-analyzer.server.path"
        );
      },
    });

    expect(onRestartRequired).toHaveBeenCalledTimes(1);
    expect(onTraceLevelChange).not.toHaveBeenCalled();
    expect(onImmediateConfigurationChange).not.toHaveBeenCalled();
  });

  it("trace_change_updates_trace_without_restart", async () => {
    __setConfigurationValue("vhs-analyzer.trace.server", "verbose");
    const onRestartRequired = vi.fn().mockResolvedValue(undefined);
    const onTraceLevelChange = vi.fn();
    const onImmediateConfigurationChange = vi.fn();

    const handleConfigurationChange = createConfigurationChangeHandler({
      onImmediateConfigurationChange,
      onRestartRequired,
      onTraceLevelChange,
    });

    await handleConfigurationChange({
      affectsConfiguration(section: string): boolean {
        return (
          section === "vhs-analyzer" || section === "vhs-analyzer.trace.server"
        );
      },
    });

    expect(onRestartRequired).not.toHaveBeenCalled();
    expect(onTraceLevelChange).toHaveBeenCalledWith("verbose");
    expect(onImmediateConfigurationChange).toHaveBeenCalledWith({
      codelensEnabled: true,
      previewAutoRefresh: true,
      serverArgs: [],
      serverPath: "",
      traceServer: "verbose",
    });
  });

  it("other_setting_changes_apply_immediately_without_restart", async () => {
    __setConfigurationValue("vhs-analyzer.codelens.enabled", false);
    const onRestartRequired = vi.fn().mockResolvedValue(undefined);
    const onTraceLevelChange = vi.fn();
    const onImmediateConfigurationChange = vi.fn();

    const handleConfigurationChange = createConfigurationChangeHandler({
      onImmediateConfigurationChange,
      onRestartRequired,
      onTraceLevelChange,
    });

    await handleConfigurationChange({
      affectsConfiguration(section: string): boolean {
        return (
          section === "vhs-analyzer" ||
          section === "vhs-analyzer.codelens.enabled"
        );
      },
    });

    expect(onRestartRequired).not.toHaveBeenCalled();
    expect(onTraceLevelChange).not.toHaveBeenCalled();
    expect(onImmediateConfigurationChange).toHaveBeenCalledWith({
      codelensEnabled: false,
      previewAutoRefresh: true,
      serverArgs: [],
      serverPath: "",
      traceServer: "off",
    });
  });
});
