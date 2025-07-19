const  { transformCode } = require("./index.js")
const { readFileSync } = require("fs");

const appCode = readFileSync("./src/App.tsx", "utf-8");

console.time("Transforming code");
const transformedCode = transformCode(appCode, "./src/App.tsx");
// console.log(transformedCode.sourcemap)
console.timeEnd("Transforming code");
