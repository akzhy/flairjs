import { ParseResult } from "@babel/parser";
import { createCacheCSSFile } from "./create-cache-dir";
import * as t from "@babel/types";
import { writeFileSync } from "fs";

export const handleCSS = ({
  ast,
  css,
  outputType,
  outputPath,
  filePath,
}: {
  ast: ParseResult<t.File>;
  css: string;
  outputType: "inject-import" | "write-css-file";
  outputPath: string;
  filePath: string;
}) => {
  if (outputType === "inject-import") {
    const { name: cacheFileName } = createCacheCSSFile({
      id: filePath,
      css,
    });

    ast.program.body.push(
      t.importDeclaration(
        [],
        t.stringLiteral(`jsx-styled-vite-plugin/cached-css/${cacheFileName}`)
      )
    );
  } else if (outputType === "write-css-file") {
    writeFileSync(outputPath, css);
  }
};
