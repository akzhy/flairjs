import { transformCode } from "@flairjs/core";
import type { Plugin } from "vite";

export default function jsxStyledVitePlugin({
  cssPreprocessor,
}: {
  cssPreprocessor?: (css: string, id: string) => string;
}): Plugin {
  return {
    name: "vite-plugin-jsx-styled",
    enforce: "pre",
    transform(code, id) {
      const result = transformCode(code, id, {
        cssOutDir: ''
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
