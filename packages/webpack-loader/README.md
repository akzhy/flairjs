# @flairjs/webpack-loader

Webpack loader for Flair CSS-in-JSX.

## Installation

```bash
npm install @flairjs/webpack-loader
```

## Usage

```js
// webpack.config.js
module.exports = {
  module: {
    rules: [
      {
        test: /\.(tsx|jsx)$/,
        use: [
          '@flairjs/webpack-loader'
        ],
        exclude: /node_modules/
      }
    ]
  }
}
```

## Configuration

```js
// webpack.config.js
module.exports = {
  module: {
    rules: [
      {
        test: /\.(tsx|jsx)$/,
        use: [
          'babel-loader',
          {
            loader: '@flairjs/webpack-loader',
            options: {
              // Configuration options
              cssPreprocessor: (css, id) => {
                // Custom CSS preprocessing
                return css
              },
              include: ['src/**/*.{tsx,jsx}'],
              exclude: ['**/*.test.{tsx,jsx}'],
              classNameList: ["className", "class"]
            }
          },
        ]
      }
    ]
  }
}
```

### Configuration Options

```typescript
interface FlairJsWebpackLoaderOptions {
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

## CSS Handling

Ensure your webpack configuration can handle CSS files:

```js
// webpack.config.js
module.exports = {
  module: {
    rules: [
      // Flair loader
      {
        test: /\.(tsx|jsx)$/,
        use: ['@flairjs/webpack-loader']
      },
      // CSS loader for generated styles
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader']
      }
    ]
  }
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

Import the theme in your entry file:

```js
// src/index.js
import '@flairjs/client/theme.css'
import './App'
```

## License

MIT
