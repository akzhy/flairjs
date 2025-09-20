import { transformCode as rustTransformCode, TransformOptions, TransformOutput } from './index'

interface ThemeObjItem {
  [key: string | number]: string | ThemeObjItem
}

export interface FlairThemeObject {
  colors?: Record<string | number, string | ThemeObjItem>
  space?: Record<string | number, string | ThemeObjItem>
  fontSizes?: Record<string | number, string | ThemeObjItem>
  fonts?: Record<string | number, string | ThemeObjItem>
  fontWeights?: Record<string | number, string | ThemeObjItem>
  lineHeights?: Record<string | number, string | ThemeObjItem>
  letterSpacings?: Record<string | number, string | ThemeObjItem>
  sizes?: Record<string | number, string | ThemeObjItem>
  borderWidths?: Record<string | number, string | ThemeObjItem>
  borderStyles?: Record<string | number, string | ThemeObjItem>
  radii?: Record<string | number, string | ThemeObjItem>
  shadows?: Record<string | number, string | ThemeObjItem>
  zIndices?: Record<string | number, string | ThemeObjItem>
  transitions?: Record<string | number, string | ThemeObjItem>
}

export type FlairThemeConfig = {
  tokens: FlairThemeObject
  breakpoints?: Record<string, string | number>
  prefix?: string
  selector: string | ((content: string, themeName?: string) => string)
  themes?: Record<
    string,
    {
      tokens: FlairThemeObject
      selector?: string | ((content: string, themeName?: string) => string)
    }
  >
}

export function defineConfig<T extends FlairThemeConfig>(config: T): T {
  return config
}

export const buildThemeTokens = (theme: FlairThemeConfig, themeName?: string) => {
  let css = ''
  const { tokens, selector } = theme

  if (typeof selector === 'string') {
    css += `${selector} {\n`
  }

  css += tokensToCSSVars(tokens, theme.prefix ? [theme.prefix] : [])

  if (typeof selector === 'string') {
    css += `}\n`
  } else {
    css = selector(css, themeName)
  }

  Object.entries(theme.themes ?? {}).forEach(([name, themeConfig]) => {
    css += buildThemeTokens({
      prefix: theme.prefix,
      selector: theme.selector,
      ...themeConfig,
    }, name)
  })

  return css
}

type Tokens = {
  [key: string]: string | number | Tokens
}

function tokensToCSSVars(tokens: Tokens | FlairThemeObject, prefix: string[] = []): string {
  let css = ''

  for (const [key, value] of Object.entries(tokens)) {
    const newPrefix = [...prefix, key]

    if (typeof value === 'string' || typeof value === 'number') {
      css += `--${newPrefix.join('-')}: ${value};\n`
    } else if (typeof value === 'object' && value !== null) {
      css += tokensToCSSVars(value, newPrefix)
    }
  }

  return css
}

const colors = {
  reset: '\x1b[0m',
  fg: {
    red: '\x1b[31m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    white: '\x1b[37m',
  },
  bg: {
    red: '\x1b[41m',
    yellow: '\x1b[43m',
    blue: '\x1b[44m',
  },
}

const logger = {
  error: (msg: string) => {
    console.log(`${colors.bg.red}${colors.fg.white}[flairjs/Error]${colors.reset} ${colors.fg.red}${msg}${colors.reset}`)
  },
  warn: (msg: string) => {
    console.log(`${colors.bg.yellow}${colors.fg.white}[flairjs/Warning]${colors.reset} ${msg}${colors.reset}`)
  },
  info: (msg: string) => {
    console.log(`${colors.bg.blue}${colors.fg.white}[flairjs/Info]${colors.reset} ${msg}${colors.reset}`)
  },
}

const transformCode = (
  code: string,
  filePath: string,
  options: TransformOptions & { cssPreprocessor?: (arg: string) => string | undefined | null },
): TransformOutput | null => {
  const result = rustTransformCode(
    code,
    filePath,
    {
      cssOutDir: options.cssOutDir,
      classNameList: options.classNameList,
      useTheme: options.useTheme,
      theme: options.theme,
    },
    options.cssPreprocessor,
  )

  const logs = result?.logs ?? []

  logs.forEach((log) => {
    if (logger[log.level]) {
      logger[log.level](log.message)
    }
  })
  return result
}

export { transformCode }
