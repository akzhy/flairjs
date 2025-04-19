import { generate } from "@babel/generator";
import { parse } from "@babel/parser";
import traverse from "@babel/traverse";
import * as t from "@babel/types";
import { createCacheCSSFile, createCacheDir } from "./create-cache-dir";
import { transformCSS } from "./transform-css";
import { extractCSS } from "./extract-css";
import { processAttribute } from "./process-attribute";

createCacheDir();

export const transform = (code: string, id: string) => {
  if (!id.endsWith(".tsx")) {
    return null;
  }

  const ast = parse(code, {
    sourceType: "module",
    plugins: ["typescript", "jsx"],
  });

  traverse(ast, {
    JSXElement(path) {
      if (path.node.openingElement.name.type !== "JSXIdentifier") {
        return;
      }

      const name = path.node.openingElement.name.name;
      if (name !== "Style") {
        return;
      }

      const parentFunction = path.findParent(
        (p) =>
          p.isArrowFunctionExpression() ||
          p.isClassDeclaration() ||
          p.isFunctionDeclaration()
      );

      if (!parentFunction) {
        return;
      }

      const styleBody = extractCSS(path.node);

      const { code: transformedCSS, exports } = transformCSS(styleBody, id);

      const { name: cacheFileName } = createCacheCSSFile({
        id,
        css: transformedCSS,
      });

      parentFunction.traverse({
        JSXAttribute(path) {
          if (path.node.name.name === "className") {
            processAttribute({
              node: path.node,
              attrName: "className",
              classNameMap: exports,
            });
          }
        },
      });

      ast.program.body.push(
        t.importDeclaration(
          [],
          t.stringLiteral(`jsx-styled-vite-plugin/cached-css/${cacheFileName}`)
        )
      );

      path.remove();
    },
  });

  return generate(ast).code;
};
