# Examples

This document provides examples of using the glow library.

## Basic Rendering

Render a markdown file to a string:

```rust
use glow::{Config, Reader};

fn main() -> std::io::Result<()> {
    let reader = Reader::new(Config::default());
    let output = reader.read_file("README.md")?;
    println!("{output}");
    Ok(())
}
```

## Custom Configuration

Configure rendering options:

```rust
use glow::{Config, Reader};

fn main() -> std::io::Result<()> {
    let config = Config::new()
        .style("light")     // Use light theme
        .width(80)          // Wrap at 80 columns
        .pager(false);      // Disable pager

    let reader = Reader::new(config);
    let output = reader.read_file("document.md")?;
    println!("{output}");
    Ok(())
}
```

## Render Markdown String

Render markdown from a string instead of a file:

```rust
use glow::{Config, Reader};

fn main() -> std::io::Result<()> {
    let markdown = r#"
# Hello World

This is **bold** and *italic* text.

- Item 1
- Item 2
- Item 3

```rust
fn main() {
    println!("Hello, world!");
}
```
"#;

    let reader = Reader::new(Config::default());
    let output = reader.render_markdown(markdown)?;
    println!("{output}");
    Ok(())
}
```

## Document Stash

Save documents for quick access:

```rust
use glow::Stash;

fn main() {
    let mut stash = Stash::new();

    // Add documents to the stash
    stash.add("README.md");
    stash.add("docs/guide.md");
    stash.add("CHANGELOG.md");

    // List all stashed documents
    println!("Stashed documents:");
    for doc in stash.documents() {
        println!("  - {doc}");
    }
}
```

## File Browser Integration

Use the file browser with `bubbletea`:

```rust
use glow::browser::{BrowserConfig, FileBrowser, Entry, FileSelectedMsg};
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};

struct App {
    browser: FileBrowser,
    selected_file: Option<String>,
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Check if a file was selected
        if let Some(selected) = msg.downcast_ref::<FileSelectedMsg>() {
            self.selected_file = Some(selected.path.clone());
            return Some(bubbletea::quit());
        }

        // Handle key presses
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::CtrlC => return Some(bubbletea::quit()),
                _ => {}
            }
        }

        // Forward to browser
        self.browser.update(msg);
        None
    }

    fn view(&self) -> String {
        self.browser.view()
    }
}
```

## Multiple Styles

Try different styles:

```rust
use glow::{Config, Reader};

fn main() -> std::io::Result<()> {
    let markdown = "# Test\n\nHello, world!";
    let styles = ["dark", "light", "ascii", "pink"];

    for style in styles {
        println!("=== {} style ===", style);
        let config = Config::new().style(style);
        let reader = Reader::new(config);
        let output = reader.render_markdown(markdown)?;
        println!("{output}\n");
    }

    Ok(())
}
```

## Error Handling

Handle potential errors gracefully:

```rust
use glow::{Config, Reader};
use std::io;

fn render_file(path: &str) -> io::Result<String> {
    let config = Config::new().style("dark");
    let reader = Reader::new(config);
    reader.read_file(path)
}

fn main() {
    match render_file("README.md") {
        Ok(output) => println!("{output}"),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("File not found");
        }
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!("Permission denied");
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
```

## GitHub Integration

Fetch READMEs from GitHub (requires `github` feature):

```rust,ignore
use glow::github::fetch_readme;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch a repository README
    let readme = fetch_readme("charmbracelet", "glow").await?;

    // Render it
    let reader = Reader::new(Config::default());
    let output = reader.render_markdown(&readme)?;
    println!("{output}");

    Ok(())
}
```

## CLI Wrapper

Create a simple CLI wrapper:

```rust
use glow::{Config, Reader};
use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.md>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let config = Config::new().style("dark").pager(true);
    let reader = Reader::new(config);
    let output = reader.read_file(path)?;
    println!("{output}");

    Ok(())
}
```

## Streaming Large Files

For very large files, consider streaming:

```rust
use glow::{Config, Reader};
use std::io::{BufRead, BufReader};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let config = Config::new()
        .style("dark")
        .width(80);
    let reader = Reader::new(config);

    let file = File::open("large-document.md")?;
    let buf_reader = BufReader::new(file);

    let mut chunk = String::new();
    for line in buf_reader.lines() {
        chunk.push_str(&line?);
        chunk.push('\n');

        // Render in chunks of 100 lines
        if chunk.lines().count() >= 100 {
            let output = reader.render_markdown(&chunk)?;
            print!("{output}");
            chunk.clear();
        }
    }

    // Render remaining content
    if !chunk.is_empty() {
        let output = reader.render_markdown(&chunk)?;
        print!("{output}");
    }

    Ok(())
}
```
