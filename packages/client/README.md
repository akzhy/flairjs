# @flairjs/client

Client-side utilities and types for Flair CSS-in-JSX.

## Installation

```bash
npm install @flairjs/client
```

## Exports

### Styling Functions

```typescript
import { flair, css, c, cn } from '@flairjs/client'
```

#### `flair(styles)`

Create component styles using CSS-in-JS object syntax:

```jsx
import { flair } from '@flairjs/client'

const Button = () => <button className="button">Click me</button>

Button.flair = flair({
  ".button": {
    backgroundColor: 'blue',
    color: 'white',
    padding: '12px 24px',
    '&:hover': {
      backgroundColor: 'darkblue'
    }
  }
})
```

#### `css` Template Literal

Write CSS using template literal syntax:

```jsx
import { css } from '@flairjs/client'

const Button = () => <button className="button">Click me</button>

Button.flair = css`
  .button {
    background-color: blue;
    color: white;
    padding: 12px 24px;
    
    &:hover {
      background-color: darkblue;
    }
  }
`
```

#### `c()` and `cn()` Class Name Utilities

Simple pass-through functions that help Flair's static analyzer find class names in your code.

**Important**: `c()` and `cn()` are **not** like `clsx` or `classnames`. They don't merge or conditionally apply classes - they simply return whatever you pass to them.

```jsx
import { c, cn } from '@flairjs/client'

// Both c() and cn() are identical - they just return their input
// Their purpose is to signal to Flair's build-time analyzer which class names to include

function getButtonClass() {
  // ✅ Signal to Flair that 'btn' and 'btn-primary' should be included
  return c('btn btn-primary')
}

const Button = () => {
  return <button className={getButtonClass()}>Click me</button>
}

Button.flair = flair({
  '.btn': { padding: '12px 24px' },
  '.btn-primary': { backgroundColor: 'blue' }
})
```

**Why use them?** Since Flair is a build-time library, it needs to statically analyze your code to find class names. When you return a class name from a function like `<button className={someFunction()}>`, Flair can't infer what class name will be used. Wrapping the class name in `c()` or `cn()` inside the function signals to Flair which classes to include.

**Runtime behavior**: Zero overhead - they literally just return their input:
```js
c('foo') === 'foo'  // true
cn('bar') === 'bar' // true
```

### Theme System

```typescript
import { defineConfig } from '@flairjs/client'
```

**Setup**: To enable theming, import the theme CSS in your top-level file (e.g., `main.tsx`, `App.tsx`, or `index.tsx`):

```jsx
import '@flairjs/client/theme.css'
```

#### `defineConfig(config)`

Define your project's theme configuration:

```typescript
// flair.theme.ts
import { defineConfig } from '@flairjs/client'

const theme = defineConfig({
  prefix: 'my-app',
  selector: 'body',
  tokens: {
    colors: {
      primary: '#3b82f6',
      secondary: '#64748b'
    },
    fonts: {
      family: "'Inter', sans-serif",
      size: {
        sm: '14px',
        md: '16px',
        lg: '18px'
      }
    },
    space: {
      1: '4px',
      2: '8px',
      3: '12px',
      4: '16px'
    }
  },
  breakpoints: {
    sm: '640px',
    md: '768px',
    lg: '1024px'
  }
})

export default theme
export type Theme = typeof theme
```

### Component Libraries

#### React

```jsx
import { Style } from '@flairjs/client/react'

const MyComponent = () => {
  return (
    <>
      <div className="my-class">Styled content</div>
      <Style>{`
        .my-class {
          color: red;
        }
      `}</Style>
    </>
  )
}

// Global styles
const App = () => {
  return (
    <>
      <Style global>{`
        body {
          margin: 0;
          font-family: sans-serif;
        }
      `}</Style>
      {/* App content */}
    </>
  )
}
```

#### Preact

```jsx
import { Style } from '@flairjs/client/preact'

// Usage is identical to React
const MyComponent = () => {
  return (
    <>
      <div className="my-class">Styled content</div>
      <Style>{`
        .my-class { color: red; }
      `}</Style>
    </>
  )
}
```

#### SolidJS

```jsx
import { Style } from '@flairjs/client/solidjs'

const MyComponent = () => {
  return (
    <>
      <div class="my-class">Styled content</div>
      <Style>{`
        .my-class { color: red; }
      `}</Style>
    </>
  )
}
```

## TypeScript Support

### Theme Intellisense

Extend the `FlairTheme` interface for theme token autocomplete:

```typescript
// types/flair.d.ts
import { Theme } from '../flair.theme'

declare module '@flairjs/client' {
  export interface FlairTheme extends Theme {}
}
```

After this setup, you'll get autocomplete for theme tokens:

```jsx
Button.flair = flair({
  ".selector": {
    backgroundColor: '$colors.primary', // ← Autocomplete available
    padding: '$space.3 $space.4',      // ← Autocomplete available
    fontSize: '$fonts.size.md'         // ← Autocomplete available
  }
})
```

### Type Definitions

```typescript
// Theme token types
type ThemeTokens<T extends FlairTheme = FlairTheme> = 
  T extends { tokens: any } ? TokensOf<T['tokens']> : never

type BreakPointTokens<T extends FlairTheme = FlairTheme> = 
  T extends { breakpoints: any } ? `$screen ${keyof T['breakpoints']}` : never

// CSS object with theme support
type FlairCSS = {
  [K in string]?: FlairObject | FlairCSS
} & {
  [K in BreakPointTokens]?: FlairObject | FlairCSS
}
```

## Usage Patterns

### Responsive Design

```jsx
const Card = () => <div className="card">Content</div>

Card.flair = flair({
  '.card': {
    padding: '$space.2',
    fontSize: '$fonts.size.sm',
    
    // Responsive breakpoints
    '$screen md': {
      padding: '$space.4',
      fontSize: '$fonts.size.md'
    },
    
    '$screen lg': {
      padding: '$space.6',
      fontSize: '$fonts.size.lg'
    }
  }
})
```

### Nested Selectors

```jsx
Card.flair = flair({
  '.card': {
    backgroundColor: 'white',
    
    '&:hover': {
      backgroundColor: '#f9f9f9'
    },
    
    '&.active': {
      borderColor: '$colors.primary'
    },
    
    '& .title': {
      fontSize: '$fonts.size.lg',
      fontWeight: 'bold'
    },
    
    '& > .content': {
      padding: '$space.4'
    }
  }
})
```

### Global Styles

```jsx
// Using globalFlair property
const App = () => <div>App content</div>

App.globalFlair = css`
  * {
    box-sizing: border-box;
  }
  
  body {
    margin: 0;
    font-family: '$fonts.family';
    background-color: '$colors.background';
  }
`
```

## License

MIT