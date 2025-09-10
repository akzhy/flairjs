import { describe, expect, test } from 'vitest'

import { readFileSync } from 'node:fs'
import path from 'node:path'
import { dirname } from 'path'
import { fileURLToPath } from 'url'
import { transformCode } from '../index'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const conflictingClassNamesContent = readFileSync(path.resolve(__dirname, './snippets/misc-conflicting-classnames.tsx'), 'utf-8')
const combinedStylesContent = readFileSync(path.resolve(__dirname, './snippets/misc-combined-styles.tsx'), 'utf-8')

describe('Misc tests', () => {
  test('conflicting classnames in same file are working', () => {
    const regex = new RegExp("^class[A-Z][A-Za-z0-9_]*");
    const result = transformCode(conflictingClassNamesContent, 'index.tsx', {
      classNameList: ['className', 'containerClassName', regex.toString()],
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })
  test('combined styles are working', () => {
    const regex = new RegExp("^class[A-Z][A-Za-z0-9_]*");
    const result = transformCode(combinedStylesContent, 'index.tsx', {
      classNameList: ['className', 'containerClassName', regex.toString()],
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })
})
