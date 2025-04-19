import { transform } from "lightningcss";

export const transformCSS = (code: string, id: string) => {
  const { code: transformedCode, exports } = transform({
    cssModules: true,
    code: Buffer.from(code),
    filename: id,
    minify: true,
  });

  return { code: transformedCode.toString(), exports: exports || {} };
};
