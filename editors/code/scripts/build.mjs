import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { build, context } from "esbuild";

const projectRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const isWatchMode = process.argv.includes("--watch");

const rawImportPlugin = {
  name: "raw-import",
  setup(buildContext) {
    buildContext.onResolve({ filter: /\?raw$/ }, (args) => {
      const importPath = args.path.slice(0, -4);
      return {
        namespace: "raw-import",
        path: path.isAbsolute(importPath)
          ? importPath
          : path.resolve(args.resolveDir, importPath),
      };
    });

    buildContext.onLoad(
      { filter: /.*/, namespace: "raw-import" },
      async (args) => {
        const contents = await readFile(args.path, "utf8");
        return {
          contents,
          loader: "text",
          watchFiles: [args.path],
        };
      },
    );
  },
};

const buildOptions = {
  bundle: true,
  entryPoints: [path.join(projectRoot, "src/extension.ts")],
  external: ["vscode"],
  format: "cjs",
  minify: !isWatchMode,
  outfile: path.join(projectRoot, "dist/extension.js"),
  platform: "node",
  plugins: [rawImportPlugin],
  sourcemap: isWatchMode,
};

if (isWatchMode) {
  const watchContext = await context(buildOptions);
  await watchContext.watch();

  const disposeAndExit = async () => {
    await watchContext.dispose();
    process.exit(0);
  };

  process.on("SIGINT", () => {
    void disposeAndExit();
  });
  process.on("SIGTERM", () => {
    void disposeAndExit();
  });
} else {
  await build(buildOptions);
}
