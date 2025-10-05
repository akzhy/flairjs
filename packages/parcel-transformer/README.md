# @flairjs/parcel-transformer

Parcel transformer for Flair CSS-in-JSX.

## Installation

```bash
npm install @flairjs/parcel-transformer
```

## Usage

Add the transformer to your `.parcelrc` file:

```json
{
  "extends": "@parcel/config-default",
  "transformers": {
    "*.{tsx,jsx}": ["@flairjs/parcel-transformer", "..."]
  }
}
```

## Configuration

You can configure the transformer by adding options to your `package.json`:

```json
{
  "@flairjs/parcel-transformer": {
    "include": ["src/**/*.{tsx,jsx}"],
    "exclude": ["**/*.test.{tsx,jsx}"],
    "classNameList": ["className", "class"]
  }
}
```

### Configuration Options

```typescript
interface FlairJsParcelTransformerOptions {
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

## Complete Parcel Setup

### 1. Install Dependencies

```bash
npm install parcel @flairjs/client @flairjs/core @flairjs/parcel-transformer
```

### 2. Create .parcelrc

```json
{
  "extends": "@parcel/config-default",
  "transformers": {
    "*.{tsx,jsx}": ["@flairjs/parcel-transformer", "..."]
  }
}
```

### 3. Package.json Scripts

```json
{
  "scripts": {
    "dev": "parcel src/index.html",
    "build": "parcel build src/index.html"
  }
}
```

### 4. Create Theme File

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

### 5. Import Theme CSS

```js
// src/index.js
import '@flairjs/client/theme.css'
import { App } from './App'

// Render your app
```

## React Setup

```jsx
// src/App.jsx
import { flair } from '@flairjs/client'

const App = () => {
  return (
    <div className="app">
      <h1>Hello Flair!</h1>
    </div>
  )
}

App.flair = flair({
  '.app': {
    fontFamily: 'system-ui, sans-serif',
    padding: '$space.4',
    backgroundColor: '$colors.primary',
    color: 'white'
  }
})

export { App }
```

## License

MIT
