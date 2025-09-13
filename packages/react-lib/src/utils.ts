import type * as CSS from "csstype";

type FlairObject = {
  [K in keyof CSS.Properties]?: CSS.Properties[K] | (string & {}) | number;
};

interface FlairCSS {
  [key: string]: FlairObject | FlairCSS;
}

export const flair = (styles: FlairCSS) => '';

const c = (className: string) => {
  return className;
};

const cn = c;

export { c, cn };
