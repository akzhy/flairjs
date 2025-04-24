import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import jsxStyledVitePlugin from "jsx-styled-vite-plugin";
import { compileString} from "sass";
// https://vite.dev/config/
export default defineConfig({
  plugins: [
    jsxStyledVitePlugin({
      cssPreprocessor: (css, id) => compileString(css).css,
    }),
    react(),
  ],
  clearScreen: false,
});
