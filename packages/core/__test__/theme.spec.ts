import { test, expect, describe } from 'vitest'

import { transformCode } from '../index'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'url'
import { dirname } from 'path'
import path from 'node:path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const styleTagContent = readFileSync(path.resolve(__dirname, './snippets/style-tag-theme.tsx'), 'utf-8')
const flairPropertyContent = readFileSync(path.resolve(__dirname, './snippets/flair-string-theme.tsx'), 'utf-8')
const flairPropertyObjectContent = readFileSync(path.resolve(__dirname, './snippets/flair-obj-theme.tsx'), 'utf-8')

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

  test('flair string is working', () => {
    const result = transformCode(flairPropertyContent, 'theme-2.tsx', {
      cssOutDir: path.resolve(__dirname, './.css'),
      useTheme: true,
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
    expect(result.css).toMatchSnapshot()
  })

  test('flair object is working', () => {
    const result = transformCode(flairPropertyObjectContent, 'theme-3.tsx', {
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