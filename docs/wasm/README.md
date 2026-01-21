# charmed-wasm

Terminal UI styling for the web, powered by WebAssembly.

charmed-wasm brings the expressive styling API of [lipgloss](https://github.com/charmbracelet/lipgloss) to web applications. Create beautiful terminal-inspired UIs with the same chainable style builder pattern you know from Rust or Go.

## Features

- **Chainable Style API** - Build styles fluently with `.foreground()`, `.bold()`, `.padding()`, etc.
- **Color Support** - Hex colors, ANSI 256 colors, and RGB values
- **Borders** - Multiple preset styles: rounded, double, thick, ASCII, and more
- **Layout** - Padding, margins, width/height, and alignment
- **Composition** - Join elements horizontally or vertically with alignment control
- **Small Bundle** - Optimized WASM binary under 200KB gzipped

## Installation

### npm / yarn / pnpm

```bash
npm install @charmed/wasm
# or
yarn add @charmed/wasm
# or
pnpm add @charmed/wasm
```

### CDN (ESM)

```html
<script type="module">
  import init, { newStyle } from 'https://unpkg.com/@charmed/wasm@latest/charmed_wasm.js';

  await init();
  const style = newStyle().bold().foreground('#ff6b6b');
  console.log(style.render('Hello!'));
</script>
```

## Quick Start

### ES Modules (Bundler)

```typescript
import init, { newStyle, joinVertical } from '@charmed/wasm';

async function main() {
  // Initialize the WASM module
  await init();

  // Create a styled header
  const header = newStyle()
    .foreground('#61dafb')
    .background('#1a1a2e')
    .bold()
    .paddingVH(1, 2)
    .render('Welcome');

  // Create body text
  const body = newStyle()
    .foreground('#888888')
    .render('Terminal styling in the browser!');

  // Combine vertically
  const result = joinVertical(0.5, [header, body]);

  document.getElementById('output').innerHTML = `<pre>${result}</pre>`;
}

main();
```

### Direct Browser Usage

```html
<!DOCTYPE html>
<html>
<head>
  <title>charmed-wasm Demo</title>
  <style>
    pre {
      font-family: 'JetBrains Mono', monospace;
      background: #1e1e1e;
      padding: 20px;
      color: #d4d4d4;
    }
  </style>
</head>
<body>
  <pre id="output"></pre>

  <script type="module">
    import init, { newStyle } from './pkg/charmed_wasm.js';

    await init();

    const style = newStyle()
      .borderStyle('rounded')
      .borderAll()
      .foreground('#58a6ff')
      .paddingAll(1)
      .render('Hello from WASM!');

    document.getElementById('output').innerHTML = style;
  </script>
</body>
</html>
```

## Live Demo

Try the interactive demo at: [charmed-rust Demo](https://dicklesworthstone.github.io/charmed_rust/demo)

## Documentation

- [Getting Started](./getting-started.md) - Installation and setup guide
- [API Reference](./api-reference.md) - Complete API documentation
- [Examples](./examples/) - Framework-specific integration guides
  - [Vanilla JS](./examples/vanilla-js.md)
  - [React](./examples/react.md)
  - [Vue](./examples/vue.md)
- [Advanced Topics](./advanced/)
  - [Performance](./advanced/performance.md)
  - [TypeScript](./advanced/typescript.md)

## Browser Support

charmed-wasm works in all modern browsers that support WebAssembly:

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## Contributing

Contributions are welcome! Please see the main [charmed_rust repository](https://github.com/Dicklesworthstone/charmed_rust) for guidelines.

## License

MIT License - see [LICENSE](../../LICENSE) for details.
