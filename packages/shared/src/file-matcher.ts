import picomatch from "picomatch";

export function shouldProcessFile(
  id: string,
  include?: string | string[],
  exclude?: string | string[]
): boolean {
  const isIncluded = picomatch(include ?? ["**/*.{js,ts,jsx,tsx}"]);
  const isExcluded = picomatch(exclude ?? ["node_modules/**"]);

  // Check if file matches include patterns
  if (!isIncluded(normalizeFilePath(id))) {
    return false;
  }

  // Check if file matches exclude patterns
  if (isExcluded(normalizeFilePath(id))) {
    return false;
  }

  return true;
}

function normalizeFilePath(filePath: string): string {
  return filePath.replace(/\\/g, '/');
}