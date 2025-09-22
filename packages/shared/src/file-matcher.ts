import picomatch from "picomatch";

export function shouldProcessFile(
  id: string,
  include?: string | string[],
  exclude?: string | string[]
): boolean {
  // Create matchers for include and exclude patterns
  const isIncluded = picomatch(include || ["**/*.{js,ts,jsx,tsx}"]);
  const isExcluded = picomatch(exclude || ["node_modules/**"]);

  // Check if file matches include patterns
  if (!isIncluded(id)) {
    return false;
  }

  // Check if file matches exclude patterns
  if (isExcluded(id)) {
    return false;
  }

  return true;
}