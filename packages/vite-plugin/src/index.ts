import { transformCode } from "@flairjs/core";
import type { Plugin } from "vite";
import module from "node:module";
import path from "node:path";
import { existsSync } from "node:fs";
import { mkdir } from "node:fs/promises";

const require = module.createRequire(import.meta.url);

export default async function flairJsVitePlugin({
  cssPreprocessor,
}: {
  cssPreprocessor?: (css: string, id: string) => string;
}): Promise<Plugin> {
  const flairThemeFile = require.resolve("@flairjs/client/theme");

  const flairGeneratedCssPath = path.resolve(flairThemeFile, "../generated-css");

  if (!existsSync(flairGeneratedCssPath)) {
    await mkdir(flairGeneratedCssPath);
  }
  
  return {
    name: "@flairjs/vite-plugin",
    enforce: "pre",
    transform(code, id) {
      const result = transformCode(code, id, {
        cssOutDir: flairGeneratedCssPath
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
