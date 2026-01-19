//! Viewport Example
//!
//! This example demonstrates:
//! - Using the bubbles Viewport component for scrollable content
//! - Handling keyboard navigation (j/k, arrows, Page Up/Down)
//! - Loading and displaying large text content
//! - Displaying scroll position indicators
//!
//! Run with: `cargo run -p example-viewport`

#![forbid(unsafe_code)]

use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Program, quit};
use lipgloss::Style;

/// Sample content to display in the viewport.
const SAMPLE_CONTENT: &str = r#"
The Rust Programming Language
=============================

Rust is a multi-paradigm, general-purpose programming language that emphasizes
performance, type safety, and concurrency. It enforces memory safety, meaning
that all references point to valid memory, without requiring the use of
automated memory management techniques such as garbage collection.

Key Features
------------

1. Memory Safety Without Garbage Collection
   Rust achieves memory safety without runtime garbage collection through its
   ownership system. The compiler enforces strict rules about how memory can
   be accessed and when it must be freed.

2. Zero-Cost Abstractions
   Rust's abstractions impose no runtime overhead. Code written using Rust's
   high-level features compiles down to assembly that's as fast as hand-written
   low-level code.

3. Fearless Concurrency
   The ownership model prevents data races at compile time. Multiple threads
   can safely access shared data without the traditional problems of concurrent
   programming.

4. Pattern Matching
   Rust provides powerful pattern matching through the match expression and
   if-let constructs, making it easy to handle complex data structures.

5. Trait-based Generics
   Rust's trait system provides a form of generics that's both flexible and
   efficient. Traits define shared behavior in an abstract way.

6. Error Handling
   Rust distinguishes between recoverable and unrecoverable errors. The Result
   type is used for recoverable errors, and the panic! macro is used for
   unrecoverable errors.

The Ecosystem
-------------

- Cargo: Rust's build system and package manager
- Crates.io: The Rust package registry
- Rustfmt: Automatic code formatter
- Clippy: A collection of lints to catch common mistakes
- Rust Analyzer: An IDE-focused language server

Popular Use Cases
-----------------

* Systems programming
* Web servers and services
* Command-line tools
* WebAssembly applications
* Game development
* Embedded systems
* Blockchain and cryptocurrency

Getting Started
---------------

To install Rust, visit https://rustup.rs and follow the instructions for your
platform. The rustup tool manages Rust versions and associated tools.

Create your first project:

    $ cargo new hello_world
    $ cd hello_world
    $ cargo run

This will create a new Rust project and run the default "Hello, World!" program.

The Rust Community
------------------

The Rust community is known for being welcoming and helpful. Key resources:

* Official Documentation: https://doc.rust-lang.org
* The Rust Book: https://doc.rust-lang.org/book/
* Rust by Example: https://doc.rust-lang.org/rust-by-example/
* Users Forum: https://users.rust-lang.org
* Discord: https://discord.gg/rust-lang

Conclusion
----------

Rust combines low-level control over performance with high-level convenience.
Whether you're building a command-line tool, a web service, or an operating
system kernel, Rust provides the tools you need to write reliable, efficient
software.

Thank you for reading!
"#;

/// The main application model.
#[derive(bubbletea::Model)]
struct App {
    viewport: Viewport,
}

impl App {
    /// Create a new app with the sample content.
    fn new() -> Self {
        // Create viewport with default dimensions (will be resized)
        let mut viewport = Viewport::new(80, 20);
        viewport.set_content(SAMPLE_CONTENT);

        Self { viewport }
    }

    fn init(&self) -> Option<Cmd> {
        None
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle keyboard input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Runes => {
                    if let Some('q' | 'Q') = key.runes.first() {
                        return Some(quit());
                    }
                }
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                _ => {}
            }
        }

        // Forward to viewport for scrolling
        self.viewport.update(&msg);
        None
    }

    fn view(&self) -> String {
        let mut output = String::new();

        // Header
        let header_style = Style::new().bold().foreground("212");
        output.push_str(&format!(
            "\n  {}\n",
            header_style.render("Viewport Example - Scrollable Content")
        ));

        // Scroll indicator
        let indicator_style = Style::new().foreground("241");
        let y_offset = self.viewport.y_offset();
        let percent = if self.viewport.at_bottom() {
            100
        } else if y_offset == 0 {
            0
        } else {
            // Approximate percentage based on offset
            let total_lines = self.viewport.total_line_count();
            if total_lines > 0 {
                (y_offset * 100) / total_lines
            } else {
                0
            }
        };
        output.push_str(&format!(
            "  {}\n\n",
            indicator_style.render(&format!("Scroll: {}%", percent))
        ));

        // Viewport content with border
        let content = self.viewport.view();
        for line in content.lines() {
            output.push_str(&format!("  {line}\n"));
        }

        output.push('\n');

        // Help text
        let help_style = Style::new().foreground("241");
        output.push_str(&format!(
            "  {}\n",
            help_style.render("j/k or ↑/↓: scroll  f/b or PgDn/PgUp: page  q: quit")
        ));

        output
    }
}

fn main() -> anyhow::Result<()> {
    Program::new(App::new()).with_alt_screen().run()?;

    println!("Goodbye!");
    Ok(())
}
