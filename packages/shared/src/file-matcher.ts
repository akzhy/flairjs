import picomatch from "picomatch";

export function shouldProcessFile(
  id: string,
  include?: string | string[],
  exclude?: string | string[]
): boolean {
  const normalizePatterns = (patterns: string | string[] | undefined, defaults: string[]) => {
    const patternsArray = patterns ? (Array.isArray(patterns) ? patterns : [patterns]) : defaults;
    return patternsArray.map(pattern => pattern.replace(/\\/g, '/'));
  };

  const isIncluded = picomatch(normalizePatterns(include, ["**/*.{js,ts,jsx,tsx}"]));
  const isExcluded = picomatch(normalizePatterns(exclude, ["node_modules/**"]));

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