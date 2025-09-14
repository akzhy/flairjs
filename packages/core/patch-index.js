const { readFileSync, writeFileSync } = require('fs')

const indexFileContent = readFileSync('./index.js', 'utf-8')

const replaced = indexFileContent.replace(
  `const { createRequire } = require('node:module')
require = createRequire(__filename)`,
  `import { createRequire } from 'module'
const require = createRequire(__filename)`,
)

writeFileSync('./index.js', replaced);