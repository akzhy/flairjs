# @flairjs/bundler-shared

Shared utilities and logic for Flair bundler integrations.

## Overview

This package contains common functionality used by all Flair bundler plugins (Vite, Rollup, Webpack, Parcel). It provides:

- File processing utilities
- Theme system integration
- CSS generation and management
- Shared configuration types


## API

### Core Functions

#### `initializeSharedContext(options?)`

Initializes the shared plugin context with theme detection and CSS directory setup.

```typescript
const context = await initializeSharedContext({
  cssPreprocessor: (css) => css,
  buildThemeFile: (theme) => buildThemeTokens(theme)
})
```

#### `shouldProcessFile(id, include?, exclude?)`

Determines if a file should be processed based on include/exclude patterns.

```typescript
if (shouldProcessFile(filePath, ['src/**/*.tsx'], ['**/*.test.*'])) {
  // Process the file
}
```

#### `transformCode(code, filePath, options)`

Transforms source code containing Flair styles.

```typescript
const result = transformCode(sourceCode, filePath, {
  cssOutDir: './dist/css', // Typically points to the directory @flairjs/client/generated-css
  useTheme: true,
  theme: userTheme
})
```

#### `getUserTheme(options?)`

Retrieves and processes the user's theme configuration.

```typescript
const userTheme = await getUserTheme({ ignoreCache: false })
if (userTheme) {
  console.log('Theme loaded:', userTheme.theme)
}
```

### Utility Functions

#### `getGeneratedCssDir()`

Returns the path to the generated CSS directory.

```typescript
const cssDir = getGeneratedCssDir()
// Returns: node_modules/@flairjs/client/generated-css
```

#### `setupGeneratedCssDir(options?)`

Sets up the CSS output directory with optional cleanup.

```typescript
const cssDir = await setupGeneratedCssDir({
  clearExisting: true // Clean up old files
})
```

#### `setupUserThemeFile(options)`

Sets up theme file processing with watch capabilities.

```typescript
await setupUserThemeFile({
  buildThemeFile: (theme) => buildThemeTokens(theme),
  onThemeFileChange: () => {
    console.log('Theme updated')
  }
})
```

#### `removeOutdatedCssFiles()`

Cleans up outdated CSS files from previous builds.

```typescript
await removeOutdatedCssFiles()
```

## Types

### `SharedPluginOptions`

```typescript
interface SharedPluginOptions {
  /**
   * Preprocess CSS before Lightning CSS processing
   */
  cssPreprocessor?: (css: string, id: string) => string
  
  /**
   * File patterns to include
   */
  include?: string | string[]
  
  /**
   * File patterns to exclude
   */
  exclude?: string | string[]
  
  /**
   * Custom theme file builder
   */
  buildThemeFile?: (theme: FlairThemeConfig) => string
  
  /**
   * List of class names for optimization
   */
  classNameList?: string[]
}
```

### `GetUserThemeResult`

```typescript
interface GetUserThemeResult {
  theme: FlairThemeConfig
  originalPath: string
  resolvedPath: string
}
```

## Usage in Bundler Plugins

### Basic Plugin Structure

```typescript
import {
  initializeSharedContext,
  shouldProcessFile,
  transformCode,
  getUserTheme
} from '@flairjs/bundler-shared'

export default function myFlairPlugin(options) {
  let context
  
  return {
    name: 'my-flair-plugin',
    async buildStart() {
      context = await initializeSharedContext(options)
    },
    
    transform(code, id) {
      if (!shouldProcessFile(id, options?.include, options?.exclude)) {
        return null
      }
      
      return transformCode(code, id, {
        cssOutDir: context.flairGeneratedCssDir,
        useTheme: !!context.userTheme,
        theme: context.userTheme?.theme
      })
    }
  }
}
```

### Theme Integration

```typescript
// Handle theme CSS imports
async resolveId(id) {
  if (id === '@flairjs/client/theme.css') {
    return id
  }
}

async load(id) {
  if (id === '@flairjs/client/theme.css') {
    const userTheme = await getUserTheme()
    if (userTheme && context.buildThemeCSS) {
      return context.buildThemeCSS(userTheme.theme)
    }
    return ''
  }
}
```

## File Processing

### Include/Exclude Patterns

The shared utilities use glob patterns for file matching:

```typescript
// Include specific directories
include: ['src/**/*.{tsx,jsx}', 'components/**/*.tsx']

// Exclude test files and node_modules
exclude: ['**/*.test.*', '**/*.spec.*', 'node_modules/**']

// Default behavior
// include: ['**/*.{tsx,jsx}'] (all TSX/JSX files)
// exclude: ['node_modules/**']
```

### CSS Preprocessing

Custom CSS preprocessing pipeline:

```typescript
const options = {
  cssPreprocessor: (css, filePath) => {
    // Remove comments in production
    if (process.env.NODE_ENV === 'production') {
      css = css.replace(/\/\*.*?\*\//g, '')
    }
    
    // Add file header
    css = `/* Generated from ${filePath} */\n${css}`
    
    // Custom transformations
    css = css.replace(/custom-prefix-/g, 'my-app-')
    
    return css
  }
}
```

## Theme System

### Theme File Resolution

The shared utilities automatically detect theme files:

1. `flair.theme.ts` (preferred)
2. `flair.theme.js`
3. `flair.config.ts`
4. `flair.config.js`

### Theme Processing

```typescript
// Theme file content
const themeConfig = defineConfig({
  tokens: {
    colors: { primary: '#blue' }
  }
})

// Processed theme CSS
const themeCSS = buildThemeTokens(themeConfig)
// Output: 
// body {
//   --colors-primary: #blue;
// }
```

## Error Handling

### Logging System

The shared utilities provide structured logging:

```typescript
interface LogEntry {
  message: string
  level: 'error' | 'warn' | 'info'
}

// Transform result includes logs
const result = transformCode(code, filePath, options)
if (result?.logs) {
  result.logs.forEach(log => {
    console[log.level](log.message)
  })
}
```

### Common Error Scenarios

- Theme file not found or invalid
- CSS syntax errors
- File permission issues
- Invalid configuration options

## Performance Optimization

### Caching Strategy

```typescript
// Theme caching
const userTheme = await getUserTheme({
  ignoreCache: false // Use cached theme if available
})

// CSS directory setup with cleanup control
const cssDir = await setupGeneratedCssDir({
  clearExisting: process.env.NODE_ENV === 'production'
})
```

### File Watching

```typescript
// Watch theme file for changes
await setupUserThemeFile({
  buildThemeFile: buildThemeTokens,
  onThemeFileChange: () => {
    // Trigger rebuild
    this.invalidate()
  }
})
```

## Contributing

When adding new bundler integrations:

1. Use the shared utilities for consistency
2. Follow the established patterns
3. Add appropriate error handling
4. Include tests for new functionality

## License

MIT
