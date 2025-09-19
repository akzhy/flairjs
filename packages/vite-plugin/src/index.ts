import { transformCode } from "@flairjs/core";
import type { Plugin } from "vite";
import module from "node:module";
import path from "node:path";
import { existsSync } from "node:fs";
import { mkdir, writeFile } from "node:fs/promises";
import { getUserTheme } from "./user-theme";
import { buildThemeTokens } from "@flairjs/core";

const require = module.createRequire(import.meta.url);

export default async function flairJsVitePlugin({
  cssPreprocessor,
}: {
  cssPreprocessor?: (css: string, id: string) => string;
}): Promise<Plugin> {
  const flairThemeFile = require.resolve("@flairjs/client/theme.css");

  const flairGeneratedCssPath = path.resolve(
    flairThemeFile,
    "../generated-css"
  );

  if (!existsSync(flairGeneratedCssPath)) {
    await mkdir(flairGeneratedCssPath);
  }
  const userTheme = getUserTheme();

  if (userTheme) {
    const themeCSS = buildThemeTokens(userTheme);

    await writeFile(flairThemeFile, themeCSS, "utf-8");
  }

  return {
    name: "@flairjs/vite-plugin",
    enforce: "pre",
    transform(code, id) {
      const result = transformCode(
        code,
        id,
        {
          cssOutDir: flairGeneratedCssPath,
          useTheme: !!userTheme,
          theme: {
            breakpoints: userTheme?.breakpoints,
            prefix: userTheme?.prefix,
          },
        },
        (css) => {
          if (cssPreprocessor) {
            return cssPreprocessor(css, id);
          }
        }
      );

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
