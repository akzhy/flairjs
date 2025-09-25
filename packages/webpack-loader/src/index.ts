import {
  getGeneratedCssDir,
  getUserTheme,
  initializeSharedContext,
  removeOutdatedCssFiles,
  setupGeneratedCssDir,
  setupUserThemeFile,
  SharedPluginOptions,
  shouldProcessFile,
  transformCode,
} from "@flairjs/bundler-shared";
import { LoaderContext } from "webpack";

interface FlairJsWebpackLoaderOptions extends SharedPluginOptions {}

let initialized = false;

export default async function flairJsLoader(
  this: LoaderContext<FlairJsWebpackLoaderOptions>,
  source: string,
  sourceMap: string
) {
  const callback = this.async();
  const options = this.getOptions() || {};

  if (!callback) {
    console.error("@flairjs/webpack-loader requires async support");
    return;
  }

  let cssGeneratedDir: string | null = null;
  let userTheme: {
    theme: any;
    originalPath: string;
  } | null = null;

  if (!initialized) {
    cssGeneratedDir = await setupGeneratedCssDir();
    userTheme = await setupUserThemeFile({
      buildThemeFile: options.buildThemeFile,
    });
    initialized = true;
  } else {
    cssGeneratedDir = getGeneratedCssDir();
    userTheme = await getUserTheme();
  }

  const fileName = this.resourcePath;

  if (!shouldProcessFile(fileName, options?.include, options?.exclude)) {
    return callback(null, source, sourceMap);
  }

  try {
    const result = transformCode(source, fileName, {
      appendTimestampToCssFile: true,
      classNameList: options?.classNameList,
      cssPreprocessor: options?.cssPreprocessor
        ? (css: string) => options.cssPreprocessor!(css, fileName)
        : undefined,
      theme: userTheme?.theme,
      useTheme: !!userTheme,
      cssOutDir: cssGeneratedDir,
    });

    if (!result) {
      return callback(null, source, sourceMap);
    }

    if (result.generatedCssName) {
      removeOutdatedCssFiles(fileName, result.generatedCssName, {
        flairGeneratedCssDir: cssGeneratedDir,
      });
    }

    callback(
      null,
      result.code,
      result.sourcemap ? JSON.parse(result.sourcemap ?? "{}") : sourceMap
    );
  } catch (error) {
    console.error("[@flairjs/webpack-loader]", error);
    callback(error as Error, source, sourceMap);
  }
}
