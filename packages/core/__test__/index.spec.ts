import { test, expect } from 'vitest'

import { transformCode } from '../index'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'url'
import { dirname } from 'path'
import path from 'node:path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const styleTagContent = readFileSync(path.resolve(__dirname, './utils/style-tag.tsx'), 'utf-8')
const flairPropertyContent = readFileSync(path.resolve(__dirname, './utils/flair-string.tsx'), 'utf-8')

test('style Tag is working', () => {
  const result = transformCode({
    code: styleTagContent,
    filePath: 'index.tsx',
    cssOutDir: path.resolve(__dirname, './.css'),
  })
  if (!result) {
    throw new Error('transformCode returned null or undefined')
  }
  expect(result.code).toMatchSnapshot()
})

test('flair property string is working', () => {
  const result = transformCode({
    code: flairPropertyContent,
    filePath: 'index.tsx',
    cssOutDir: path.resolve(__dirname, './.css'),
  })
  if (!result) {
    throw new Error('transformCode returned null or undefined')
  }
  expect(result.code).toMatchSnapshot()
});