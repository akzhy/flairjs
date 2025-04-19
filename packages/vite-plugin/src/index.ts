import { transform } from "jsx-styled-core";
import type { Plugin } from "vite";

export default function jsxStyledVitePlugin(): Plugin {
  return {
    name: "vite-plugin-jsx-styled",
    enforce: "pre",
    transform(code, id) {
      return transform(code, id);
    },
  };
}
