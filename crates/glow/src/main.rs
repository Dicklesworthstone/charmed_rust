#![forbid(unsafe_code)]

//! # Glow CLI
//!
//! Terminal-based markdown reader.
//!
//! ## Usage
//!
//! ```bash
//! glow README.md           # Render a file
//! glow                     # Browse local files
//! glow github.com/user/repo # Read GitHub README
//! ```

use glow::{Config, Reader};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let config = Config::new();
    let reader = Reader::new(config);

    if let Some(path) = args.get(1) {
        match reader.read_file(path) {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("Error reading file: {e}"),
        }
    } else {
        println!("Glow - Terminal Markdown Reader");
        println!();
        println!("Usage: glow [file]");
        println!();
        println!("Arguments:");
        println!("  [file]  Markdown file to render");
    }
}
