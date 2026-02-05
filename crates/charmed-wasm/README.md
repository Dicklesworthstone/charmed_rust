# charmed-wasm

WebAssembly bindings for charmed_rust styling and layout primitives.

## Role in the charmed_rust (FrankenTUI) stack

charmed-wasm is the web-facing bridge of the ecosystem. It re-exports WASM-
friendly APIs for `lipgloss` so that the same styling model can be shared between
terminal UIs and web-based previews or tooling. It does not depend on bubbletea;
it focuses purely on styling and layout primitives.

## Crates.io package

Package name: `charmed-wasm`  
Library crate name: `charmed_wasm`

## Typical usage (JavaScript)

```javascript
import init, { newStyle, Position } from "charmed-wasm";

async function main() {
  await init();
  const style = newStyle().foreground("#ff69b4").padding(1, 2, 1, 2);
  const rendered = style.render("Hello, Web");
  document.body.innerHTML = `<pre>${rendered}</pre>`;
}

main();
```

## Where to look next

- `crates/charmed-wasm/src/lib.rs`
