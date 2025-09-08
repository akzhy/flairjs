import { transformCode } from "@flairjs/core";
import type { Plugin } from "vite";
import { writeFileSync } from "fs";

export default function jsxStyledVitePlugin({
  cssPreprocessor,
}: {
  cssPreprocessor?: (css: string, id: string) => string;
}): Plugin {
  return {
    name: "vite-plugin-jsx-styled",
    enforce: "pre",
    transform(code, id) {
      const result = transformCode({
        code,
        filePath: id,
        // cssPreprocessor,
        cssOutDir: ''
      });

      
      if (!result) {
        return code;
      }

      return {
        code: result.code,
        map: result.map,
      };
    },
  };
}
