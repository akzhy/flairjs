import {
  initializeSharedContext,
  SharedPluginOptions,
  shouldProcessFile,
  transformCode,
} from "@flairjs/bundler-shared";
import { LoaderContext } from "webpack";

interface FlairJsWebpackLoaderOptions extends SharedPluginOptions {}

export default async function flairJsLoader(
  this: LoaderContext<FlairJsWebpackLoaderOptions>,
  source: string,
  sourceMap: string
) {
  const callback = this.async();

  if (!callback) {
    console.error("@flairjs/webpack-loader requires async support");
    return;
  }

  const options = this.getOptions() || {};
  const context = await initializeSharedContext(options);
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
      theme: context.userTheme?.theme,
      useTheme: !!context.userTheme,
      cssOutDir: context.flairGeneratedCssDir,
    });

    if (!result) {
      return callback(null, source, sourceMap);
    }

    if (result.generatedCssName) {
      context.refreshCssFile(result.generatedCssName);
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
