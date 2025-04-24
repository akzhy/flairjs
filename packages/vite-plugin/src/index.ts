import { transform } from "jsx-styled-core";
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
      return transform({
        code,
        filePath: id,
        cssPreprocessor,
      });
    },
  };
}
