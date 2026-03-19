import { describe, expect, it } from "vitest";

import { extractOutputPath } from "./utils";

describe("extractOutputPath", () => {
  it("extracts_unquoted_output_paths", () => {
    expect(extractOutputPath("Output demo.gif")).toBe("demo.gif");
  });

  it("extracts_quoted_output_paths", () => {
    expect(extractOutputPath('Output "demo file.gif"')).toBe("demo file.gif");
  });
});
