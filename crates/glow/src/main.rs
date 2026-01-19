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

use std::io::Read;
use std::path::PathBuf;

use clap::{ArgAction, CommandFactory, Parser};
use glow::{Config, Reader};

#[derive(Debug, Parser)]
#[command(name = "glow", about = "Terminal-based markdown reader", version)]
struct Cli {
    /// Markdown file to render. Use "-" to read from stdin.
    path: Option<PathBuf>,

    /// Style theme (dark, light, ascii, pink, auto, no-tty)
    #[arg(short = 's', long, default_value = "dark")]
    style: String,

    /// Word wrap width (defaults to glamour's default if omitted)
    #[arg(short, long)]
    width: Option<usize>,

    /// Disable pager mode (not yet implemented)
    #[arg(long = "no-pager", action = ArgAction::SetTrue)]
    no_pager: bool,
}

fn main() {
    let cli = Cli::parse();

    let mut config = Config::new().style(cli.style).pager(!cli.no_pager);
    if let Some(width) = cli.width {
        config = config.width(width);
    }

    let reader = Reader::new(config);

    if let Some(path) = cli.path {
        if path.as_os_str() == "-" {
            let mut input = String::new();
            if let Err(err) = std::io::stdin().read_to_string(&mut input) {
                eprintln!("Error reading stdin: {err}");
                std::process::exit(1);
            }
            match reader.render_markdown(&input) {
                Ok(output) => print!("{output}"),
                Err(err) => {
                    eprintln!("Error rendering markdown: {err}");
                    std::process::exit(1);
                }
            }
        } else {
            match reader.read_file(&path) {
                Ok(output) => print!("{output}"),
                Err(err) => {
                    eprintln!("Error reading file: {err}");
                    std::process::exit(1);
                }
            }
        }
    } else {
        let mut cmd = Cli::command();
        let _ = cmd.print_help();
        println!();
    }
}
