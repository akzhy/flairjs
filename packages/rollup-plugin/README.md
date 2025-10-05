# @flairjs/rollup-plugin

Rollup plugin for Flair CSS-in-JSX.

## Installation

```bash
npm install @flairjs/rollup-plugin
```

## Usage

```js
// rollup.config.js
import flairjs from '@flairjs/rollup-plugin'
import { nodeResolve } from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import babel from '@rollup/plugin-babel'

export default {
  input: 'src/index.js',
  output: {
    file: 'dist/bundle.js',
    format: 'esm'
  },
  plugins: [
    flairjs(), // Add before babel
    babel({
      babelHelpers: 'bundled',
      presets: ['@babel/preset-react']
    }),
    nodeResolve(),
    commonjs()
  ]
}
```

## Configuration

```typescript
interface FlairJsRollupPluginOptions {
  /**
   * Preprocess the extracted CSS before it is passed to lightningcss
   * @experimental
   */
  cssPreprocessor?: (css: string, id: string) => string
  
  /**
   * Files to include (default: all .tsx/.jsx files)
   */
  include?: string | string[]
  
  /**
   * Files to exclude (default: node_modules)
   */
  exclude?: string | string[]
  
  /**
   * Override the default theme file content
   */
  buildThemeFile?: (theme: FlairThemeConfig) => string
  
  /**
   * List of class names used in the project. Supports regex.
   */
  classNameList?: string[]
}
```

### Example with Options

```js
// rollup.config.js
import flairjs from '@flairjs/rollup-plugin'

export default {
  plugins: [
    flairjs({
      // Only process source files
      include: ['src/**/*.{tsx,jsx}'],
      
      // Exclude test files
      exclude: ['**/*.test.{tsx,jsx}'],
      
    })
  ]
}
```

## Plugin Order

Ensure the Flair plugin runs before JSX transformation:

```js
export default {
  plugins: [
    flairjs(),     // 1. Process Flair styles first
    babel({        // 2. Then transform JSX
      presets: ['@babel/preset-react']
    }),
    nodeResolve(), // 3. Resolve imports
    commonjs()     // 4. Handle CommonJS
  ]
}
```

## CSS Handling

The plugin generates CSS files that need to be handled by your build process:

```js
// rollup.config.js
import css from 'rollup-plugin-css-only'

export default {
  plugins: [
    flairjs(),
    css({ output: 'dist/styles.css' }), // Handle generated CSS
    // other plugins...
  ]
}
```

## Theme Integration

Create a `flair.theme.ts` in your project root:

```typescript
// flair.theme.ts
import { defineConfig } from '@flairjs/client'

export default defineConfig({
  tokens: {
    colors: {
      primary: '#3b82f6',
      secondary: '#64748b'
    }
  }
})
```

Import the theme CSS in your application:

```js
// src/index.js
import '@flairjs/client/theme.css'
// your app code...
```

## License

MIT
