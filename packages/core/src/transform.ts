import { generate } from "@babel/generator";
import { parse } from "@babel/parser";
import traverse from "@babel/traverse";
import * as t from "@babel/types";
import {
  CLASSNAME_ATTRIBUTES,
  CLASSNAME_UTIL_FUNCTIONS,
  CLIENT_PACKAGE_NAMES,
  STYLE_TAG_NAME,
} from "./constants";
import { createCacheCSSFile, createCacheDir } from "./create-cache-dir";
import { extractCSS } from "./extract-css";
import {
  AttributeProcessor,
} from "./process-attribute";
import { transformCSS } from "./transform-css";

createCacheDir();

export interface TransformOptions {
  code: string;
  filePath: string;
  cssPreprocessor?: (css: string, id: string) => string;
}

export const transform = async ({
  code,
  filePath,
  cssPreprocessor,
}: TransformOptions) => {
  if (!filePath.endsWith(".tsx")) {
    return null;
  }

  const ast = parse(code, {
    sourceType: "module",
    plugins: ["typescript", "jsx"],
  });

  let localStyleTagName = STYLE_TAG_NAME;
  let localClassNameUtilFunctions: string[] = [];

  traverse(ast, {
    ImportDeclaration(path) {
      if (CLIENT_PACKAGE_NAMES.includes(path.node.source.value)) {
        for (const specifier of path.node.specifiers) {
          if (specifier.type === "ImportSpecifier") {
            let importedName =
              specifier.imported.type === "Identifier"
                ? specifier.imported.name
                : specifier.imported.value;

            if (importedName === STYLE_TAG_NAME) {
              localStyleTagName = specifier.local.name;
            } else if (CLASSNAME_UTIL_FUNCTIONS.includes(importedName)) {
              localClassNameUtilFunctions.push(specifier.local.name);
            }
          }
        }
      }
    },
    JSXElement(path) {
      if (path.node.openingElement.name.type !== "JSXIdentifier") {
        return;
      }

      const name = path.node.openingElement.name.name;
      if (name !== localStyleTagName) {
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

      let processedCSS = styleBody;
      if (cssPreprocessor) {
        processedCSS = cssPreprocessor(styleBody, filePath);
      }

      const { code: transformedCSS, exports: classNameMap } = transformCSS(
        processedCSS,
        filePath
      );

      const { name: cacheFileName } = createCacheCSSFile({
        id: filePath,
        css: transformedCSS,
      });

      parentFunction.traverse({
        JSXAttribute(path) {
          if (
            typeof path.node.name.name === "string" &&
            CLASSNAME_ATTRIBUTES.includes(path.node.name.name)
          ) {
            new AttributeProcessor({
              path,
              attrName: path.node.name.name,
              classNameMap,
            }).updateAttribute();;
          }
        },
        CallExpression(path) {
          if (
            t.isIdentifier(path.node.callee) &&
            localClassNameUtilFunctions.includes(path.node.callee.name)
          ) {
            new AttributeProcessor({
              path,
              attrName: path.node.callee.name,
              classNameMap,
            }).updateCallExpression(path.node);
          }
        },
      });

      ast.program.body.push(
        t.importDeclaration(
          [],
          t.stringLiteral(`jsx-styled-vite-plugin/cached-css/${cacheFileName}`)
        )
      );

      const nodeEnv = process.env.NODE_ENV;
      if (nodeEnv === "production") {
        path.remove();
      }
    },
  });

  return generate(ast).code;
};
