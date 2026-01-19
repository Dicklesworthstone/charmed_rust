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

use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, WindowSizeMsg, quit};
use clap::{ArgAction, CommandFactory, Parser};
use glow::{Config, Reader};
use lipgloss::Style;

#[derive(Debug, Parser)]
#[command(name = "glow", about = "Terminal-based markdown reader", version)]
struct Cli {
    /// Markdown file to render. Use "-" to read from stdin.
    path: Option<PathBuf>,

    /// Style theme (dark, light, ascii, pink, auto, no-tty)
    #[arg(short = 's', long, default_value = "dark")]
    style: String,

    /// Word wrap width (defaults to terminal width if omitted)
    #[arg(short, long)]
    width: Option<usize>,

    /// Disable pager mode (print to stdout and exit)
    #[arg(long = "no-pager", action = ArgAction::SetTrue)]
    no_pager: bool,
}

/// Pager model for scrollable markdown viewing.
struct Pager {
    viewport: Viewport,
    content: String,
    title: String,
    ready: bool,
    status_style: Style,
    help_style: Style,
}

impl Pager {
    fn new(content: String, title: String) -> Self {
        Self {
            viewport: Viewport::new(80, 24),
            content,
            title,
            ready: false,
            status_style: Style::new().foreground("#7D56F4").bold(),
            help_style: Style::new().foreground("#626262"),
        }
    }

    fn status_bar(&self) -> String {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let percent = (self.viewport.scroll_percent() * 100.0) as usize;
        let line = self.viewport.y_offset() + 1;
        let total = self.viewport.total_line_count();

        let info = format!("  {} · {}% · {}/{} ", self.title, percent, line, total);
        let help = "  q quit · ↑/↓ scroll · pgup/pgdn page ";

        format!(
            "{}\n{}",
            self.status_style.render(&info),
            self.help_style.render(help)
        )
    }
}

impl Model for Pager {
    fn init(&self) -> Option<Cmd> {
        // Request window size on startup
        Some(bubbletea::window_size())
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle window resize
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            // Reserve 2 lines for status bar
            let height = (size.height as usize).saturating_sub(2);
            self.viewport = Viewport::new(size.width as usize, height);
            self.viewport.set_content(&self.content);
            self.ready = true;
            return None;
        }

        // Handle key input
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::CtrlC | KeyType::Esc => return Some(quit()),
                KeyType::Runes if key.runes == vec!['q'] => return Some(quit()),
                KeyType::Runes if key.runes == vec!['g'] => {
                    self.viewport.goto_top();
                    return None;
                }
                KeyType::Runes if key.runes == vec!['G'] => {
                    self.viewport.goto_bottom();
                    return None;
                }
                _ => {}
            }
        }

        // Delegate to viewport for navigation keys
        self.viewport.update(&msg);
        None
    }

    fn view(&self) -> String {
        if !self.ready {
            return "Loading...".to_string();
        }

        format!("{}\n{}", self.viewport.view(), self.status_bar())
    }
}

fn main() {
    let cli = Cli::parse();

    let mut config = Config::new().style(cli.style.clone()).pager(!cli.no_pager);
    if let Some(width) = cli.width {
        config = config.width(width);
    }

    let reader = Reader::new(config);

    if let Some(path) = cli.path {
        // Read content from stdin or file
        let (content, title) = if path.as_os_str() == "-" {
            let mut input = String::new();
            if let Err(err) = std::io::stdin().read_to_string(&mut input) {
                eprintln!("Error reading stdin: {err}");
                std::process::exit(1);
            }
            (input, "stdin".to_string())
        } else {
            let title = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("markdown")
                .to_string();
            match std::fs::read_to_string(&path) {
                Ok(content) => (content, title),
                Err(err) => {
                    eprintln!("Error reading file: {err}");
                    std::process::exit(1);
                }
            }
        };

        // Render markdown
        let rendered = match reader.render_markdown(&content) {
            Ok(output) => output,
            Err(err) => {
                eprintln!("Error rendering markdown: {err}");
                std::process::exit(1);
            }
        };

        // If no-pager mode, just print and exit
        if cli.no_pager {
            print!("{rendered}");
            return;
        }

        // Run TUI pager
        let pager = Pager::new(rendered, title);
        if let Err(err) = Program::new(pager)
            .with_alt_screen()
            .with_mouse_cell_motion()
            .run()
        {
            eprintln!("Error running pager: {err}");
            std::process::exit(1);
        }
    } else {
        let mut cmd = Cli::command();
        let _ = cmd.print_help();
        println!();
    }
}
