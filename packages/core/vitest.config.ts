import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    include: ['__test__/**/*.spec.ts'],
    setupFiles: ['__test__/setup.ts'],
    env: {
      OXC_TSCONFIG_PATH: './__test__/tsconfig.json'
    }
  },
  esbuild: {
    loader: 'ts',
    target: 'node18'
  }
})
