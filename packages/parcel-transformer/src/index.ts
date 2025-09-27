import {
  removeOutdatedCssFiles,
  setupGeneratedCssDir,
  setupUserThemeFile,
  SharedPluginOptions,
  shouldProcessFile,
  transformCode,
} from "@flairjs/bundler-shared";
import { Transformer } from "@parcel/plugin";
import SourceMapImport from "@parcel/source-map";

const SourceMap = (SourceMapImport as any).default ?? SourceMapImport;

interface FlairJsParcelTransformerOptions extends SharedPluginOptions {}

export default new Transformer({
  async loadConfig({ config }) {
    const getConfigResult = await config.getConfig(["tool.config.js"]);

    const filePath = getConfigResult?.filePath;
    const configContents = getConfigResult?.contents as
      | FlairJsParcelTransformerOptions
      | undefined;

    if (filePath?.endsWith(".js")) {
      config.invalidateOnStartup();
    }

    const cssGeneratedDir = await setupGeneratedCssDir();
    const userTheme = await setupUserThemeFile({
      buildThemeFile: configContents?.buildThemeFile,
    });

    return { ...configContents, cssGeneratedDir, userTheme };
  },
  async transform({ asset, config, logger, options }) {
    const filePath = asset.filePath;
    const cssOutDir = config?.cssGeneratedDir ?? null;

    if (!shouldProcessFile(filePath, config.include, config.exclude)) {
      return [asset];
    }

    if (!cssOutDir) {
      return [asset];
    }

    try {
      const code = await asset.getCode();

      const result = transformCode(code, asset.filePath, {
        appendTimestampToCssFile: true,
        classNameList: config?.classNameList,
        cssPreprocessor: config?.cssPreprocessor
          ? (css: string) => config.cssPreprocessor!(css, asset.filePath)
          : undefined,
        theme: config.userTheme?.theme,
        useTheme: !!config.userTheme,
        cssOutDir,
      });

      if (!result) {
        return [asset];
      }

      if (result.generatedCssName) {
        removeOutdatedCssFiles(asset.filePath, result.generatedCssName, {
          flairGeneratedCssDir: cssOutDir,
        });
      }

      asset.setCode(result.code);
      if (result.sourcemap) {
        const sourcemap = new SourceMap(options.projectRoot);
        sourcemap.addVLQMap(JSON.parse(result.sourcemap));
        asset.setMap(sourcemap);
      }

      return [asset];
    } catch (error) {
      logger.error({
        message: `Error during flairjs transformation: ${
          error instanceof Error ? error.message : String(error)
        }`,
        origin: "@flairjs/parcel-transformer",
      });

      // Return the original asset if transformation fails
      return [asset];
    }
  },
});
