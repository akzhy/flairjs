import { defineConfig, RolldownOptions } from "rolldown";

const createOptions = (format: "esm" | "cjs"): RolldownOptions => {
  return {
    input: "src/index.ts",
    platform: "node",
    output: {
      dir: `dist/${format}`,
      format: format,
      esModule: true,
    },
    external: (id: string) => {
      return (
        id.endsWith(".node") ||
        id.includes("node_modules") ||
        id === "@flairjs/core" ||
        id === "picomatch" ||
        id === "esbuild" ||
        id === "rollup"
      );
    },
  };
};

export default defineConfig([createOptions("esm"), createOptions("cjs")]);
