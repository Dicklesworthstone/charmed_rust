# charmed-wasm

WebAssembly bindings for charmed_rust styling and layout primitives.

## TL;DR

**The Problem:** Sharing a styling system between terminal and web UIs is
painful; you often duplicate theme logic.

**The Solution:** `charmed-wasm` exposes lipgloss-style builders in WASM so you
can reuse the same styling semantics in web contexts.

**Why charmed-wasm**

- **Shared styling model**: reuse lipgloss concepts on the web.
- **Small surface**: focused on styling and layout helpers.
- **WASM-friendly**: built on wasm-bindgen.

## Role in the charmed_rust (FrankenTUI) stack

charmed-wasm is the web-facing bridge for the ecosystem. It re-exports
`lipgloss` functionality via WASM bindings and does not depend on bubbletea.

## Crates.io package

Package name: `charmed-wasm`  
Library crate name: `charmed_wasm`

## Installation

```toml
[dependencies]
charmed_wasm = { package = "charmed-wasm", version = "0.1.2" }
```

## Quick Start (JavaScript)

```javascript
import init, { newStyle } from "charmed-wasm";

async function main() {
  await init();
  const style = newStyle().foreground("#ff69b4").padding(1, 2, 1, 2);
  const rendered = style.render("Hello, Web");
  document.body.innerHTML = `<pre>${rendered}</pre>`;
}

main();
```

## Building WASM

Typical build flow (using wasm-pack or your own bundler):

```bash
# Example using wasm-pack
wasm-pack build crates/charmed-wasm --target web
```

## Feature Flags

- `console_error_panic_hook` (default): nicer browser errors.
- `wee_alloc`: smaller WASM binary (trades performance for size).

```toml
charmed_wasm = { package = "charmed-wasm", version = "0.1.2", default-features = false, features = ["wee_alloc"] }
```

## API Overview

Bindings mirror a subset of lipgloss:

- `newStyle()` → style builder
- `joinHorizontal()` / `joinVertical()`
- `place()` for layout
- `stringWidth()` / `stringHeight()`

See `crates/charmed-wasm/src/lib.rs` for the full exported API.

## Troubleshooting

- **Module won’t load**: ensure your bundler supports WASM modules.
- **No output**: call `init()` before using any exports.

## Limitations

- Intended for styling, not full terminal emulation.
- API surface is intentionally small.

## FAQ

**Does it render to a canvas?**  
No. It returns strings; you choose how to display them.

**Is it compatible with Node?**  
Yes, if your toolchain supports wasm-bindgen outputs.

## About Contributions

Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

## License

MIT. See `LICENSE` at the repository root.
