# Flair ✨

A build-time CSS-in-JSX solution that brings the power of modern CSS to your React components with zero runtime overhead.

Try it online on StackBlitz.

[React Vite](https://stackblitz.com/edit/vitejs-vite-kxbnhpuc?file=src%2FApp.tsx) | [SolidJS Vite](https://stackblitz.com/edit/solidjs-templates-famw2yzx?file=src%2FApp.tsx) | [Preact Vite](https://stackblitz.com/edit/vitejs-vite-zt9k3zr7?file=src%2Fapp.tsx)

## Features

- 🚀 **Zero Runtime** - All CSS processing happens at build time
- 💅 **Multiple Styling APIs** - Choose between `<Style>` tags, `flair()` objects, or `css` template literals
- 🌙 **Theme System** - Built-in theming with TypeScript intellisense
- 🔧 **Build Tool Integration** - Supports Vite, NextJS, Rollup, Webpack, and Parcel
- 🎯 **Scoped by Default** - Component-scoped styles with global override option
- ⚡ **Rust-Powered** - Fast AST parsing with OXC and CSS processing with Lightning CSS
- 🔍 **Static Analysis** - Class names and CSS are analyzed at build time for optimal performance

## Quick Start

### Installation

```bash
# Install client packages
npm install @flairjs/client

# Install your bundler plugin
npm install @flairjs/vite-plugin        # For Vite
npm install @flairjs/rollup-plugin      # For Rollup
npm install @flairjs/webpack-loader     # For Webpack
npm install @flairjs/parcel-transformer # For Parcel
```

### Basic Setup

#### Vite Configuration

```js
// vite.config.js
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import flairjs from "@flairjs/vite-plugin";

export default defineConfig({
  plugins: [react(), flairjs()],
});
```

#### Component Usage

```jsx
import { flair } from "@flairjs/client";

const Button = () => {
  return <button className="button">Click me!</button>;
};

// Style with flair object
Button.flair = flair({
  ".button": {
    backgroundColor: "blue",
    color: "white",
    padding: "12px 24px",
    borderRadius: "8px",
    border: "none",
    "&:hover": {
      backgroundColor: "darkblue",
    },
  },
});

export default Button;
```

## Styling Methods

Flair provides three ways to write CSS in your components:

### 1. Flair Object API

```jsx
import { flair } from "@flairjs/client";

const Card = () => <div className="card">Content</div>;

Card.flair = flair({
  ".card": {
    backgroundColor: "white",
    borderRadius: "8px",
    boxShadow: "0 2px 4px rgba(0,0,0,0.1)",
    padding: "16px",
  },
});
```

### 2. CSS Template Literals

```jsx
import { css } from "@flairjs/client";

const Card = () => <div className="card">Content</div>;

Card.flair = css`
  .card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    padding: 16px;
  }
`;
```

### 3. Style Tag Components

```jsx
import { Style } from "@flairjs/client/react";

const Card = () => {
  return (
    <>
      <div className="card">Content</div>
      <Style>{`
        .card {
          background-color: white;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
          padding: 16px;
        }
      `}</Style>
    </>
  );
};
```

## Global Styles

By default, styles are scoped to components. You can make styles global:

### With Style Tag

```jsx
import { Style } from "@flairjs/client/react";

const App = () => {
  return (
    <>
      <Style global>{`
        body {
          margin: 0;
          font-family: -apple-system, BlinkMacSystemFont, sans-serif;
        }
      `}</Style>
      {/* Your app content */}
    </>
  );
};
```

### With globalFlair Property

```jsx
const App = () => <div>App content</div>;

App.globalFlair = css`
  body {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  }
`;
```

## Theming

### Setup

To enable theming support, you need to:

1. **Import the theme CSS** in your top-level file (e.g., `main.tsx`, `App.tsx`, or `index.tsx`):

```jsx
import "@flairjs/client/theme.css";
```

2. **Create a theme configuration file** `flair.theme.ts` in your project root:

```typescript
// flair.theme.ts
import { defineConfig } from "@flairjs/client";

const theme = defineConfig({
  prefix: "flair",
  selector: "body",
  tokens: {
    colors: {
      primary: "#3b82f6",
      secondary: "#64748b",
      success: "#10b981",
      danger: "#ef4444",
    },
    fonts: {
      family: "'Inter', sans-serif",
      size: {
        sm: "14px",
        md: "16px",
        lg: "18px",
      },
    },
    space: {
      1: "4px",
      2: "8px",
      3: "12px",
      4: "16px",
      5: "20px",
      6: "24px",
    },
  },
  breakpoints: {
    sm: "640px",
    md: "768px",
    lg: "1024px",
    xl: "1280px",
  },
});

export default theme;
export type Theme = typeof theme;
```

### Using Theme Tokens

```jsx
import { flair } from "@flairjs/client";

const Button = () => <button className="button">Click me</button>;

Button.flair = flair({
  ".button": {
    backgroundColor: "$colors.primary",
    color: "white",
    padding: "$space.3 $space.5",
    fontSize: "$fonts.size.md",
    fontFamily: "$fonts.family",
  },
});
```

### TypeScript Intellisense

For theme token autocomplete, extend the `FlairTheme` interface:

```typescript
// types/flair.d.ts
import { Theme } from "../flair.theme";

declare module "@flairjs/client" {
  export interface FlairTheme extends Theme {}
}
```

### Responsive Design

```jsx
Button.flair = flair({
  ".button": {
    padding: "$space.2 $space.3",
    fontSize: "$fonts.size.sm",

    // Responsive breakpoints
    "$screen md": {
      padding: "$space.3 $space.5",
      fontSize: "$fonts.size.md",
    },

    "$screen lg": {
      padding: "$space.4 $space.6",
      fontSize: "$fonts.size.lg",
    },
  },
});
```

## Bundler Integration

### Vite

```js
// vite.config.js
import flairjs from "@flairjs/vite-plugin";

export default {
  plugins: [
    flairjs({
      classNameList: ["className", "class"],
      // Optional: Include/exclude files
      include: ["src/**/*.{tsx,jsx}"],
      exclude: ["node_modules/**"],
    }),
  ],
};
```

### Webpack

```js
// webpack.config.js
module.exports = {
  module: {
    rules: [
      {
        test: /\.(tsx|jsx)$/,
        use: ["@flairjs/webpack-loader"],
      },
    ],
  },
};
```

### Rollup

```js
// rollup.config.js
import flairjs from "@flairjs/rollup-plugin";

export default {
  plugins: [
    // Make sure rollup is configured to handle css imports.
    // Add flair before any other JSX parsers
    flairjs(),
  ],
};
```

### Parcel

```json
// .parcelrc
{
  "extends": "@parcel/config-default",
  "transformers": {
    "*.{tsx,jsx}": ["@flairjs/parcel-transformer", "..."]
  }
}
```

## Advanced Features

### Static Analysis and Class Name Inference

Since Flair is a **build-time library**, all CSS and class names must be **statically inferrable** at build time. Flair cannot process dynamically generated CSS or class names that are only known at runtime.

#### What Works (Static Analysis)

In most cases, Flair can infer class names automatically:

```jsx
// ✅ Direct className
<button className="btn">Click me</button>

// ✅ Variable stored className
const buttonClass = "btn btn-primary"
<button className={buttonClass}>Click me</button>

// ✅ Function parameters
const variant = "primary"
<button className={clsx(variant)}>Click me</button>
```

#### What Doesn't Work (Dynamic Class Names)

```jsx
// ❌ Function calls - Flair cannot infer the return value
<button className={someFunction()}>Click me</button>

// ❌ Complex runtime expressions
<button className={`btn btn-${variant}`}>Click me</button>
```

### Class Name Utilities: `c()` and `cn()`

When Flair cannot directly infer a class name (e.g., when returned from a function), use the `c()` or `cn()` utilities to signal which class names should be included:

```jsx
import { c, cn } from "@flairjs/client";

// Both c() and cn() are identical - they simply return what you pass to them
// Their purpose is to signal to Flair's build-time analyzer which class names to include

function getButtonClass() {
  // ✅ Signal to Flair that 'btn' and 'btn-primary' should be included
  return c("btn btn-primary");
}

const Button = () => {
  return <button className={getButtonClass()}>Click me</button>;
};

Button.flair = flair({
  ".btn": { padding: "12px 24px" },
  ".btn-primary": { backgroundColor: "blue" },
});
```

**Important Notes:**

- `c()` and `cn()` are **not** like `clsx` or `classnames` - they don't merge or conditionally apply classes
- They are simple pass-through functions: `c('foo')` just returns `'foo'`
- Their only purpose is to help Flair's static analyzer find class names in your code
- At runtime, they have zero overhead (they literally just return their input)

### Nesting and Pseudo-selectors

```jsx
Card.flair = flair({
  ".card": {
    backgroundColor: "white",

    "&:hover": {
      backgroundColor: "#f9f9f9",
    },

    "&.active": {
      borderColor: "$colors.primary",
    },

    "& .title": {
      fontSize: "$fonts.size.lg",
      fontWeight: "bold",
    },
  },
});
```

### Media Queries

```jsx
Card.flair = flair({
  ".card": {
    padding: "$space.3",

    "@media (min-width: 768px)": {
      padding: "$space.5",
    },
  },
});
```

## Framework Support

Currently, `flair` property is supported in all JSX frameworks.
Flair component is supported in:

- ✅ React (via `@flairjs/client/react`)
- ✅ Preact (via `@flairjs/client/preact`)
- ✅ SolidJS (via `@flairjs/client/solidjs`)

## Performance

- **Zero Runtime Overhead** - All CSS is extracted at build time
- **Optimal Bundle Size** - Only the CSS you use is included
- **Fast Builds** - Rust-powered transformation with OXC and Lightning CSS
- **Efficient Caching** - Smart caching of transformed components

## Browser Support

Flair generates modern CSS that works in all evergreen browsers. Legacy browser support depends on your build setup and CSS processing pipeline.

## Contributing

We welcome contributions! Here's how to get started:

### Development Setup

1. **Clone the repository**

   ```bash
   git clone https://github.com/akzhy/flairjs.git
   cd flairjs
   ```

2. **Install dependencies**

   ```bash
   pnpm install
   ```

3. **Build packages**

   ```bash
   # Build all packages
   pnpm build

   # Or build specific packages
   pnpm build:core              # Build core Rust package
   pnpm build:non-core-packages # Build all other packages
   ```

### Making Changes

When contributing changes, please follow these steps:

1. **Create a new branch** for your feature or bugfix

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** and ensure all packages build successfully

3. **Add a changeset** to document your changes

   ```bash
   pnpm changeset
   ```

   This will prompt you to:

   - Select which packages are affected by your changes
   - Specify the type of change (major, minor, patch)
   - Write a description of your changes

   The changeset system ensures proper versioning and generates changelogs automatically.

4. **Commit your changes** including the changeset file

   ```bash
   git add .
   git commit -m "a sensible commit message"
   ```

5. **Push your branch** and create a pull request
   ```bash
   git push origin feature/your-feature-name
   ```

### Changeset Guidelines

- **Patch** (0.0.x): Bug fixes, documentation updates, internal refactors
- **Minor** (0.x.0): New features, non-breaking enhancements
- **Major** (x.0.0): Breaking changes, API changes

Example changeset workflow:

```bash
# After making changes to @flairjs/vite-plugin
pnpm changeset

# You'll be prompted:
# - Select @flairjs/vite-plugin
# - Choose "patch" for a bugfix
# - Describe: "Fixed issue with theme token resolution"
```

### Testing

Before submitting a PR:

- Ensure all packages build without errors: `pnpm build`
- Test your changes in the example project: `examples/vite-react-ts`
- Run any available tests in the affected packages

### Questions?

Feel free to open an issue for any questions or discussions about contributing!

## License

MIT License - see [LICENSE](LICENSE) for details.

## Packages

This monorepo contains the following packages:

- [`@flairjs/core`](./packages/core) - Core transformation engine (Rust + NAPI)
- [`@flairjs/client`](./packages/client) - Client-side utilities and types
- [`@flairjs/bundler-shared`](./packages/shared) - Shared bundler utilities
- [`@flairjs/vite-plugin`](./packages/vite-plugin) - Vite integration
- [`@flairjs/rollup-plugin`](./packages/rollup-plugin) - Rollup integration
- [`@flairjs/webpack-loader`](./packages/webpack-loader) - Webpack integration
- [`@flairjs/parcel-transformer`](./packages/parcel-transformer) - Parcel integration
