import { test, expect, describe } from 'vitest'

import { transformCode } from '../index'
import { readdirSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'url'
import { dirname } from 'path'
import path from 'node:path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const styleTagContent = readFileSync(path.resolve(__dirname, './snippets/style-tag.tsx'), 'utf-8')
const flairPropertyContent = readFileSync(path.resolve(__dirname, './snippets/flair-string.tsx'), 'utf-8')
const flairPropertyObjectContent = readFileSync(path.resolve(__dirname, './snippets/flair-obj.tsx'), 'utf-8')

describe('Core tests', () => {
  test('style tag is working', () => {
    const result = transformCode(styleTagContent, 'index.tsx', {
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })

  test('flair property string is working', () => {
    const result = transformCode(flairPropertyContent, 'index.tsx', {
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })

  test('flair property object is working', () => {
    const result = transformCode(flairPropertyObjectContent, 'index.tsx', {
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    const cssFiles = readdirSync(path.resolve(__dirname, './.css')).filter((f) => f.endsWith('.css'))
    const cssContent = readFileSync(path.resolve(__dirname, './.css', cssFiles[0]), 'utf-8')


    expect(cssContent).toMatchSnapshot()

    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })
})
