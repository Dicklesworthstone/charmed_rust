# Vanilla JavaScript Integration

Using charmed-wasm with plain JavaScript (no framework).

## Basic HTML Setup

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>charmed-wasm Demo</title>
  <style>
    body {
      background: #1e1e1e;
      font-family: system-ui, sans-serif;
      padding: 2rem;
    }

    .terminal {
      font-family: 'JetBrains Mono', 'Fira Code', monospace;
      background: #0d1117;
      border-radius: 8px;
      padding: 1rem;
      color: #c9d1d9;
      white-space: pre-wrap;
      line-height: 1.4;
    }
  </style>
</head>
<body>
  <div id="app" class="terminal">Loading...</div>

  <script type="module">
    import init, { newStyle, joinVertical } from '@charmed/wasm';

    async function main() {
      // Initialize WASM
      await init();

      // Create styled content
      const title = newStyle()
        .foreground('#61dafb')
        .bold()
        .render('Hello, charmed-wasm!');

      const subtitle = newStyle()
        .foreground('#888888')
        .italic()
        .render('Terminal styling in the browser');

      const box = newStyle()
        .borderStyle('rounded')
        .borderAll()
        .foreground('#58a6ff')
        .paddingAll(1)
        .render(joinVertical(0.5, [title, '', subtitle]));

      // Display
      document.getElementById('app').innerHTML = box;
    }

    main().catch(console.error);
  </script>
</body>
</html>
```

## Dynamic Content Updates

```html
<script type="module">
  import init, { newStyle } from '@charmed/wasm';

  let wasmReady = false;

  async function setup() {
    await init();
    wasmReady = true;
  }

  function createButton(text, color, isActive = false) {
    if (!wasmReady) return text;

    let style = newStyle()
      .paddingVH(0, 2);

    if (isActive) {
      style = style.background(color).foreground('#ffffff').bold();
    } else {
      style = style.foreground(color);
    }

    return style.render(text);
  }

  function renderMenu(activeIndex) {
    const items = [
      { text: 'Home', color: '#3498db' },
      { text: 'About', color: '#2ecc71' },
      { text: 'Contact', color: '#e74c3c' },
    ];

    const rendered = items.map((item, i) =>
      createButton(item.text, item.color, i === activeIndex)
    );

    document.getElementById('menu').innerHTML = rendered.join(' ');
  }

  // Initialize and render
  setup().then(() => {
    renderMenu(0);

    // Handle keyboard navigation
    let activeIndex = 0;
    document.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowRight') {
        activeIndex = (activeIndex + 1) % 3;
        renderMenu(activeIndex);
      } else if (e.key === 'ArrowLeft') {
        activeIndex = (activeIndex - 1 + 3) % 3;
        renderMenu(activeIndex);
      }
    });
  });
</script>
```

## Live Code Editor

Build a simple live editor:

```html
<style>
  .editor-container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .editor {
    font-family: monospace;
    font-size: 14px;
    padding: 1rem;
    border: 1px solid #333;
    border-radius: 4px;
    background: #0d1117;
    color: #c9d1d9;
    resize: vertical;
    min-height: 200px;
  }

  .preview {
    font-family: monospace;
    padding: 1rem;
    border: 1px solid #333;
    border-radius: 4px;
    background: #1e1e1e;
    color: #d4d4d4;
    white-space: pre-wrap;
    min-height: 200px;
  }
</style>

<div class="editor-container">
  <textarea id="code" class="editor">newStyle()
  .foreground('#ff6b6b')
  .bold()
  .render('Hello!')</textarea>
  <pre id="preview" class="preview"></pre>
</div>

<script type="module">
  import init, { newStyle, joinHorizontal, joinVertical, place } from '@charmed/wasm';

  let wasm = null;

  async function setup() {
    await init();
    wasm = { newStyle, joinHorizontal, joinVertical, place };
    updatePreview();
  }

  function updatePreview() {
    const code = document.getElementById('code').value;
    const preview = document.getElementById('preview');

    try {
      // Create execution context
      const fn = new Function(
        'newStyle', 'joinHorizontal', 'joinVertical', 'place',
        `return (${code})`
      );

      const result = fn(
        wasm.newStyle,
        wasm.joinHorizontal,
        wasm.joinVertical,
        wasm.place
      );

      preview.innerHTML = result;
      preview.style.borderColor = '#333';
    } catch (e) {
      preview.textContent = `Error: ${e.message}`;
      preview.style.borderColor = '#e74c3c';
    }
  }

  // Debounce updates
  let timeout;
  document.getElementById('code').addEventListener('input', () => {
    clearTimeout(timeout);
    timeout = setTimeout(updatePreview, 300);
  });

  setup();
</script>
```

## Performance Tips

1. **Cache styles**: Create style objects once and reuse them:

```javascript
// Good: Create once
const headerStyle = newStyle().bold().foreground('#61dafb');

function renderHeader(text) {
  return headerStyle.render(text);
}

// Avoid: Creating new style every render
function renderHeader(text) {
  return newStyle().bold().foreground('#61dafb').render(text);
}
```

2. **Use `.copy()` for variations**:

```javascript
const baseStyle = newStyle().paddingVH(0, 2);

const success = baseStyle.copy().foreground('#2ecc71');
const error = baseStyle.copy().foreground('#e74c3c');
const warning = baseStyle.copy().foreground('#f39c12');
```

3. **Batch DOM updates**:

```javascript
// Good: Single DOM update
const parts = items.map(item => renderItem(item));
container.innerHTML = parts.join('');

// Avoid: Multiple DOM updates
items.forEach(item => {
  container.innerHTML += renderItem(item);
});
```
