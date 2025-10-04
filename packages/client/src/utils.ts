import type * as CSS from "csstype";
export interface FlairTheme {}

type Key = string | number;
type Join<P extends string, K extends Key> = `${P}${P extends ""
  ? ""
  : "."}${K}`;

type Paths<T, P extends string = ""> = T extends object
  ? { [K in keyof T & Key]: Paths<T[K], Join<P, K>> }[keyof T & Key]
  : P;

export type TokensOf<T> = `$${Paths<T>}`;
export type ThemeTokens<T extends FlairTheme = FlairTheme> = T extends { tokens: any }
  ? TokensOf<T["tokens"]>
  : never;

export type BreakPointTokens<T extends FlairTheme = FlairTheme> = T extends { breakpoints: any }
  ? `$screen ${Extract<keyof T["breakpoints"], string>}`
  : never;

type FlairObject<T extends FlairTheme = FlairTheme> = {
  [K in keyof CSS.Properties]?:
    | CSS.Properties[K]
    | ThemeTokens<T>
    | (string & {})
    | number;
};

type FlairCSS = {
  [K in string]?: FlairObject | FlairCSS;
} & {
  [K in BreakPointTokens]?: FlairObject | FlairCSS;
};

export const flair = (styles: FlairCSS) => "";

const c = (className: string) => {
  return className;
};

const cn = c;

const css = String.raw;

export { c, cn, css };
