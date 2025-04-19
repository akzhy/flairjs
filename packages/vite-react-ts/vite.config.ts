import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import jsxStyledVitePlugin from "jsx-styled-vite-plugin";
// https://vite.dev/config/
export default defineConfig({
  plugins: [jsxStyledVitePlugin(), react()],
  clearScreen: false,
});
