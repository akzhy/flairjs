import {
  initializeSharedContext,
  shouldProcessFile,
} from "@flairjs/bundler-shared";
import { FlairThemeConfig, transformCode } from "@flairjs/core";
import type { Plugin } from "rollup";

interface FlairJsRollupPluginOptions {
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
}

export default async function flairJsRollupPlugin(
  options?: FlairJsRollupPluginOptions
): Promise<Plugin> {
  const context = await initializeSharedContext(options);

  return {
    name: "@flairjs/rollup-plugin",
    transform(code, id) {
      if (!shouldProcessFile(id, options?.include, options?.exclude)) {
        return null;
      }

      const result = transformCode(code, id, {
        appendTimestampToCssFile: true,
        classNameList: [],
        cssPreprocessor: options?.cssPreprocessor
          ? (css: string) => options.cssPreprocessor!(css, id)
          : undefined,
        theme: context.userTheme?.theme,
        useTheme: !!context.userTheme,
        cssOutDir: context.flairGeneratedCssDir,
      });

      if (!result) {
        return null;
      }

      if (result.generatedCssName) {
        context.refreshCssFile(result.generatedCssName);
      }

      return {
        code: result.code,
        map: JSON.parse(result.sourcemap ?? "{}"),
      };
    },
  };
}
