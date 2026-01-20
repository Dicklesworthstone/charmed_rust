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

/// Input mode for the pager.
#[derive(Debug, Clone, PartialEq)]
enum InputMode {
    /// Normal navigation mode.
    Normal,
    /// Help overlay displayed.
    Help,
    /// Search input mode.
    Search { forward: bool },
}

/// Search state for incremental search.
#[derive(Debug, Clone, Default)]
struct SearchState {
    /// Current search query.
    query: String,
    /// Line indices of matches.
    matches: Vec<usize>,
    /// Current match index.
    current: usize,
}

/// Pager model for scrollable markdown viewing.
struct Pager {
    viewport: Viewport,
    content: String,
    /// Content lines for search.
    lines: Vec<String>,
    title: String,
    ready: bool,
    mode: InputMode,
    search: SearchState,
    status_style: Style,
    help_style: Style,
    search_style: Style,
    match_style: Style,
}

impl Pager {
    fn new(content: String, title: String) -> Self {
        let lines: Vec<String> = content.lines().map(String::from).collect();
        Self {
            viewport: Viewport::new(80, 24),
            content,
            lines,
            title,
            ready: false,
            mode: InputMode::Normal,
            search: SearchState::default(),
            status_style: Style::new().foreground("#7D56F4").bold(),
            help_style: Style::new().foreground("#626262"),
            search_style: Style::new().foreground("#FFCC00").bold(),
            match_style: Style::new().foreground("#00FF00"),
        }
    }

    fn status_bar(&self) -> String {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let percent = (self.viewport.scroll_percent() * 100.0) as usize;
        let line = self.viewport.y_offset() + 1;
        let total = self.viewport.total_line_count();

        let info = format!("  {} · {}% · {}/{} ", self.title, percent, line, total);

        // Show search info if there's a query
        let search_info = if self.search.query.is_empty() {
            String::new()
        } else if self.search.matches.is_empty() {
            format!(" [no matches for \"{}\"]", self.search.query)
        } else {
            format!(
                " [{}/{} \"{}\"]",
                self.search.current + 1,
                self.search.matches.len(),
                self.search.query
            )
        };

        let help = match self.mode {
            InputMode::Normal => "  q quit · h help · / search · j/k scroll · g/G top/bottom ",
            InputMode::Help => "  Press any key to close help ",
            InputMode::Search { .. } => "  Enter confirm · Esc cancel · n/N next/prev ",
        };

        format!(
            "{}{}\n{}",
            self.status_style.render(&info),
            self.match_style.render(&search_info),
            self.help_style.render(help)
        )
    }

    fn search_input_bar(&self) -> String {
        let prefix = match self.mode {
            InputMode::Search { forward: true } => "/",
            InputMode::Search { forward: false } => "?",
            _ => "",
        };
        self.search_style
            .render(&format!("{}{}_", prefix, self.search.query))
    }

    #[allow(clippy::unused_self)]
    fn help_overlay(&self, width: usize, height: usize) -> String {
        let help_text = [
            "",
            "  Keyboard Navigation",
            "  ───────────────────",
            "  j/↓        Scroll down one line",
            "  k/↑        Scroll up one line",
            "  d/Ctrl+d   Scroll down half page",
            "  u/Ctrl+u   Scroll up half page",
            "  f/Space    Scroll down full page",
            "  b          Scroll up full page",
            "  g          Go to top",
            "  G          Go to bottom",
            "",
            "  Search",
            "  ──────",
            "  /          Search forward",
            "  ?          Search backward",
            "  n          Next match",
            "  N          Previous match",
            "",
            "  Other",
            "  ─────",
            "  h          Show this help",
            "  q/Esc      Quit",
            "",
            "  Press any key to close",
        ];

        let box_width = 40;
        let box_height = help_text.len();
        let start_x = width.saturating_sub(box_width) / 2;
        let start_y = height.saturating_sub(box_height) / 2;

        let border_style = Style::new().foreground("#7D56F4");
        let text_style = Style::new().foreground("#FFFFFF");

        let mut lines: Vec<String> = Vec::new();

        // Add top padding
        for _ in 0..start_y {
            lines.push(String::new());
        }

        // Top border
        let top_border = format!(
            "{}╭{}╮",
            " ".repeat(start_x),
            "─".repeat(box_width.saturating_sub(2))
        );
        lines.push(border_style.render(&top_border));

        // Content lines
        for text in &help_text {
            let padded = format!("{:width$}", text, width = box_width - 4);
            let line = format!("{}│ {} │", " ".repeat(start_x), padded);
            lines.push(text_style.render(&line));
        }

        // Bottom border
        let bottom_border = format!(
            "{}╰{}╯",
            " ".repeat(start_x),
            "─".repeat(box_width.saturating_sub(2))
        );
        lines.push(border_style.render(&bottom_border));

        lines.join("\n")
    }

    fn perform_search(&mut self) {
        self.search.matches.clear();
        self.search.current = 0;

        if self.search.query.is_empty() {
            return;
        }

        let query_lower = self.search.query.to_lowercase();
        for (i, line) in self.lines.iter().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                self.search.matches.push(i);
            }
        }
    }

    fn goto_next_match(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        self.search.current = (self.search.current + 1) % self.search.matches.len();
        let line = self.search.matches[self.search.current];
        self.viewport.set_y_offset(line);
    }

    fn goto_prev_match(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        if self.search.current == 0 {
            self.search.current = self.search.matches.len() - 1;
        } else {
            self.search.current -= 1;
        }
        let line = self.search.matches[self.search.current];
        self.viewport.set_y_offset(line);
    }

    fn goto_first_match_from_current(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        let current_line = self.viewport.y_offset();
        // Find first match at or after current position
        for (i, &line) in self.search.matches.iter().enumerate() {
            if line >= current_line {
                self.search.current = i;
                self.viewport.set_y_offset(line);
                return;
            }
        }
        // Wrap to beginning
        self.search.current = 0;
        self.viewport.set_y_offset(self.search.matches[0]);
    }

    fn goto_last_match_before_current(&mut self) {
        if self.search.matches.is_empty() {
            return;
        }
        let current_line = self.viewport.y_offset();
        let mut last_match = None;
        for (i, &line) in self.search.matches.iter().enumerate() {
            if line <= current_line {
                last_match = Some((i, line));
            } else {
                break;
            }
        }
        if let Some((i, line)) = last_match {
            self.search.current = i;
            self.viewport.set_y_offset(line);
            return;
        }
        self.search.current = self.search.matches.len() - 1;
        let line = self.search.matches[self.search.current];
        self.viewport.set_y_offset(line);
    }
}

impl Model for Pager {
    fn init(&self) -> Option<Cmd> {
        // Request window size on startup
        Some(bubbletea::window_size())
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle window resize
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            // Reserve 2 lines for status bar (or 3 in search mode)
            let reserve = if matches!(self.mode, InputMode::Search { .. }) {
                3
            } else {
                2
            };
            let height = (size.height as usize).saturating_sub(reserve);
            self.viewport = Viewport::new(size.width as usize, height);
            self.viewport.set_content(&self.content);
            self.ready = true;
            return None;
        }

        // Handle key input based on mode
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match &self.mode {
                InputMode::Help => {
                    // Any key closes help
                    self.mode = InputMode::Normal;
                    return None;
                }
                InputMode::Search { forward } => {
                    let forward = *forward;
                    match key.key_type {
                        KeyType::Esc => {
                            // Cancel search
                            self.mode = InputMode::Normal;
                            return None;
                        }
                        KeyType::Enter => {
                            // Confirm search and go to first match
                            self.perform_search();
                            if forward {
                                self.goto_first_match_from_current();
                            } else {
                                // For backward search, find last match before current
                                self.goto_last_match_before_current();
                            }
                            self.mode = InputMode::Normal;
                            return None;
                        }
                        KeyType::Backspace => {
                            self.search.query.pop();
                            self.perform_search();
                            return None;
                        }
                        KeyType::Runes => {
                            // Add characters to search query
                            for c in &key.runes {
                                self.search.query.push(*c);
                            }
                            self.perform_search();
                            return None;
                        }
                        _ => return None,
                    }
                }
                InputMode::Normal => {
                    // Normal mode key handling
                    match key.key_type {
                        KeyType::CtrlC => return Some(quit()),
                        KeyType::Esc => {
                            // Clear search on Esc, or quit if no search
                            if !self.search.query.is_empty() {
                                self.search.query.clear();
                                self.search.matches.clear();
                                return None;
                            }
                            return Some(quit());
                        }
                        KeyType::Runes => match key.runes.as_slice() {
                            ['q'] => return Some(quit()),
                            ['g'] => {
                                self.viewport.goto_top();
                                return None;
                            }
                            ['G'] => {
                                self.viewport.goto_bottom();
                                return None;
                            }
                            ['h' | '?'] if self.search.query.is_empty() => {
                                self.mode = InputMode::Help;
                                return None;
                            }
                            ['/'] => {
                                self.search.query.clear();
                                self.mode = InputMode::Search { forward: true };
                                return None;
                            }
                            ['?'] => {
                                self.search.query.clear();
                                self.mode = InputMode::Search { forward: false };
                                return None;
                            }
                            ['n'] => {
                                self.goto_next_match();
                                return None;
                            }
                            ['N'] => {
                                self.goto_prev_match();
                                return None;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }

        // Delegate to viewport for navigation keys (only in normal mode)
        if self.mode == InputMode::Normal {
            self.viewport.update(&msg);
        }
        None
    }

    fn view(&self) -> String {
        if !self.ready {
            return "Loading...".to_string();
        }

        match &self.mode {
            InputMode::Help => self.help_overlay(self.viewport.width, self.viewport.height + 2),
            InputMode::Search { .. } => {
                format!(
                    "{}\n{}\n{}",
                    self.viewport.view(),
                    self.search_input_bar(),
                    self.status_bar()
                )
            }
            InputMode::Normal => {
                format!("{}\n{}", self.viewport.view(), self.status_bar())
            }
        }
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
