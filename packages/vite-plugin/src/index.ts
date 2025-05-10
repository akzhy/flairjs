import { transform } from "jsx-styled-core";
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
      const result = transform({
        code,
        filePath: id,
        cssPreprocessor,
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
