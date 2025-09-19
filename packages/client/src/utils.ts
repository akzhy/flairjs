import type * as CSS from "csstype";

export interface FlairTheme {}

type Key = string | number;
type Join<P extends string, K extends Key> =
  `${P}${P extends "" ? "" : "."}${K}`;

type Paths<T, P extends string = ""> =
  T extends object
    ? { [K in keyof T & Key]: Paths<T[K], Join<P, K>> }[keyof T & Key]
    : P;

export type TokensOf<T> = `$${Paths<T>}`;
export type Tokens = TokensOf<FlairTheme>;

type FlairObject = {
  [K in keyof CSS.Properties]?:
    | CSS.Properties[K]
    | Tokens
    | (string & {})
    | number;
};

type FlairCSS = {
  [key: string]: FlairObject | FlairCSS;
} & FlairObject;

export const flair = (styles: FlairCSS) => "";

const c = (className: string) => {
  return className;
};

const cn = c;

export { c, cn };
