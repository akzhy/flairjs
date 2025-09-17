const { readFileSync, writeFileSync } = require('fs')

const indexFileContent = readFileSync('./index.js', 'utf-8')

let replaced = indexFileContent.replace(
  `const { createRequire } = require('node:module')
require = createRequire(__filename)`,
  `import { createRequire } from 'module'
const require = createRequire(import.meta.url);`,
)

replaced = replaced.replace(
  `module.exports = nativeBinding
module.exports.transformCode = nativeBinding.transformCode`,

  `export default nativeBinding;
export const transformCode = nativeBinding.transformCode;`,
)

writeFileSync('./index.js', replaced)
