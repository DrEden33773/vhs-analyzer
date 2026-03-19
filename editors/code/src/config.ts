import {
  type ConfigurationChangeEvent,
  type Disposable,
  workspace,
} from "vscode";

export const extensionSection = "vhs-analyzer";

export type ServerTraceLevel = "messages" | "off" | "verbose";

export interface ExtensionConfiguration {
  codelensEnabled: boolean;
  previewAutoRefresh: boolean;
  serverArgs: string[];
  serverPath: string;
  traceServer: ServerTraceLevel;
}

export interface ConfigurationChangeHandlers {
  onImmediateConfigurationChange: (
    configuration: ExtensionConfiguration,
  ) => void;
  onRestartRequired: () => Promise<void>;
  onTraceLevelChange: (traceLevel: ServerTraceLevel) => Promise<void> | void;
}

export function getExtensionConfiguration(): ExtensionConfiguration {
  const configuration = workspace.getConfiguration(extensionSection);

  return {
    codelensEnabled: configuration.get("codelens.enabled", true),
    previewAutoRefresh: configuration.get("preview.autoRefresh", true),
    serverArgs: configuration.get("server.args", []),
    serverPath: configuration.get("server.path", ""),
    traceServer: configuration.get("trace.server", "off"),
  };
}

export function createConfigurationChangeHandler(
  handlers: ConfigurationChangeHandlers,
): (event: ConfigurationChangeEvent) => Promise<void> {
  return async (event) => {
    if (!event.affectsConfiguration(extensionSection)) {
      return;
    }

    const configuration = getExtensionConfiguration();

    if (
      event.affectsConfiguration(`${extensionSection}.server.path`) ||
      event.affectsConfiguration(`${extensionSection}.server.args`)
    ) {
      await handlers.onRestartRequired();
      return;
    }

    if (event.affectsConfiguration(`${extensionSection}.trace.server`)) {
      await handlers.onTraceLevelChange(configuration.traceServer);
    }

    handlers.onImmediateConfigurationChange(configuration);
  };
}

export function registerConfigurationChangeHandler(
  handlers: ConfigurationChangeHandlers,
): Disposable {
  return workspace.onDidChangeConfiguration(
    createConfigurationChangeHandler(handlers),
  );
}
