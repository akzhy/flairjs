export {
  getGeneratedCssDir,
  initializeSharedContext,
  removeOutdatedCssFiles,
  setupGeneratedCssDir,
  setupUserThemeFile,
  type SharedPluginOptions,
} from "./plugin-core.js";

export { shouldProcessFile } from "./file-matcher.js";
export { transformCode } from "./transform.js";
export { getUserTheme } from "./user-theme.js";
