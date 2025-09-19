import { transformCode } from "@flairjs/core";
import type { Plugin } from "vite";
import module from "node:module";
import path from "node:path";
import { existsSync, watch } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import { getUserTheme } from "./user-theme";
import { buildThemeTokens } from "@flairjs/core";
import picomatch from "picomatch";

const require = module.createRequire(import.meta.url);

function shouldProcessFile(
  id: string,
  include?: string | string[],
  exclude?: string | string[]
): boolean {
  // Create matchers for include and exclude patterns
  const isIncluded = picomatch(include || ["**/*.{js,ts,jsx,tsx}"]);
  const isExcluded = picomatch(exclude || []);

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

export default async function flairJsVitePlugin({
  cssPreprocessor,
  include,
  exclude = ["node_modules/**"],
}: {
  cssPreprocessor?: (css: string, id: string) => string;
  include?: string | string[];
  exclude?: string | string[];
}): Promise<Plugin> {
  const flairThemeFile = require.resolve("@flairjs/client/theme.css");

  const flairGeneratedCssPath = path.resolve(
    flairThemeFile,
    "../generated-css"
  );

  if (!existsSync(flairGeneratedCssPath)) {
    await mkdir(flairGeneratedCssPath);
  }

  let userTheme = await getUserTheme();

  if (userTheme) {
    const themeCSS = buildThemeTokens(userTheme.theme);
    await writeFile(flairThemeFile, themeCSS, "utf-8");

    watch(userTheme.path, async (event) => {
      userTheme = await getUserTheme();
      if (!userTheme) {
        return;
      }
      const themeCSS = buildThemeTokens(userTheme.theme);
      await writeFile(flairThemeFile, themeCSS, "utf-8");
    });
  }

  return {
    name: "@flairjs/vite-plugin",
    enforce: "pre",
    transform(code, id) {
      // Check if the file should be transformed based on include/exclude patterns
      if (!shouldProcessFile(id, include, exclude)) {
        return code;
      }

      const result = transformCode(code, id, {
        cssOutDir: flairGeneratedCssPath,
        useTheme: !!userTheme,
        theme: {
          breakpoints: userTheme?.theme?.breakpoints,
          prefix: userTheme?.theme?.prefix,
        },
        cssPreprocessor: cssPreprocessor
          ? (css) => cssPreprocessor(css, id)
          : undefined,
      });

      if (!result) {
        return code;
      }

      return {
        code: result.code,
        map: result.sourcemap,
      };
    },
  };
}
