# Syntax Highlighting Research for Glamour

> Research document for bead charmed_rust-hf0: [Syntax] Research syntect integration patterns

---

## Executive Summary

This document details the research findings for adding syntax highlighting to the glamour crate using the **syntect** library. The integration is technically feasible, performant, and follows established patterns from successful projects like bat, mdcat, and zola.

**Key Findings:**
- syntect is the de-facto standard for Rust syntax highlighting
- Integration with pulldown-cmark + lipgloss is straightforward
- WASM support exists but requires `fancy-regex` feature (no native C dependencies)
- Performance is excellent with lazy loading of syntax sets
- Theme → lipgloss color mapping is direct (RGBA to hex conversion)

---

## 1. Syntect API Overview

### Core Types

```rust
// SyntaxSet: Collection of all syntax definitions
use syntect::parsing::SyntaxSet;

// ThemeSet: Collection of highlighting themes
use syntect::highlighting::ThemeSet;

// HighlightLines: Main API for line-by-line highlighting
use syntect::easy::HighlightLines;

// Style: Contains foreground, background, and font_style
use syntect::highlighting::Style;

// Color: RGBA color from theme
use syntect::highlighting::Color;
// struct Color { r: u8, g: u8, b: u8, a: u8 }

// FontStyle: bitflags for bold, italic, underline
use syntect::highlighting::FontStyle;
```

### Key APIs

```rust
// Load built-in syntax definitions (includes ~60 languages)
let ss = SyntaxSet::load_defaults_newlines();

// Load built-in themes (includes ~10 themes)
let ts = ThemeSet::load_defaults();

// Find syntax by language token (from markdown fence)
let syntax = ss.find_syntax_by_token("rust")
    .unwrap_or_else(|| ss.find_syntax_plain_text());

// Get a theme by name
let theme = &ts.themes["base16-ocean.dark"];

// Create a highlighter
let mut highlighter = HighlightLines::new(syntax, theme);

// Highlight a line (returns styled regions)
let regions: Vec<(Style, &str)> = highlighter.highlight_line(line, &ss)?;

// Convert to 24-bit ANSI terminal escape sequences
use syntect::util::as_24_bit_terminal_escaped;
let escaped = as_24_bit_terminal_escaped(&regions[..], true);
```

### Built-in Themes

| Theme Name | Description |
|------------|-------------|
| `base16-ocean.dark` | Popular dark theme (recommended default) |
| `base16-ocean.light` | Light counterpart |
| `base16-eighties.dark` | Retro dark theme |
| `base16-mocha.dark` | Warm dark theme |
| `InspiredGitHub` | GitHub-like light theme |
| `Solarized (dark)` | Classic Solarized dark |
| `Solarized (light)` | Classic Solarized light |

### Supported Languages (Partial List)

Rust, Python, JavaScript, TypeScript, Go, C, C++, Java, Ruby, PHP, HTML, CSS, JSON, YAML, TOML, Markdown, SQL, Shell/Bash, and ~50 more via Sublime Text syntax definitions.

---

## 2. Integration Patterns from Existing Projects

### 2.1 bat (Terminal Syntax Highlighter)

**Repository:** https://github.com/sharkdp/bat

**Pattern:** Uses syntect as a library through `PrettyPrinter` struct.

**Key insights:**
- Handles terminals with/without truecolor support
- Recommends 24-bit terminals for best results
- Uses `fancy-regex` for WASM/Windows compatibility
- Lazy-loads syntaxes for fast startup

### 2.2 mdcat (Markdown Terminal Renderer)

**Repository:** https://github.com/swsnr/mdcat

**Pattern:** Integrates syntect specifically for code block highlighting in markdown.

**Key insights:**
- Uses pulldown-cmark for markdown parsing (same as glamour)
- Detects code block language from fence info
- Falls back to plain text for unknown languages
- Outputs ANSI escape sequences for terminal rendering

### 2.3 zola (Static Site Generator)

**Pattern:** Uses syntect for HTML output in static site generation.

**Key insights:**
- Caches syntax sets for performance
- Supports custom themes via configuration
- Uses CSS classes (not inline styles) for flexibility

### 2.4 delta (Git Diff Viewer)

**Pattern:** Line-by-line highlighting for diff output.

**Key insights:**
- Processes text incrementally
- Handles partial/incomplete code gracefully
- Integrates with terminal pager

---

## 3. Theme Color Mapping Strategy

### Syntect Style → Lipgloss Style Conversion

```rust
use syntect::highlighting::{Style as SynStyle, FontStyle as SynFontStyle, Color as SynColor};
use lipgloss::Style as LipStyle;

fn syntect_to_lipgloss(syn_style: SynStyle) -> LipStyle {
    let mut style = LipStyle::new();

    // Convert foreground color (RGBA → hex string)
    let fg = syn_style.foreground;
    if fg.a > 0 {
        let hex = format!("#{:02x}{:02x}{:02x}", fg.r, fg.g, fg.b);
        style = style.foreground(&hex);
    }

    // Convert background color (optional, often transparent in themes)
    let bg = syn_style.background;
    if bg.a > 0 {
        let hex = format!("#{:02x}{:02x}{:02x}", bg.r, bg.g, bg.b);
        style = style.background(&hex);
    }

    // Convert font style (bitflags)
    let font = syn_style.font_style;
    if font.contains(SynFontStyle::BOLD) {
        style = style.bold();
    }
    if font.contains(SynFontStyle::ITALIC) {
        style = style.italic();
    }
    if font.contains(SynFontStyle::UNDERLINE) {
        style = style.underline();
    }

    style
}
```

### Color Space Considerations

From syntect docs: "Because these numbers come directly from the theme, you might have to do your own color space conversion if you're outputting a different color space from the theme."

**Recommendation:** Themes typically use sRGB, and terminals expect sRGB. No conversion should be necessary for most use cases. Lipgloss handles ANSI escape sequence generation internally.

### Handling Transparency (Alpha Channel)

- Most theme colors have `a = 255` (fully opaque)
- Background colors may have `a = 0` (transparent)
- When `a = 0`, skip setting that color property in lipgloss

---

## 4. WASM Compatibility

### Current Status

**syntect supports WASM** when compiled with the `fancy-regex` feature instead of `onig`.

```toml
[dependencies]
syntect = { version = "5", default-features = false, features = ["default-fancy"] }
```

### Feature Flags

| Feature | Description | WASM Compatible |
|---------|-------------|-----------------|
| `default-onig` | Uses Oniguruma C library | No |
| `default-fancy` | Uses pure-Rust fancy-regex | Yes |
| `regex-syntax` | Syntax definitions only | Yes |
| `plist-load` | Load .tmTheme files | Yes |
| `bincode` | Binary serialization | Yes |

### WASM Considerations

1. **Binary Data:** Syntax definitions are embedded as binary data (~2MB). This is acceptable for WASM bundle size.

2. **No File I/O:** `load_defaults_*` methods work in WASM; file-based loading (`load_from_folder`) does not.

3. **Third-party bindings:** The [syntect-js](https://github.com/Menci/syntect-js) project provides proven WASM bindings.

### Recommendation

Use `default-fancy` feature from the start. This ensures:
- WASM compatibility for future web-based glamour rendering
- No C library dependencies (easier builds, especially on Windows)
- Slightly slower regex matching (acceptable for our use case)

---

## 5. Performance Considerations

### Loading Times (Approximate)

| Operation | Time |
|-----------|------|
| `SyntaxSet::load_defaults_newlines()` | ~23ms |
| `ThemeSet::load_defaults()` | ~5ms |
| First `highlight_line()` call | ~1ms |
| Subsequent `highlight_line()` | <0.1ms |

### Optimization Strategies

1. **Lazy Loading with LazyLock:**
   ```rust
   use std::sync::LazyLock;

   static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(|| {
       SyntaxSet::load_defaults_newlines()
   });

   static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(|| {
       ThemeSet::load_defaults()
   });
   ```

2. **Pre-serialized Dumps:** For extreme optimization, syntect supports pre-serializing syntax sets to binary blobs for faster loading.

3. **Caching Highlighters:** Reuse `HighlightLines` instances when highlighting multiple blocks of the same language.

### Memory Usage

- SyntaxSet: ~10MB in memory (all languages loaded)
- ThemeSet: ~2MB in memory (all themes loaded)
- Per-highlighter: ~1KB

This is acceptable for CLI/TUI applications.

---

## 6. Proposed API Design for Glamour

### Feature Flag

```toml
# crates/glamour/Cargo.toml

[features]
default = []
syntax-highlighting = ["syntect"]

[dependencies]
syntect = { version = "5", default-features = false, features = ["default-fancy"], optional = true }
```

### API Changes

```rust
// StyleCodeBlock gains a theme field (already exists)
pub struct StyleCodeBlock {
    pub block: StyleBlock,
    pub theme: Option<String>,  // e.g., "base16-ocean.dark"
}

// New Renderer option
impl Renderer {
    /// Enables syntax highlighting with the specified theme.
    /// Requires the `syntax-highlighting` feature.
    #[cfg(feature = "syntax-highlighting")]
    pub fn with_syntax_highlighting(mut self, enabled: bool) -> Self {
        self.options.syntax_highlighting = enabled;
        self
    }

    /// Sets the syntax highlighting theme.
    #[cfg(feature = "syntax-highlighting")]
    pub fn with_syntax_theme(mut self, theme: &str) -> Self {
        self.options.syntax_theme = Some(theme.to_string());
        self
    }
}
```

### Internal Implementation

```rust
#[cfg(feature = "syntax-highlighting")]
mod syntax {
    use std::sync::LazyLock;
    use syntect::parsing::SyntaxSet;
    use syntect::highlighting::ThemeSet;

    pub static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(|| {
        SyntaxSet::load_defaults_newlines()
    });

    pub static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(|| {
        ThemeSet::load_defaults()
    });

    pub fn highlight_code(code: &str, language: &str, theme_name: &str) -> String {
        let syntax = SYNTAX_SET.find_syntax_by_token(language)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        let theme = THEME_SET.themes.get(theme_name)
            .unwrap_or_else(|| &THEME_SET.themes["base16-ocean.dark"]);

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut output = String::new();

        for line in LinesWithEndings::from(code) {
            let regions = highlighter.highlight_line(line, &SYNTAX_SET).unwrap();
            for (style, text) in regions {
                let lipgloss_style = syntect_to_lipgloss(style);
                output.push_str(&lipgloss_style.render(text));
            }
        }

        output
    }
}
```

### Usage Example

```rust
// Without syntax highlighting (default)
let renderer = Renderer::new().with_style(Style::Dark);
let output = renderer.render("```rust\nfn main() {}\n```");

// With syntax highlighting (requires feature)
#[cfg(feature = "syntax-highlighting")]
let renderer = Renderer::new()
    .with_style(Style::Dark)
    .with_syntax_highlighting(true)
    .with_syntax_theme("base16-ocean.dark");
let output = renderer.render("```rust\nfn main() {}\n```");
```

---

## 7. Implementation Plan

### Phase 1: Add Dependency and Feature Flag
- Add syntect with `default-fancy` features
- Create feature flag for opt-in syntax highlighting
- Update Cargo.toml

### Phase 2: Core Implementation
- Create `syntax` module with lazy-loaded sets
- Implement `syntect_to_lipgloss()` conversion
- Implement `highlight_code()` function

### Phase 3: Integration with Renderer
- Modify `flush_code_block()` to optionally highlight
- Add renderer options for theme selection
- Handle unknown languages gracefully

### Phase 4: Testing
- Unit tests for color conversion
- Integration tests with various languages
- Conformance tests against Go glamour output

### Phase 5: Documentation
- Document feature flag usage
- List available themes
- Provide examples

---

## 8. Decision Record

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Library | syntect | De-facto standard, comprehensive language support |
| Regex Engine | fancy-regex | WASM compatible, no C dependencies |
| Default Theme | base16-ocean.dark | Popular, good contrast, matches Dark style |
| Feature Flag | `syntax-highlighting` | Opt-in to avoid binary size increase |
| Loading | LazyLock | Fast startup, load on first use |

---

## 9. References

- [syntect GitHub](https://github.com/trishume/syntect)
- [syntect docs.rs](https://docs.rs/syntect)
- [bat GitHub](https://github.com/sharkdp/bat)
- [mdcat GitHub](https://github.com/swsnr/mdcat)
- [Rust Markdown Syntax Highlighting Guide](https://bandarra.me/posts/Rust-Markdown-Syntax-Highlighting-A-Practical-Guide)
- [syntect-js (WASM bindings)](https://github.com/Menci/syntect-js)

---

## 10. Acceptance Criteria Checklist

- [x] Documentation covers all syntect APIs we'll use
- [x] Theme mapping strategy documented
- [x] WASM compatibility status confirmed (use `fancy-regex`)
- [x] API design proposal written
- [ ] Benchmark results (requires implementation spike)

---

*Document created: 2026-01-19*
*Bead: charmed_rust-hf0*
