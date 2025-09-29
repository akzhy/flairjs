import {
  getUserTheme,
  initializeSharedContext,
  SharedPluginOptions,
  shouldProcessFile,
  transformCode,
} from "@flairjs/bundler-shared";
import type { Plugin } from "rollup";

export type { SharedPluginOptions };

interface FlairJsRollupPluginOptions extends SharedPluginOptions {}

export default async function flairJsRollupPlugin(
  options?: FlairJsRollupPluginOptions
): Promise<Plugin> {
  const context = await initializeSharedContext(options);

  return {
    name: "@flairjs/rollup-plugin",
    buildStart() {
      if (context?.userTheme) {
        const themeFile = context.userTheme.originalPath;
        this.addWatchFile(themeFile);
      }
    },
    resolveId(source) {
      if (source === "@flairjs/client/theme.css") {
        return source;
      }
      return null;
    },
    async load(id) {
      if (id !== "@flairjs/client/theme.css") {
        return null;
      }
      if (context?.userTheme?.originalPath) {
        this.addWatchFile(context.userTheme.originalPath);
      }
      let userTheme = await getUserTheme({ ignoreCache: true });
      const buildThemeCSS = context?.buildThemeCSS;
      if (userTheme && buildThemeCSS) {
        const themeCSS = buildThemeCSS(userTheme.theme);
        return themeCSS;
      }
      return "";
    },
    transform(code, id) {
      if (!shouldProcessFile(id, options?.include, options?.exclude)) {
        return null;
      }

      if (!context) {
        return null;
      }

      const result = transformCode(code, id, {
        appendTimestampToCssFile: true,
        cssPreprocessor: options?.cssPreprocessor
          ? (css: string) => options.cssPreprocessor!(css, id)
          : undefined,
        theme: context.userTheme?.theme,
        useTheme: !!context.userTheme,
        cssOutDir: context.flairGeneratedCssDir,
        classNameList: options?.classNameList,
      });

      if (!result) {
        return null;
      }

      if (result.generatedCssName) {
        context.refreshCssFile(id, result.generatedCssName);
      }

      return {
        code: result.code,
        map: JSON.parse(result.sourcemap ?? "{}"),
      };
    },
  };
}
