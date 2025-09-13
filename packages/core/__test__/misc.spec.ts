import { describe, expect, test } from 'vitest'

import { readFileSync } from 'node:fs'
import path from 'node:path'
import { dirname } from 'path'
import { fileURLToPath } from 'url'
import { transformCode } from '../index'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const conflictingClassNamesContent = readFileSync(
  path.resolve(__dirname, './snippets/misc-conflicting-classnames.tsx'),
  'utf-8',
)
const combinedStylesContent = readFileSync(path.resolve(__dirname, './snippets/misc-combined-styles.tsx'), 'utf-8')
const globalStyleContent = readFileSync(path.resolve(__dirname, './snippets/misc-global-style.tsx'), 'utf-8')
const globalFlairStyleContent = readFileSync(path.resolve(__dirname, './snippets/misc-global-flair.tsx'), 'utf-8')
const functionVariants = readFileSync(path.resolve(__dirname, './snippets/misc-function-variants.tsx'), 'utf-8')

describe('Misc tests', () => {
  test('conflicting classnames in same file are working', () => {
    const regex = new RegExp('^class[A-Z][A-Za-z0-9_]*')
    const result = transformCode(conflictingClassNamesContent, 'misc-1.tsx', {
      classNameList: ['className', 'containerClassName', regex.toString()],
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })

  test('combined styles are working', () => {
    const regex = new RegExp('^class[A-Z][A-Za-z0-9_]*')
    const result = transformCode(combinedStylesContent, 'misc-2.tsx', {
      classNameList: ['className', 'containerClassName', regex.toString()],
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })

  test('global styles are working', () => {
    const result = transformCode(globalStyleContent, 'misc-3.tsx', {
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })

  test('global flair styles are working', () => {
    const result = transformCode(globalFlairStyleContent, 'misc-4.tsx', {
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })

  test('different function variants are working', () => {
    const result = transformCode(functionVariants, 'misc-5.tsx', {
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })
})
