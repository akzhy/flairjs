import {
  transformCode as rustTransformCode,
  TransformOptions,
  TransformOutput,
} from "@flairjs/core";
import { CssData } from "@flairjs/core";

const colors = {
  reset: "\x1b[0m",
  fg: {
    red: "\x1b[31m",
    yellow: "\x1b[33m",
    blue: "\x1b[34m",
    white: "\x1b[37m",
  },
  bg: {
    red: "\x1b[41m",
    yellow: "\x1b[43m",
    blue: "\x1b[44m",
  },
};

const logger = {
  error: (msg: string) => {
    console.log(
      `${colors.bg.red}${colors.fg.white}[flairjs/Error]${colors.reset} ${colors.fg.red}${msg}${colors.reset}`,
    );
  },
  warn: (msg: string) => {
    console.log(
      `${colors.bg.yellow}${colors.fg.white}[flairjs/Warning]${colors.reset} ${msg}${colors.reset}`,
    );
  },
  info: (msg: string) => {
    console.log(
      `${colors.bg.blue}${colors.fg.white}[flairjs/Info]${colors.reset} ${msg}${colors.reset}`,
    );
  },
};

export const transformCode = (
  code: string,
  filePath: string,
  options: TransformOptions & {
    cssPreprocessor?: (cssData: CssData) => string;
  },
): TransformOutput => {
  const result = rustTransformCode(
    code,
    filePath,
    {
      cssOutDir: options.cssOutDir,
      classNameList: options.classNameList,
      useTheme: options.useTheme,
      theme: options.theme,
      appendTimestampToCssFile: options.appendTimestampToCssFile,
    },
    options.cssPreprocessor,
  );

  const logs = result?.logs ?? [];

  logs.forEach((log) => {
    if (logger[log.level]) {
      logger[log.level](log.message);
    }
  });

  result.unusedClassnames.forEach((unused) => {
    logger.warn(
      `Unused classname detected: "${unused.className}" in file "${filePath}:${unused.line}:${unused.column}".`,
    );
  });

  return result;
};
