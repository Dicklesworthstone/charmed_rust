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
use std::process::Command as ProcessCommand;

use bubbles::viewport::Viewport;
use bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, Program, WindowSizeMsg, quit};
use clap::{ArgAction, Parser};
use glow::browser::{BrowserConfig, FileBrowser, FileSelectedMsg};
#[cfg(feature = "github")]
use glow::github::{FetcherConfig, GitHubFetcher, RepoRef};
use glow::{Config, Reader};
use lipgloss::Style;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
#[command(name = "glow", about = "Terminal-based markdown reader", version)]
struct Cli {
    /// Markdown file, URL, or GitHub repo to render. Use "-" to read from stdin.
    path: Option<String>,

    /// Style theme (dark, light, dracula, ascii, pink, auto, no-tty)
    #[arg(short = 's', long, default_value = "dark")]
    style: String,

    /// Word wrap width (defaults to terminal width if omitted)
    #[arg(short, long)]
    width: Option<usize>,

    /// Disable pager mode (print to stdout and exit)
    #[arg(long = "no-pager", action = ArgAction::SetTrue)]
    no_pager: bool,

    /// Show all files including hidden (for file browser mode)
    #[arg(short = 'a', long)]
    all: bool,

    /// Show line numbers in code blocks
    #[arg(short = 'l', long = "line-numbers")]
    line_numbers: bool,

    /// Enable mouse support
    #[arg(short = 'm', long)]
    mouse: bool,

    /// Preserve newlines in markdown output
    #[arg(long = "preserve-new-lines")]
    preserve_new_lines: bool,
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
    /// Original markdown source (for reload/editor).
    source_markdown: String,
    /// Content lines for search.
    lines: Vec<String>,
    title: String,
    /// Source path or URL (for reload/editor).
    source_path: Option<String>,
    ready: bool,
    mode: InputMode,
    search: SearchState,
    status_style: Style,
    help_style: Style,
    search_style: Style,
    match_style: Style,
    /// Whether mouse support is enabled.
    mouse_enabled: bool,
    /// Render configuration, used to re-render after the source is edited.
    config: Config,
    /// When opened from the file browser, `q`/`esc` return to it instead of
    /// quitting.
    return_to_browser: bool,
}

/// Sent by a pager opened from the file browser to go back to the list.
struct BackToBrowserMsg;

/// Sent when the external editor launched with `e` exits.
struct EditorClosedMsg;

impl Pager {
    fn new(
        content: String,
        source_markdown: String,
        title: String,
        source_path: Option<String>,
    ) -> Self {
        let lines: Vec<String> = content.lines().map(String::from).collect();
        Self {
            viewport: Viewport::new(80, 24),
            content,
            source_markdown,
            lines,
            title,
            source_path,
            ready: false,
            mode: InputMode::Normal,
            search: SearchState::default(),
            status_style: Style::new().foreground("#7D56F4").bold(),
            help_style: Style::new().foreground("#626262"),
            search_style: Style::new().foreground("#FFCC00").bold(),
            match_style: Style::new().foreground("#00FF00"),
            mouse_enabled: false,
            config: Config::new(),
            return_to_browser: false,
        }
    }

    /// Quits, or returns to the file browser when opened from it.
    fn leave(&self) -> Cmd {
        if self.return_to_browser {
            Cmd::new(|| Message::new(BackToBrowserMsg))
        } else {
            quit()
        }
    }

    fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    const fn with_mouse(mut self, enabled: bool) -> Self {
        self.mouse_enabled = enabled;
        self
    }

    /// Copies content to clipboard using system commands.
    fn copy_to_clipboard(&self) -> bool {
        // Try different clipboard commands based on platform
        #[cfg(target_os = "macos")]
        {
            if let Ok(mut child) = ProcessCommand::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(self.source_markdown.as_bytes());
                }
                return child.wait().is_ok();
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Try xclip first, then xsel
            for cmd in &["xclip", "xsel"] {
                if let Ok(mut child) = ProcessCommand::new(cmd)
                    .args(["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        use std::io::Write;
                        let _ = stdin.write_all(self.source_markdown.as_bytes());
                    }
                    if child.wait().is_ok() {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Returns the local file backing this pager, if it can be edited.
    fn editable_path(&self) -> Option<&str> {
        let path = self.source_path.as_deref()?;
        // Only local files can be opened in an editor.
        if path.starts_with("http") || path.contains("github.com") {
            return None;
        }
        Some(path)
    }

    /// Opens the source in `$EDITOR` (falling back to `$VISUAL`, then `vi`).
    ///
    /// The TUI hands the terminal to the editor and reloads the document
    /// when the editor exits.
    fn open_in_editor(&self) -> Option<Cmd> {
        let path = self.editable_path()?;
        let editor = std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "vi".to_string());
        // `$EDITOR` may carry arguments, e.g. "code --wait".
        let mut parts = editor.split_whitespace();
        let program = parts.next()?;
        let mut command = ProcessCommand::new(program);
        command.args(parts).arg(path);
        Some(bubbletea::exec_process(command, |_| {
            Some(Message::new(EditorClosedMsg))
        }))
    }

    /// Re-reads and re-renders the source file after it was edited.
    fn reload(&mut self) {
        let Some(path) = self.editable_path() else {
            return;
        };
        let Ok(markdown) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(rendered) = Reader::new(self.config.clone()).render_markdown(&markdown) else {
            return;
        };
        self.source_markdown = markdown;
        self.lines = rendered.lines().map(String::from).collect();
        self.content = rendered;
        let offset = self.viewport.y_offset();
        self.viewport.set_content(&self.content);
        self.viewport.set_y_offset(offset);
        self.perform_search();
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
            InputMode::Normal => "  q quit · h help · / search · c copy · e edit · j/k scroll ",
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
            "  Actions",
            "  ───────",
            "  c          Copy source to clipboard",
            "  e          Open in $EDITOR",
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

        if msg.is::<EditorClosedMsg>() {
            self.reload();
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
                            self.viewport.height += 1;
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
                            self.viewport.height += 1;
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
                            return Some(self.leave());
                        }
                        KeyType::Runes => match key.runes.as_slice() {
                            ['q'] => return Some(self.leave()),
                            ['g'] => {
                                self.viewport.goto_top();
                                return None;
                            }
                            ['G'] => {
                                self.viewport.goto_bottom();
                                return None;
                            }
                            ['h'] if self.search.query.is_empty() => {
                                self.mode = InputMode::Help;
                                return None;
                            }
                            ['/'] => {
                                self.search.query.clear();
                                self.mode = InputMode::Search { forward: true };
                                self.viewport.height = self.viewport.height.saturating_sub(1);
                                return None;
                            }
                            ['?'] => {
                                self.search.query.clear();
                                self.mode = InputMode::Search { forward: false };
                                self.viewport.height = self.viewport.height.saturating_sub(1);
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
                            ['c'] => {
                                // Copy markdown source to clipboard
                                self.copy_to_clipboard();
                                return None;
                            }
                            ['e'] => {
                                // Open in editor (suspends TUI, reloads on exit)
                                return self.open_in_editor();
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

/// Top-level TUI when glow is started on a directory: a markdown file
/// browser that opens documents in the pager.
struct App {
    browser: FileBrowser,
    pager: Option<Pager>,
    config: Config,
    mouse: bool,
    size: Option<WindowSizeMsg>,
    error: Option<String>,
}

impl App {
    /// Lines used by the browser's header, filter bar, and footer.
    const BROWSER_CHROME: usize = 6;

    const fn new(browser: FileBrowser, config: Config, mouse: bool) -> Self {
        Self {
            browser,
            pager: None,
            config,
            mouse,
            size: None,
            error: None,
        }
    }

    fn open(&mut self, path: &std::path::Path) {
        let markdown = match std::fs::read_to_string(path) {
            Ok(m) => m,
            Err(err) => {
                self.error = Some(format!("Error reading {}: {err}", path.display()));
                return;
            }
        };
        let rendered = match Reader::new(self.config.clone()).render_markdown(&markdown) {
            Ok(r) => r,
            Err(err) => {
                self.error = Some(format!("Error rendering {}: {err}", path.display()));
                return;
            }
        };
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("markdown")
            .to_string();
        let mut pager = Pager::new(
            rendered,
            markdown,
            title,
            Some(path.to_string_lossy().into_owned()),
        )
        .with_mouse(self.mouse)
        .with_config(self.config.clone());
        pager.return_to_browser = true;
        if let Some(size) = self.size {
            pager.update(Message::new(size));
        }
        self.error = None;
        self.pager = Some(pager);
    }
}

impl Model for App {
    fn init(&self) -> Option<Cmd> {
        bubbletea::batch(vec![self.browser.init(), Some(bubbletea::window_size())])
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>() {
            self.size = Some(*size);
            self.browser
                .set_height((size.height as usize).saturating_sub(Self::BROWSER_CHROME));
        }

        if let Some(pager) = &mut self.pager {
            if msg.is::<BackToBrowserMsg>() {
                self.pager = None;
                return None;
            }
            return pager.update(msg);
        }

        if let Some(selected) = msg.downcast_ref::<FileSelectedMsg>() {
            let path = selected.path.clone();
            self.open(&path);
            return None;
        }

        if let Some(key) = msg.downcast_ref::<KeyMsg>()
            && !self.browser.is_filter_mode()
        {
            match key.key_type {
                KeyType::CtrlC => return Some(quit()),
                KeyType::Esc if self.browser.filter().is_empty() => return Some(quit()),
                KeyType::Runes if key.runes.as_slice() == ['q'] => return Some(quit()),
                _ => {}
            }
        }

        self.browser.update(msg)
    }

    fn view(&self) -> String {
        if let Some(pager) = &self.pager {
            return pager.view();
        }
        let view = self.browser.view();
        match &self.error {
            Some(err) => format!("{view}\n{}", Style::new().foreground("#FF5F87").render(err)),
            None => view,
        }
    }
}

/// Runs the file browser TUI rooted at `dir`.
fn run_browser(dir: &std::path::Path, config: Config, cli: &Cli) {
    let browser_config = BrowserConfig {
        show_hidden: cli.all,
        ..BrowserConfig::default()
    };
    let browser = match FileBrowser::with_directory(dir, browser_config) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("Error opening directory: {err}");
            std::process::exit(1);
        }
    };
    let mut program = Program::new(App::new(browser, config, cli.mouse)).with_alt_screen();
    if cli.mouse {
        program = program.with_mouse_cell_motion();
    }
    if let Err(err) = program.run() {
        eprintln!("Error running browser: {err}");
        std::process::exit(1);
    }
}

/// Returns the directory to browse: the current one when no path is given,
/// or the path itself when it is a directory.
fn browse_target(path: Option<&str>) -> Option<PathBuf> {
    match path {
        None => Some(PathBuf::from(".")),
        Some(p) if PathBuf::from(p).is_dir() => Some(PathBuf::from(p)),
        Some(_) => None,
    }
}

/// Prints every markdown file under `dir` (recursively), one per line.
fn list_markdown_files(dir: &std::path::Path, show_hidden: bool) {
    let config = BrowserConfig {
        show_hidden,
        recursive: true,
        ..BrowserConfig::default()
    };
    let mut browser = FileBrowser::with_directory(dir, config).unwrap_or_else(|err| {
        eprintln!("Error opening directory: {err}");
        std::process::exit(1);
    });
    if let Err(err) = browser.scan() {
        eprintln!("Error scanning directory: {err}");
        std::process::exit(1);
    }
    for entry in browser.entries() {
        println!("{}", entry.path.display());
    }
}

/// Determines the source type from a path string.
enum Source {
    Stdin,
    File(PathBuf),
    #[cfg(feature = "github")]
    GitHub(RepoRef),
    #[cfg(feature = "github")]
    Url(String),
}

impl Source {
    fn parse(path: &str) -> Self {
        if path == "-" {
            return Self::Stdin;
        }

        // Check for GitHub repo references
        #[cfg(feature = "github")]
        {
            let is_github_pattern = path.contains("github.com")
                || path.starts_with("git@github.com")
                || (path.contains('/') && !path.contains('.') && !PathBuf::from(path).exists());

            if is_github_pattern && let Ok(repo) = RepoRef::parse(path) {
                return Self::GitHub(repo);
            }

            // Check for URL (requires github feature for reqwest)
            if path.starts_with("http://") || path.starts_with("https://") {
                return Self::Url(path.to_string());
            }
        }

        Self::File(PathBuf::from(path))
    }
}

fn main() {
    let cli = Cli::parse();

    let mut config = Config::new()
        .style(cli.style.clone())
        .pager(!cli.no_pager)
        .line_numbers(cli.line_numbers)
        .preserve_newlines(cli.preserve_new_lines);
    if let Some(width) = cli.width {
        config = config.width(width);
    }

    let reader = Reader::new(config.clone());

    // A directory (or no argument at all) opens the markdown file browser.
    if let Some(dir) = browse_target(cli.path.as_deref()) {
        if cli.no_pager || !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
            // Non-interactive: list markdown files instead of starting a TUI.
            list_markdown_files(&dir, cli.all);
        } else {
            run_browser(&dir, config, &cli);
        }
        return;
    }

    if let Some(path) = cli.path {
        let source = Source::parse(&path);

        // Read content based on source type
        let (content, title, source_path) = match source {
            Source::Stdin => {
                let mut input = String::new();
                if let Err(err) = std::io::stdin().read_to_string(&mut input) {
                    eprintln!("Error reading stdin: {err}");
                    std::process::exit(1);
                }
                (input, "stdin".to_string(), None)
            }
            Source::File(path) => {
                let title = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("markdown")
                    .to_string();
                let source = path.to_string_lossy().to_string();
                match std::fs::read_to_string(&path) {
                    Ok(content) => (content, title, Some(source)),
                    Err(err) => {
                        eprintln!("Error reading file: {err}");
                        std::process::exit(1);
                    }
                }
            }
            #[cfg(feature = "github")]
            Source::GitHub(repo) => {
                let fetcher = GitHubFetcher::new(FetcherConfig::default());
                let title = format!("{}/{}", repo.owner, repo.name);
                match fetcher.fetch_readme(&repo) {
                    Ok(content) => (content, title, None),
                    Err(err) => {
                        eprintln!("Error fetching README: {err}");
                        std::process::exit(1);
                    }
                }
            }
            #[cfg(feature = "github")]
            Source::Url(url) => {
                // Simple URL fetching using reqwest (blocking)
                match reqwest::blocking::get(&url) {
                    Ok(resp) => match resp.text() {
                        Ok(content) => {
                            let title = url.rsplit('/').next().unwrap_or("markdown").to_string();
                            (content, title, Some(url))
                        }
                        Err(err) => {
                            eprintln!("Error reading URL response: {err}");
                            std::process::exit(1);
                        }
                    },
                    Err(err) => {
                        eprintln!("Error fetching URL: {err}");
                        std::process::exit(1);
                    }
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
        let pager = Pager::new(rendered, content, title, source_path)
            .with_mouse(cli.mouse)
            .with_config(reader.config().clone());
        let mut program = Program::new(pager).with_alt_screen();

        if cli.mouse {
            program = program.with_mouse_cell_motion();
        }

        if let Err(err) = program.run() {
            eprintln!("Error running pager: {err}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir_with_doc() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "glow-app-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        std::fs::write(dir.join("doc.md"), "# Hello\n\nWorld\n").expect("write doc");
        dir
    }

    fn app_for(dir: &std::path::Path) -> App {
        let browser =
            FileBrowser::with_directory(dir, BrowserConfig::default()).expect("browser opens");
        let mut app = App::new(browser, Config::new().style("ascii"), false);
        app.update(Message::new(WindowSizeMsg {
            width: 80,
            height: 24,
        }));
        app
    }

    fn key(c: char) -> Message {
        Message::new(KeyMsg::from_char(c))
    }

    fn run(cmd: Option<Cmd>) -> Option<Message> {
        cmd.and_then(Cmd::execute)
    }

    #[test]
    fn browse_target_defaults_to_current_dir() {
        assert_eq!(browse_target(None), Some(PathBuf::from(".")));
        assert_eq!(browse_target(Some("does-not-exist.md")), None);
        let dir = temp_dir_with_doc();
        assert_eq!(
            browse_target(Some(dir.to_str().unwrap())),
            Some(dir.clone())
        );
        assert_eq!(
            browse_target(Some(dir.join("doc.md").to_str().unwrap())),
            None
        );
    }

    #[test]
    fn selecting_a_file_opens_pager_and_q_returns_to_browser() {
        let dir = temp_dir_with_doc();
        let mut app = app_for(&dir);
        app.browser.scan().expect("scan");
        assert!(app.view().contains("doc.md"));

        app.update(Message::new(FileSelectedMsg {
            path: dir.join("doc.md"),
        }));
        assert!(app.pager.is_some(), "pager should open");
        assert!(app.view().contains("Hello"), "rendered doc: {}", app.view());

        // `q` in the pager asks to go back rather than quitting.
        let msg = run(app.update(key('q'))).expect("back message");
        assert!(msg.is::<BackToBrowserMsg>());
        assert!(run(app.update(msg)).is_none());
        assert!(app.pager.is_none(), "should be back in the browser");
    }

    #[test]
    fn q_in_browser_quits() {
        let dir = temp_dir_with_doc();
        let mut app = app_for(&dir);
        let msg = run(app.update(key('q'))).expect("quit message");
        assert!(msg.is::<bubbletea::QuitMsg>());
    }

    #[test]
    fn q_in_filter_mode_is_typed_not_quit() {
        let dir = temp_dir_with_doc();
        let mut app = app_for(&dir);
        app.update(key('/'));
        assert!(run(app.update(key('q'))).is_none());
        assert_eq!(app.browser.filter(), "q");
    }

    #[test]
    fn missing_file_reports_error_and_stays_in_browser() {
        let dir = temp_dir_with_doc();
        let mut app = app_for(&dir);
        app.update(Message::new(FileSelectedMsg {
            path: dir.join("missing.md"),
        }));
        assert!(app.pager.is_none());
        assert!(app.view().contains("Error reading"));
    }

    #[test]
    fn standalone_pager_q_quits() {
        let pager = Pager::new("x".into(), "x".into(), "t".into(), None);
        let msg = run(Some(pager.leave())).expect("quit");
        assert!(msg.is::<bubbletea::QuitMsg>());
    }
}
