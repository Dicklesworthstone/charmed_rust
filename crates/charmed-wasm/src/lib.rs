//! # charmed-wasm
//!
//! Terminal UI components for the web, compiled to WebAssembly.
//!
//! This crate provides WebAssembly bindings for the `charmed_rust` library,
//! allowing you to use lipgloss styling in web applications.
//!
//! ## Quick Start (JavaScript)
//!
//! ```javascript
//! import init, { newStyle, joinVertical, Position } from 'charmed-wasm';
//!
//! async function main() {
//!     await init();
//!
//!     const style = newStyle()
//!         .foreground("#ff00ff")
//!         .background("#1a1a1a")
//!         .bold()
//!         .padding(1, 2, 1, 2);
//!
//!     const rendered = style.render("Hello, World!");
//!     document.body.innerHTML = `<pre>${rendered}</pre>`;
//! }
//!
//! main();
//! ```
//!
//! ## Available APIs
//!
//! ### Styling (from lipgloss)
//!
//! - `newStyle()` - Create a new style builder
//! - `JsStyle` - Chainable style configuration
//! - `JsColor` - Color utilities
//!
//! ### Layout
//!
//! - `joinHorizontal(position, items)` - Join strings horizontally
//! - `joinVertical(position, items)` - Join strings vertically
//! - `place(width, height, hPos, vPos, content)` - Place content in a container
//!
//! ### Utilities
//!
//! - `stringWidth(s)` - Get visible width of a string
//! - `stringHeight(s)` - Get height (line count) of a string
//! - `borderPresets()` - Get available border preset names

#![forbid(unsafe_code)]

// Use wee_alloc for smaller binaries (optional)
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use wasm_bindgen::prelude::*;

// Re-export everything from lipgloss wasm module
pub use lipgloss::wasm::*;

/// Module version information.
#[must_use]
#[wasm_bindgen(js_name = "version")]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check if the module is properly initialized.
#[must_use]
#[wasm_bindgen(js_name = "isReady")]
#[allow(clippy::missing_const_for_fn)] // wasm_bindgen doesn't support const fn
pub fn is_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }
}
