import { afterEach, describe, expect, it, vi } from "vitest";

import { __resetMockVscode, env, window } from "./__mocks__/vscode";
import { checkRuntimeDependencies } from "./dependencies";

describe("checkRuntimeDependencies", () => {
  afterEach(() => {
    __resetMockVscode();
    vi.restoreAllMocks();
  });

  it("runtime_dependency_detection_shows_vhs_install_prompt_when_missing", async () => {
    window.showInformationMessage.mockResolvedValue("Install");
    const resolveExecutable = vi
      .fn<(binaryName: string) => Promise<string | null>>()
      .mockResolvedValueOnce(null)
      .mockResolvedValueOnce("/usr/bin/ttyd")
      .mockResolvedValueOnce("/usr/bin/ffmpeg");

    await checkRuntimeDependencies({
      resolveExecutable,
    });

    expect(window.showInformationMessage).toHaveBeenCalledWith(
      "vhs not found. Preview and Run features require vhs.",
      "Install",
    );
    expect(env.openExternal).toHaveBeenCalledTimes(1);
    const openedUri = env.openExternal.mock.calls.at(0)?.at(0) as
      | { toString(): string }
      | undefined;
    expect(openedUri).toBeDefined();
    expect(openedUri?.toString()).toBe(
      "https://github.com/charmbracelet/vhs#installation",
    );
  });
});
