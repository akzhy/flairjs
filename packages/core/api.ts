import { transformCode } from './index'

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
  selector: string | ((content: string, themeName?: string) => string)
  themes?: Record<
    string,
    {
      tokens: FlairThemeObject
      selector: string | ((content: string, themeName?: string) => string)
    }
  >
}

export function defineConfig<T extends FlairThemeConfig>(config: T): T {
  return config
}

export const buildThemeTokens = (theme: FlairThemeConfig) => {
  let css = ''
  const { tokens, selector } = theme

  if (typeof selector === 'string') {
    css += `${selector} {\n`
  }

  css += tokensToCSSVars(tokens)

  if (typeof selector === 'string') {
    css += `}\n`
  } else {
    css = selector(css)
  }

  Object.entries(theme.themes ?? {}).forEach(([themeName, themeConfig]) => {
    if (typeof themeConfig.selector === 'string') {
      css += `${themeConfig.selector} {\n`
    }

    css += buildThemeTokens(themeConfig)

    if (typeof themeConfig.selector === 'string') {
      css += `}\n`
    } else {
      css = themeConfig.selector(css, themeName)
    }
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


export { transformCode }