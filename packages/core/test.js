const  { transformCode } = require("./index.js")
const { readFileSync } = require("fs");

const appCode = readFileSync("./src/App.tsx", "utf-8");

console.time("Transforming code");
const transformedCode = transformCode({
  code: appCode,
  filePath: "./src/App.tsx",
  cssOutDir: "./out"
});
// console.log(transformedCode.sourcemap)
// console.log("Transforming code \n", transformedCode);
