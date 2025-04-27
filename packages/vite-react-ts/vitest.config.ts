import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react-swc";
import jsxStyledVitePlugin from "jsx-styled-vite-plugin";
import { compileString } from "sass";

export default defineConfig({
  plugins: [
    jsxStyledVitePlugin({
      cssPreprocessor: (css) => compileString(css).css,
    }),
    react(),
  ],
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./tests/setup.ts",
    css: true,
  },
  server: {
    watch: {
      ignored: ["**/vite-plugin/**"],
    },
  },
});
