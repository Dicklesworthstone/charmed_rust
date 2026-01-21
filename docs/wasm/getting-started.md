# Getting Started with charmed-wasm

This guide will help you set up charmed-wasm in your web project.

## Prerequisites

- Node.js 18+ (for npm-based projects)
- A modern browser with WebAssembly support

## Installation

### Using npm/yarn/pnpm

```bash
# npm
npm install @charmed/wasm

# yarn
yarn add @charmed/wasm

# pnpm
pnpm add @charmed/wasm
```

### Using a CDN

For quick prototyping, you can load directly from a CDN:

```html
<script type="module">
  import init, { newStyle } from 'https://unpkg.com/@charmed/wasm@latest/charmed_wasm.js';
  // ...
</script>
```

## Basic Setup

### Step 1: Initialize the Module

Before using any charmed-wasm functions, you must initialize the WASM module:

```typescript
import init, { newStyle } from '@charmed/wasm';

// Initialize once at app startup
await init();

// Now you can use the styling functions
const style = newStyle().bold();
```

### Step 2: Create Your First Style

```typescript
import init, { newStyle } from '@charmed/wasm';

async function main() {
  await init();

  // Create a simple styled string
  const greeting = newStyle()
    .foreground('#ff6b6b')  // Coral color
    .bold()                  // Make it bold
    .render('Hello, World!');

  console.log(greeting);
}

main();
```

### Step 3: Display in HTML

The rendered output is HTML with inline styles:

```typescript
const output = newStyle()
  .foreground('#61dafb')
  .background('#1a1a2e')
  .paddingVH(1, 2)
  .render('Styled Content');

// Insert into the DOM
document.getElementById('container').innerHTML = `<pre>${output}</pre>`;
```

**Important**: Wrap the output in a `<pre>` tag to preserve whitespace and line breaks.

## Bundler Configuration

### Vite

Vite works out of the box with WASM. Just import and use:

```typescript
import init, { newStyle } from '@charmed/wasm';
```

If you're using the local package during development, you may need to exclude it from optimization:

```typescript
// vite.config.ts
export default defineConfig({
  optimizeDeps: {
    exclude: ['@charmed/wasm'],
  },
});
```

### Webpack 5

Webpack 5 has built-in WASM support. Enable the `asyncWebAssembly` experiment:

```javascript
// webpack.config.js
module.exports = {
  experiments: {
    asyncWebAssembly: true,
  },
};
```

### Create React App

CRA doesn't support WASM imports out of the box. Use CRACO or eject:

```javascript
// craco.config.js
module.exports = {
  webpack: {
    configure: (config) => {
      config.experiments = {
        ...config.experiments,
        asyncWebAssembly: true,
      };
      return config;
    },
  },
};
```

## TypeScript Support

charmed-wasm includes TypeScript definitions. Import types as needed:

```typescript
import init, { newStyle, JsStyle } from '@charmed/wasm';

function applyStyle(style: JsStyle, text: string): string {
  return style.render(text);
}
```

## Error Handling

### Initialization Errors

Always handle potential WASM loading failures:

```typescript
try {
  await init();
  console.log('WASM loaded successfully');
} catch (error) {
  console.error('Failed to load WASM:', error);
  // Fall back to plain text or show error message
}
```

### Runtime Errors

Style methods are designed to be forgiving, but you should still wrap execution:

```typescript
try {
  const output = newStyle()
    .foreground('#invalid') // Invalid colors are handled gracefully
    .render('Test');
} catch (error) {
  console.error('Styling error:', error);
}
```

## Common Patterns

### Creating Reusable Styles

```typescript
// Define base styles
const primaryButton = newStyle()
  .background('#3498db')
  .foreground('#ffffff')
  .bold()
  .paddingVH(0, 2);

const dangerButton = newStyle()
  .background('#e74c3c')
  .foreground('#ffffff')
  .bold()
  .paddingVH(0, 2);

// Use them
const confirm = primaryButton.copy().render('Confirm');
const cancel = dangerButton.copy().render('Cancel');
```

### Layout Composition

```typescript
import { newStyle, joinVertical, joinHorizontal } from '@charmed/wasm';

// Vertical stack
const menu = joinVertical(0, [
  newStyle().bold().render('File'),
  newStyle().render('Edit'),
  newStyle().render('View'),
]);

// Horizontal row
const toolbar = joinHorizontal(0.5, [
  newStyle().render('[Save]'),
  newStyle().render('[Load]'),
  newStyle().render('[Exit]'),
]);
```

## Next Steps

- Read the [API Reference](./api-reference.md) for complete method documentation
- Check out [Examples](./examples/) for framework-specific guides
- Try the [Live Demo](https://dicklesworthstone.github.io/charmed_rust/demo)
