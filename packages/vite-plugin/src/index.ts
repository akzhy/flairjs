import {
  initializeSharedContext,
  SharedPluginOptions,
  shouldProcessFile
} from "@flairjs/bundler-shared";
import { transformCode } from "@flairjs/core";
import type { Plugin } from "vite";

interface FlairJsVitePluginOptions extends SharedPluginOptions {};

export default async function flairJsVitePlugin(
  options?: FlairJsVitePluginOptions
): Promise<Plugin> {
  const context = await initializeSharedContext(options);

  return {
    name: "@flairjs/vite-plugin",
    enforce: "pre",
    transform(code, id) {

      if (!shouldProcessFile(id, options?.include, options?.exclude)) {
        return null;
      }

      const result = transformCode(code, id, {
        appendTimestampToCssFile: true,
        classNameList: options?.classNameList,
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
