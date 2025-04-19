import { defineConfig } from '@rslib/core';

export default defineConfig({
  lib: [
    {
      format: 'esm',
      syntax: 'es2021',
      dts: true,
      output: {
        externals: ['lightningcss'],
      }
    },
    {
      format: 'cjs',
      syntax: 'es2021',
      autoExternal: true,
      output: {
        externals: ['lightningcss'],
      }
    },
  ],
});
