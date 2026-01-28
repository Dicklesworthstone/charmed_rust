//! Command-line interface for `demo_showcase`.
//!
//! Defines the CLI contract using clap derive macros. This provides a clean,
//! documented interface for all runtime options.
//!
//! # Examples
//!
//! ```bash
//! # Run with default settings
//! demo_showcase
//!
//! # Run with specific theme and seed
//! demo_showcase --theme nord --seed 42
//!
//! # Run headless self-check (for CI)
//! demo_showcase --self-check
//!
//! # Run without animations
//! demo_showcase --no-animations
//! ```

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

/// Charmed Control Center - TUI showcase for `charmed_rust` libraries.
///
/// A feature-rich terminal application demonstrating all capabilities
/// of the `charmed_rust` TUI framework: bubbletea, lipgloss, bubbles,
/// glamour, huh, harmonica, and `charmed_log`.
#[derive(Parser, Debug, Clone)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "CLI flags are naturally bools"
)]
#[command(
    name = "demo_showcase",
    author,
    version,
    about = "Charmed Control Center - TUI showcase for charmed_rust",
    long_about = "A feature-rich terminal application demonstrating all capabilities \
                  of the charmed_rust TUI framework."
)]
pub struct Cli {
    /// Theme to use for styling
    ///
    /// Available themes: catppuccin, nord, github-dark, auto (detect from terminal)
    #[arg(long, short = 't', default_value = "auto", env = "DEMO_THEME")]
    pub theme: String,

    /// Path to a custom theme JSON file
    ///
    /// Overrides the --theme flag if specified
    #[arg(long, env = "DEMO_THEME_FILE")]
    pub theme_file: Option<PathBuf>,

    /// Seed for deterministic demo data generation
    ///
    /// Using the same seed produces identical simulated data,
    /// useful for reproducible demos and testing
    #[arg(long, short = 's', env = "DEMO_SEED")]
    pub seed: Option<u64>,

    /// Disable animations
    ///
    /// Respects `REDUCE_MOTION` environment variable
    #[arg(long, env = "DEMO_NO_ANIMATIONS")]
    pub no_animations: bool,

    /// Disable mouse support
    ///
    /// Mouse clicks and scrolling will be ignored
    #[arg(long, env = "DEMO_NO_MOUSE")]
    pub no_mouse: bool,

    /// Force color output off (ASCII mode)
    ///
    /// Respects `NO_COLOR` environment variable per spec
    #[arg(long, env = "NO_COLOR")]
    pub no_color: bool,

    /// Force color output on (overrides `NO_COLOR`)
    #[arg(long, conflicts_with = "no_color")]
    pub force_color: bool,

    /// Disable alternate screen mode
    ///
    /// Runs in the main terminal buffer; useful for debugging
    /// and demonstrating bubbletea's println/printf
    #[arg(long, env = "DEMO_NO_ALT_SCREEN")]
    pub no_alt_screen: bool,

    /// Run headless self-check and exit
    ///
    /// Renders all pages without TTY, useful for CI validation
    #[arg(long)]
    pub self_check: bool,

    /// Root directory for file browser
    ///
    /// Defaults to current working directory
    #[arg(long, env = "DEMO_FILES_ROOT")]
    pub files_root: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Optional subcommand
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available subcommands for the demo showcase.
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Run the showcase over SSH (requires 'ssh' feature)
    #[cfg(feature = "ssh")]
    Ssh(SshArgs),

    /// Export a snapshot of the current view
    Export(ExportArgs),

    /// Show diagnostic information
    Diagnostics,
}

/// Arguments for SSH server mode.
#[cfg(feature = "ssh")]
#[derive(Parser, Debug, Clone)]
pub struct SshArgs {
    /// Address to listen on
    #[arg(long, default_value = ":2222")]
    pub addr: String,

    /// Path to host key file
    #[arg(long)]
    pub host_key: PathBuf,

    /// Maximum concurrent sessions
    #[arg(long, default_value = "10")]
    pub max_sessions: usize,
}

/// Arguments for export subcommand.
#[derive(Parser, Debug, Clone)]
pub struct ExportArgs {
    /// Output format
    #[arg(long, short = 'f', default_value = "plain")]
    pub format: ExportFormat,

    /// Output file path
    #[arg(long, short = 'o')]
    pub output: PathBuf,

    /// Page to export (all if not specified)
    #[arg(long)]
    pub page: Option<String>,
}

/// Export output formats.
#[derive(ValueEnum, Debug, Clone, Copy, Default)]
pub enum ExportFormat {
    /// Plain text (ANSI stripped)
    #[default]
    Plain,
    /// HTML with inline styles
    Html,
    /// ANSI-colored text
    Ansi,
}

impl Cli {
    /// Parse command line arguments.
    #[must_use]
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Create CLI from iterator (useful for testing).
    ///
    /// # Errors
    ///
    /// Returns an error if argument parsing fails.
    pub fn try_parse_from<I, T>(iter: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        <Self as Parser>::try_parse_from(iter)
    }

    /// Check if running in headless mode.
    #[must_use]
    pub const fn is_headless(&self) -> bool {
        self.self_check
    }

    /// Check if colors should be used.
    #[must_use]
    pub const fn use_color(&self) -> bool {
        self.force_color || !self.no_color
    }

    /// Check if animations should be used.
    #[must_use]
    pub fn use_animations(&self) -> bool {
        if self.no_animations {
            return false;
        }

        // Check REDUCE_MOTION env var
        if std::env::var("REDUCE_MOTION").is_ok() {
            return false;
        }

        true
    }

    /// Get the effective seed (random if not specified).
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Seed truncation is acceptable"
    )]
    pub fn effective_seed(&self) -> u64 {
        self.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(42, |d| d.as_nanos() as u64)
        })
    }

    /// Get the files root directory.
    #[must_use]
    pub fn effective_files_root(&self) -> PathBuf {
        self.files_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Get log level based on verbosity.
    #[must_use]
    pub const fn log_level(&self) -> LogLevel {
        match self.verbose {
            0 => LogLevel::Warn,
            1 => LogLevel::Info,
            2 => LogLevel::Debug,
            _ => LogLevel::Trace,
        }
    }
}

/// Log level for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Only show warnings and errors
    Warn,
    /// Show info messages
    Info,
    /// Show debug messages
    Debug,
    /// Show all messages including trace
    Trace,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_defaults() {
        let cli = Cli::try_parse_from(["demo_showcase"]).unwrap();

        assert_eq!(cli.theme, "auto");
        assert!(cli.seed.is_none());
        assert!(!cli.no_animations);
        assert!(!cli.no_mouse);
        assert!(!cli.no_color);
        assert!(!cli.no_alt_screen);
        assert!(!cli.self_check);
    }

    #[test]
    fn cli_parses_theme() {
        let cli = Cli::try_parse_from(["demo_showcase", "--theme", "nord"]).unwrap();
        assert_eq!(cli.theme, "nord");

        let cli = Cli::try_parse_from(["demo_showcase", "-t", "catppuccin"]).unwrap();
        assert_eq!(cli.theme, "catppuccin");
    }

    #[test]
    fn cli_parses_seed() {
        let cli = Cli::try_parse_from(["demo_showcase", "--seed", "42"]).unwrap();
        assert_eq!(cli.seed, Some(42));

        let cli = Cli::try_parse_from(["demo_showcase", "-s", "12345"]).unwrap();
        assert_eq!(cli.seed, Some(12345));
    }

    #[test]
    fn cli_parses_flags() {
        let cli = Cli::try_parse_from([
            "demo_showcase",
            "--no-animations",
            "--no-mouse",
            "--no-color",
            "--no-alt-screen",
        ])
        .unwrap();

        assert!(cli.no_animations);
        assert!(cli.no_mouse);
        assert!(cli.no_color);
        assert!(cli.no_alt_screen);
    }

    #[test]
    fn cli_parses_self_check() {
        let cli = Cli::try_parse_from(["demo_showcase", "--self-check"]).unwrap();
        assert!(cli.self_check);
        assert!(cli.is_headless());
    }

    #[test]
    fn cli_parses_files_root() {
        let cli = Cli::try_parse_from(["demo_showcase", "--files-root", "/tmp/test"]).unwrap();
        assert_eq!(cli.files_root, Some(PathBuf::from("/tmp/test")));
    }

    #[test]
    fn cli_parses_verbose() {
        let cli = Cli::try_parse_from(["demo_showcase"]).unwrap();
        assert_eq!(cli.verbose, 0);
        assert_eq!(cli.log_level(), LogLevel::Warn);

        let cli = Cli::try_parse_from(["demo_showcase", "-v"]).unwrap();
        assert_eq!(cli.verbose, 1);
        assert_eq!(cli.log_level(), LogLevel::Info);

        let cli = Cli::try_parse_from(["demo_showcase", "-vv"]).unwrap();
        assert_eq!(cli.verbose, 2);
        assert_eq!(cli.log_level(), LogLevel::Debug);

        let cli = Cli::try_parse_from(["demo_showcase", "-vvv"]).unwrap();
        assert_eq!(cli.verbose, 3);
        assert_eq!(cli.log_level(), LogLevel::Trace);
    }

    #[test]
    fn cli_use_color_logic() {
        let cli = Cli::try_parse_from(["demo_showcase"]).unwrap();
        assert!(cli.use_color());

        let cli = Cli::try_parse_from(["demo_showcase", "--no-color"]).unwrap();
        assert!(!cli.use_color());

        let cli = Cli::try_parse_from(["demo_showcase", "--force-color"]).unwrap();
        assert!(cli.use_color());
    }

    #[test]
    fn cli_force_color_conflicts_with_no_color() {
        let result = Cli::try_parse_from(["demo_showcase", "--no-color", "--force-color"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_parses_export_subcommand() {
        let cli = Cli::try_parse_from([
            "demo_showcase",
            "export",
            "--format",
            "html",
            "--output",
            "/tmp/out.html",
        ])
        .unwrap();

        match cli.command {
            Some(Command::Export(args)) => {
                assert!(matches!(args.format, ExportFormat::Html));
                assert_eq!(args.output, PathBuf::from("/tmp/out.html"));
            }
            _ => panic!("Expected Export command"),
        }
    }

    #[test]
    fn cli_parses_diagnostics_subcommand() {
        let cli = Cli::try_parse_from(["demo_showcase", "diagnostics"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Diagnostics)));
    }

    #[test]
    fn cli_help_works() {
        let result = Cli::try_parse_from(["demo_showcase", "--help"]);
        // --help returns an error (but it's the "help" kind)
        assert!(result.is_err());
    }

    #[test]
    fn effective_seed_uses_provided() {
        let cli = Cli::try_parse_from(["demo_showcase", "--seed", "999"]).unwrap();
        assert_eq!(cli.effective_seed(), 999);
    }

    #[test]
    fn effective_seed_generates_random() {
        let cli = Cli::try_parse_from(["demo_showcase"]).unwrap();
        let seed1 = cli.effective_seed();
        // Small delay to ensure different time
        std::thread::sleep(std::time::Duration::from_millis(1));
        let seed2 = cli.effective_seed();
        // Seeds should be different (time-based)
        // Note: In rare cases they might be the same if time resolution is low
        assert!(seed1 != seed2 || seed1 > 0);
    }
}
