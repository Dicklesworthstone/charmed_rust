# Glamour

A markdown rendering library for terminal applications, ported from the [Go glamour](https://github.com/charmbracelet/glamour) library.

Glamour transforms markdown into beautifully styled terminal output with:
- Styled headings, lists, and tables
- Code block formatting with optional syntax highlighting
- Link and image handling
- Customizable themes (Dark, Light, ASCII, Pink)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
glamour = "0.1"
```

## Basic Usage

```rust
use glamour::{render, Renderer, Style};

// Quick render with default dark style
let output = render("# Hello\n\nThis is **bold** text.", Style::Dark).unwrap();
println!("{}", output);

// Custom renderer with word wrap
let renderer = Renderer::new()
    .with_style(Style::Light)
    .with_word_wrap(80);
let output = renderer.render("# Heading\n\nParagraph text.");
```

## Table Rendering

Glamour supports Markdown tables out of the box. For advanced table rendering
APIs (parsing, low-level rendering, and styling), see:

- `crates/glamour/docs/tables/README.md`
- `crates/glamour/src/table.rs` (API docs)

## Syntax Highlighting

Glamour supports syntax highlighting for code blocks using [syntect](https://crates.io/crates/syntect).

### Enabling

Add the `syntax-highlighting` feature:

```toml
[dependencies]
glamour = { version = "0.1", features = ["syntax-highlighting"] }
```

> **Note**: This adds ~2MB to binary size due to embedded syntax definitions for ~60 languages.

### Basic Usage

```rust
use glamour::{render, Style};

let markdown = r#"
```rust
fn main() {
    println!("Hello, world!");
}
```
"#;

// Code blocks with language hints are automatically highlighted
let output = render(markdown, Style::Dark).unwrap();
println!("{}", output);
```

### Theme Selection

```rust
use glamour::{Renderer, StyleConfig};

// Using StyleConfig builder
let config = StyleConfig::default()
    .syntax_theme("Solarized (dark)")
    .with_line_numbers(true);

let renderer = Renderer::new()
    .with_style_config(config);

// Or modify at runtime
let mut renderer = Renderer::new();
renderer.set_syntax_theme("Solarized (light)").unwrap();
renderer.set_line_numbers(true);
```

### Available Themes

| Theme | Description |
|-------|-------------|
| `base16-ocean.dark` | Default, blue-toned dark theme |
| `base16-eighties.dark` | Retro 80s color palette |
| `base16-mocha.dark` | Warm brown-toned dark theme |
| `InspiredGitHub` | GitHub-style colors |
| `Solarized (dark)` | Popular dark theme |
| `Solarized (light)` | Light theme variant |

### Language Aliases

Map custom language identifiers:

```rust
use glamour::StyleConfig;

let config = StyleConfig::default()
    .language_alias("dockerfile", "docker")
    .language_alias("jsonc", "json")
    .language_alias("rs", "rust");
```

### Disabling for Specific Languages

```rust
use glamour::StyleConfig;

let config = StyleConfig::default()
    .disable_language("plaintext")
    .disable_language("text");
```

### Supported Languages

Over 60 languages are supported including:

- **Systems**: Rust, C, C++, Go, Assembly
- **Web**: JavaScript, TypeScript, HTML, CSS, JSON
- **Scripting**: Python, Ruby, Perl, Bash, PowerShell
- **JVM**: Java, Kotlin, Scala, Groovy
- **Markup**: Markdown, YAML, TOML, XML
- **Others**: SQL, GraphQL, Dockerfile, Makefile

Use `LanguageDetector::supported_languages()` for the full list.

## Built-in Styles

| Style | Description |
|-------|-------------|
| `Style::Dark` | Dark terminal background (default) |
| `Style::Light` | Light terminal background |
| `Style::Ascii` | ASCII-only, no special characters |
| `Style::Pink` | Pink accent colors |
| `Style::NoTty` | Plain output for non-terminals |
| `Style::Auto` | Auto-detect from terminal |

## API Reference

### Quick Functions

- `render(markdown, style)` - Render with built-in style
- `render_bytes(bytes, style)` - Render from bytes

### Renderer

```rust
let renderer = Renderer::new()
    .with_style(Style::Dark)
    .with_word_wrap(80)
    .with_style_config(custom_config);

let output = renderer.render("# Hello");
```

### StyleConfig

Full control over rendering styles including:
- Document, paragraph, blockquote styles
- Heading styles (H1-H6)
- List and code block styles
- Syntax highlighting configuration

## Feature Flags

| Feature | Description | Size Impact |
|---------|-------------|-------------|
| `syntax-highlighting` | Syntax highlighting via syntect | ~2MB |
| `serde` | Serialize/deserialize configs | Minimal |

## Terminal Compatibility

For best results:
- **TrueColor terminals**: iTerm2, Kitty, Alacritty, Windows Terminal
- **256-color terminals**: Most modern terminals
- **tmux/screen**: Set `TERM=xterm-256color` or `tmux-256color`

## License

MIT License - see LICENSE file for details.

## Credits

Port of [github.com/charmbracelet/glamour](https://github.com/charmbracelet/glamour) to Rust.
