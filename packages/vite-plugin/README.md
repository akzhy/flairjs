# @flairjs/vite-plugin

Vite plugin for Flair CSS-in-JSX.

## Installation

```bash
npm install @flairjs/vite-plugin
```

## Usage

```js
// vite.config.js
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import flairjs from '@flairjs/vite-plugin'

export default defineConfig({
  plugins: [
    react(),
    flairjs({
      // Optional configuration
    })
  ]
})
```

## Configuration

```typescript
interface FlairJsVitePluginOptions {
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
// vite.config.js
import flairjs from '@flairjs/vite-plugin'

export default {
  plugins: [
    flairjs({
      // Only process files in src directory
      include: ['src/**/*.{tsx,jsx}'],
      
      // Exclude test files
      exclude: ['**/*.test.{tsx,jsx}', '**/*.spec.{tsx,jsx}'],
      
      // Specify known class names for optimization
      classNameList: ["className", "class"]
    })
  ]
}
```

## Features

- **Hot Module Replacement** - Styles update instantly during development
- **Theme File Watching** - Automatically rebuilds when `flair.theme.ts` changes
- **CSS Import Resolution** - Handles `@flairjs/client/theme.css` imports
- **Development Optimization** - Fast rebuilds with intelligent caching


## Theme Integration

The plugin automatically detects and processes your `flair.theme.ts` file:

```typescript
// flair.theme.ts
import { defineConfig } from '@flairjs/client'

export default defineConfig({
  tokens: {
    colors: {
      primary: '#3b82f6'
    }
  }
})
```

The theme CSS is available via:

```js
import '@flairjs/client/theme.css'
```


## License

MIT
