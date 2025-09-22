import { FlairThemeConfig, buildThemeTokens } from "@flairjs/core";
import { existsSync, watch } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import module from "node:module";
import path from "node:path";
import { getUserTheme } from "./user-theme.js";

const require = module.createRequire(import.meta.url);

export interface SharedPluginOptions {
  /**
   * Preprocess the extracted CSS before it is passed to lightningcss
   * @experimental
   * @param css the extracted css
   * @param id the id of the file being processed
   * @returns the processed css
   */
  cssPreprocessor?: (css: string, id: string) => string;
  include?: string | string[];
  exclude?: string | string[];
  /**
   * Override the default theme file content based on the user theme
   * @param theme the user theme
   * @returns the theme file content
   */
  buildThemeFile?: (theme: FlairThemeConfig) => string;

  /**
   * List of class names used in the project. Supports regex.
   */
  classNameList?: string[];
}

interface SharedPluginContext {
  flairThemeFile: string;
  flairGeneratedCssDir: string;
  userTheme: Awaited<ReturnType<typeof getUserTheme>>;
  buildThemeCSS: (theme: FlairThemeConfig) => string;
  refreshCssFile: (filePath: string) => void;
}

/**
 * Initialize the shared plugin context
 */
export async function initializeSharedContext(
  options: SharedPluginOptions = {}
): Promise<SharedPluginContext> {
  const { buildThemeFile } = options;
  const fileNameToGeneratedCssNameMap = new Map<string, string>();

  const flairThemeFile = require.resolve("@flairjs/client/theme.css");
  const flairGeneratedCssDir = path.resolve(flairThemeFile, "../generated-css");

  // Setup generated CSS directory
  if (!existsSync(flairGeneratedCssDir)) {
    await mkdir(flairGeneratedCssDir);
  } else {
    await rm(flairGeneratedCssDir, { recursive: true, force: true });
    await mkdir(flairGeneratedCssDir);
  }

  // Load user theme
  let userTheme = await getUserTheme();
  const buildThemeCSS = buildThemeFile ?? buildThemeTokens;

  // Setup theme file and watcher
  if (userTheme) {
    const themeCSS = buildThemeCSS(userTheme.theme);
    await writeFile(flairThemeFile, themeCSS, "utf-8");

    watch(userTheme.originalPath, async () => {
      userTheme = await getUserTheme();
      if (!userTheme) {
        return;
      }
      const themeCSS = buildThemeCSS(userTheme.theme);
      await writeFile(flairThemeFile, themeCSS, "utf-8");
    });
  }

  const refreshCssFile = (filePath: string) => {
    if (fileNameToGeneratedCssNameMap.has(filePath)) {
      const previousGeneratedCssName =
        fileNameToGeneratedCssNameMap.get(filePath);
      setTimeout(() => {
        rm(path.join(flairGeneratedCssDir, previousGeneratedCssName!));
      }, 2000);
    }
  };

  return {
    flairThemeFile,
    flairGeneratedCssDir,
    userTheme,
    buildThemeCSS,
    refreshCssFile,
  };
}
