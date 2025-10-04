import { defineConfig, RolldownOptions } from 'rolldown';
import typescript from '@rollup/plugin-typescript';

const createOptions = (format: 'esm' | 'cjs', entry: string, outDir: string): RolldownOptions => {
  return {
    input: entry,
    platform: 'node',
    output: {
      dir: outDir,
      format: format,
      esModule: true,
    },
    external: (id) => {
      return id.endsWith('.node') || id.includes('node_modules');
    },
  };
};

export default defineConfig([
  // Main entry points
  createOptions('esm', 'src/index.ts', 'dist/esm'),
  createOptions('cjs', 'src/index.ts', 'dist/cjs'),
  
  // Framework-specific entry points
  createOptions('esm', 'src/react/index.ts', 'dist/esm/react'),
  createOptions('cjs', 'src/react/index.ts', 'dist/cjs/react'),
  createOptions('esm', 'src/preact/index.ts', 'dist/esm/preact'),
  createOptions('cjs', 'src/preact/index.ts', 'dist/cjs/preact'),
  createOptions('esm', 'src/solidjs/index.ts', 'dist/esm/solidjs'),
  createOptions('cjs', 'src/solidjs/index.ts', 'dist/cjs/solidjs'),
  
  // Type definitions - single config for all types
  {
    input: {
      'index': 'src/index.ts',
      'react/index': 'src/react/index.ts',
      'preact/index': 'src/preact/index.ts',
      'solidjs/index': 'src/solidjs/index.ts',
    },
    output: {
      dir: 'dist/types',
      preserveModules: true,
    },
    plugins: [
      typescript({ 
        tsconfig: './tsconfig.json',
        declaration: true,
        emitDeclarationOnly: true,
        outDir: 'dist/types',
      }),
      {
        name: 'remove-js-files',
        generateBundle(options, bundle) {
          for (const file of Object.keys(bundle)) {
            if (file.endsWith('.js') || file.endsWith('.js.map')) {
              delete bundle[file];
            }
          }
        }
      }
    ],
  },
]);