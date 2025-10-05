# @flairjs/core

The core transformation engine for Flair, built with Rust for maximum performance.

## Overview

This package contains the core logic for transforming JSX files with Flair styles. It uses:

- **OXC** for fast AST parsing and manipulation
- **Lightning CSS** for CSS parsing, transformation, and optimization
- **NAPI-RS** for Node.js bindings

## Installation

```bash
npm install @flairjs/core
```

## API

### `transformCode(code, filePath, options, cssPreprocessor?)`

Transforms JSX/TSX code containing Flair styles.

**Parameters:**
- `code: string` - The source code to transform
- `filePath: string` - Path to the file being transformed
- `options: TransformOptions` - Transformation options
- `cssPreprocessor?: (css: string) => string` - Optional CSS preprocessor function

**Returns:** `TransformOutput | null`

### TransformOptions

```typescript
interface TransformOptions {
  cssOutDir: string                 // Output directory for generated CSS
  classNameList?: Array<string>     // List of class names to process
  useTheme?: boolean               // Enable theme processing
  theme?: Theme                    // Theme configuration
  appendTimestampToCssFile?: boolean // Add timestamp to CSS filename
}
```

### TransformOutput

```typescript
interface TransformOutput {
  code: string              // Transformed JavaScript/TypeScript code
  sourcemap?: string       // Source map (if generated)
  css: string             // Extracted CSS
  logs: Array<LogEntry>   // Build logs and warnings
  generatedCssName?: string // Name of generated CSS file
}
```

### Theme Interface

```typescript
interface Theme {
  breakpoints: Record<string, string>
  prefix?: string
}
```

## Platform Support

This package includes native binaries for:

- Windows (x64, x86, ARM64)
- macOS (x64, ARM64)
- Linux (x64, ARM64, ARMv7, musl variants)
- FreeBSD (x64)
- Android (ARM64, ARMv7)
- WebAssembly (WASI)


## Development

### Building from Source

```bash
# Install Rust and Node.js dependencies
pnpm install

# Build the native module in debug mode
pnpm run build:debug
# Or prod build
pnpm run build

# Run tests
pnpm test
```

### Architecture

The core is structured as follows:

- `src/lib.rs` - Main entry point and NAPI bindings
- `src/transform.rs` - Core transformation logic
- `src/parse_css.rs` - CSS parsing and processing
- `src/style_tag.rs` - `<Style>` tag handling
- `src/flair_property.rs` - `.flair` property processing
- `src/update_attribute.rs` - Class name injection

## License

MIT
