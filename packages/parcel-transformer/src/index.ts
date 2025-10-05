import {
  getUserTheme,
  setupGeneratedCssDir,
  setupUserThemeFile,
  type SharedPluginOptions,
  shouldProcessFile,
  transformCode,
} from "@flairjs/bundler-shared";
import type { Transformer as TransformerType } from "@parcel/plugin";
import { Transformer } from "@parcel/plugin";
import SourceMapImport from "@parcel/source-map";
import * as path from "node:path";

const SourceMap = (SourceMapImport as any).default ?? SourceMapImport;

export interface FlairJsParcelTransformerOptions extends SharedPluginOptions {}
let initialized = false;

const transformer: TransformerType<FlairJsParcelTransformerOptions> =
  new Transformer({
    async loadConfig({ config }) {
      const getConfigResult = await config.getConfig(["flair.config.js"]);

      const filePath = getConfigResult?.filePath;
      const configContents = getConfigResult?.contents as
        | FlairJsParcelTransformerOptions
        | undefined;

      if (filePath?.endsWith(".js")) {
        config.invalidateOnStartup();
      }

      const cssGeneratedDir = await setupGeneratedCssDir({
        clearExisting: false,
      });

      const userTheme = initialized
        ? await getUserTheme()
        : await setupUserThemeFile({
            buildThemeFile: configContents?.buildThemeFile,
          });

      if (!initialized) {
        initialized = true;
      }

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
          appendTimestampToCssFile: false,
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

        if (!result.generatedCssName) {
          return [asset];
        }

        asset.addURLDependency(
          path.resolve(cssOutDir, result.generatedCssName),
          {
            pipeline: "css",
          }
        );

        asset.setCode(result.code);
        if (result.sourcemap) {
          try {
            const sourcemap = new SourceMap(options.projectRoot);
            sourcemap.addVLQMap(JSON.parse(result.sourcemap));
            asset.setMap(sourcemap);
          } catch {}
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

export default transformer;
