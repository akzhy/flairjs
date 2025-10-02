import {
  getGeneratedCssDir,
  getUserTheme,
  setupGeneratedCssDir,
  setupUserThemeFile,
  SharedPluginOptions,
  shouldProcessFile,
  transformCode
} from "@flairjs/bundler-shared";
import * as path from "path";
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

  const fileName = this.resourcePath;

  if (!shouldProcessFile(fileName, options?.include, options?.exclude)) {
    return callback(null, source, sourceMap);
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
      deleteBeforeWrite: true,
    });
    initialized = true;
  } else {
    cssGeneratedDir = getGeneratedCssDir();
    userTheme = await getUserTheme();
  }

  if (!cssGeneratedDir) {
    console.error(
      "[flairjs] Could not find generated CSS directory. Skipping processing."
    );
    return callback(null, source, sourceMap);
  }

  try {
    const result = transformCode(source, fileName, {
      appendTimestampToCssFile: false,
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

    if (!result.generatedCssName) {
      return callback(null, source, sourceMap);
    }

    this.addDependency(path.resolve(result.generatedCssName));

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
