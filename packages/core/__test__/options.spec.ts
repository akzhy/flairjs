import { describe, expect, test } from 'vitest'

import { readFileSync } from 'node:fs'
import path from 'node:path'
import { dirname } from 'path'
import { fileURLToPath } from 'url'
import { transformCode } from '../index'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const classNameListContent = readFileSync(path.resolve(__dirname, './snippets/options-class-name-list.tsx'), 'utf-8')

describe('Options tests', () => {
  test('class name list is working', () => {
    const regex = new RegExp("^class[A-Z][A-Za-z0-9_]*");
    const result = transformCode(classNameListContent, 'options-1.tsx', {
      classNameList: ['className', 'containerClassName', regex.toString()],
      cssOutDir: path.resolve(__dirname, './.css'),
    })
    if (!result) {
      throw new Error('transformCode returned null or undefined')
    }
    expect(result.code).toMatchSnapshot()
  })
})
