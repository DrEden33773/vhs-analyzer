// Keep Output parsing consistent across preview and CodeLens.
export const OUTPUT_DIRECTIVE_REGEX = /^Output\s+(?:["'](.+?)["']|(\S+))/m;

export function extractOutputPath(source: string): string | null {
  const match = OUTPUT_DIRECTIVE_REGEX.exec(source);
  return match?.[1] ?? match?.[2] ?? null;
}
