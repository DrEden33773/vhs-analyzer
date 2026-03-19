import { Uri, env, window } from "vscode";

export interface RuntimeDependency {
  installUrl: string;
  name: "ffmpeg" | "ttyd" | "vhs";
}

export interface CheckRuntimeDependenciesOptions {
  resolveExecutable: (binaryName: string) => Promise<string | null>;
}

export const runtimeDependencies: readonly RuntimeDependency[] = [
  {
    name: "vhs",
    installUrl: "https://github.com/charmbracelet/vhs#installation",
  },
  {
    name: "ttyd",
    installUrl: "https://github.com/tsl0922/ttyd#installation",
  },
  {
    name: "ffmpeg",
    installUrl: "https://ffmpeg.org/download.html",
  },
];

export async function checkRuntimeDependencies(
  options: CheckRuntimeDependenciesOptions,
): Promise<void> {
  for (const dependency of runtimeDependencies) {
    const resolvedPath = await options.resolveExecutable(dependency.name);

    if (resolvedPath !== null) {
      continue;
    }

    const selection = await window.showInformationMessage(
      `${dependency.name} not found. Preview and Run features require ${dependency.name}.`,
      "Install",
    );

    if (selection === "Install") {
      await env.openExternal(Uri.parse(dependency.installUrl));
    }
  }
}
