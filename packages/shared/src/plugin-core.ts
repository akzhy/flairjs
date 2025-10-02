import { FlairThemeConfig, buildThemeTokens } from "@flairjs/client";
import { existsSync, watch } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import module from "node:module";
import path from "node:path";
import { getUserTheme, GetUserThemeResult } from "./user-theme.js";
import { store } from "./store.js";

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
  userTheme: GetUserThemeResult | null;
  buildThemeCSS: (theme: FlairThemeConfig) => string;
  refreshCssFile: (sourceFilePath: string, cssFilePath: string) => void;
}

export const getGeneratedCssDir = (): string => {
  const flairThemeFile = require.resolve("@flairjs/client/theme.css");
  const flairGeneratedCssDir = path.resolve(flairThemeFile, "../generated-css");
  return flairGeneratedCssDir;
};

export const setupGeneratedCssDir = async (options?: {
  clearExisting?: boolean;
}): Promise<string | null> => {
  const flairGeneratedCssDir = getGeneratedCssDir();
  const { clearExisting = true } = options ?? {};

  // Setup generated CSS directory
  try {
    if (!existsSync(flairGeneratedCssDir)) {
      await mkdir(flairGeneratedCssDir);
    } else if (clearExisting) {
      await rm(flairGeneratedCssDir, { recursive: true, force: true });
      await mkdir(flairGeneratedCssDir);
    }
  } catch (err: any) {
    if (err?.code === "EEXIST") {
      return flairGeneratedCssDir;
    } else if (existsSync(flairGeneratedCssDir)) {
      return flairGeneratedCssDir;
    }

    console.error(
      `[flairjs] Could not create generated CSS directory: ${flairGeneratedCssDir}`,
      err
    );

    return null;
  }

  return flairGeneratedCssDir;
};

export const setupUserThemeFile = async ({
  buildThemeFile,
  onThemeFileChange,
  deleteBeforeWrite = false,
}: {
  buildThemeFile?: SharedPluginContext["buildThemeCSS"];
  onThemeFileChange?: () => void;
  deleteBeforeWrite?: boolean;
}) => {
  const flairThemeFile = require.resolve("@flairjs/client/theme.css");
  let userTheme = await getUserTheme();
  const buildThemeCSS = buildThemeFile ?? buildThemeTokens;

  if (userTheme) {
    const themeCSS = buildThemeCSS(userTheme.theme);
    store.setLastThemeUpdate(Date.now());
    
    if (deleteBeforeWrite) {
      // For some reason, in turbopack, overwriting the existing file throws an error.
      // So we delete the file first before writing to it.
      await rm(flairThemeFile, { force: true });
    }
    await writeFile(flairThemeFile, themeCSS, "utf-8");

    watch(userTheme.originalPath, async () => {
      userTheme = await getUserTheme();
      store.setLastThemeUpdate(Date.now());
      if (!userTheme) {
        return;
      }
      const themeCSS = buildThemeCSS(userTheme.theme);
      onThemeFileChange?.();
      await writeFile(flairThemeFile, themeCSS, "utf-8");
    });
  }

  return userTheme;
};

export const removeOutdatedCssFiles = async (
  sourceFilePath: string,
  cssFilePath: string,
  {
    flairGeneratedCssDir,
    clearInstantly = false,
  }: { flairGeneratedCssDir: string; clearInstantly?: boolean }
) => {
  const previousGeneratedCssName = store.getGeneratedCssName(sourceFilePath);
  if (previousGeneratedCssName && previousGeneratedCssName !== cssFilePath) {
    if (clearInstantly) {
      await rm(path.join(flairGeneratedCssDir, previousGeneratedCssName), {
        force: true,
      });
      store.setFileNameToGeneratedCssNameMap(sourceFilePath, cssFilePath);
      return;
    }

    setTimeout(() => {
      rm(path.join(flairGeneratedCssDir, previousGeneratedCssName), {
        force: true,
      });
    }, 2000);
  }
  store.setFileNameToGeneratedCssNameMap(sourceFilePath, cssFilePath);
};

/**
 * Initialize the shared plugin context
 */
export async function initializeSharedContext(
  options: SharedPluginOptions = {}
): Promise<SharedPluginContext | null> {
  const flairThemeFile = require.resolve("@flairjs/client/theme.css");
  const flairGeneratedCssDir = await setupGeneratedCssDir();
  const buildThemeCSS = options.buildThemeFile ?? buildThemeTokens;

  if (!flairGeneratedCssDir) {
    console.error("[flairjs] Could not setup generated CSS directory.");
    return null;
  }

  const userTheme = await getUserTheme();

  const refreshCssFile = (sourceFilePath: string, cssFilePath: string) => {
    removeOutdatedCssFiles(sourceFilePath, cssFilePath, {
      flairGeneratedCssDir,
    });
  };

  return {
    flairThemeFile,
    flairGeneratedCssDir,
    userTheme,
    buildThemeCSS,
    refreshCssFile,
  };
}
