import { test, expect, describe } from 'vitest'

import { transformCode } from '../index'
import { readdirSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'url'
import { dirname } from 'path'
import path from 'node:path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const styleTagContent = readFileSync(path.resolve(__dirname, './snippets/style-tag-theme.tsx'), 'utf-8')
const flairPropertyContent = readFileSync(path.resolve(__dirname, './snippets/flair-string.tsx'), 'utf-8')
const flairPropertyObjectContent = readFileSync(path.resolve(__dirname, './snippets/flair-obj.tsx'), 'utf-8')

describe('Theme tests', () => {
  test('style tag is working', () => {
    const result = transformCode(styleTagContent, 'theme-1.tsx', {
      cssOutDir: path.resolve(__dirname, './.css'),
      useTheme: true,
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
    expect(result.css).toMatchSnapshot()
  })
});