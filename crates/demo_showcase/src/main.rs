#![forbid(unsafe_code)]

//! # Demo Showcase Binary
//!
//! Flagship demonstration of all `charmed_rust` TUI capabilities.
//!
//! This showcase serves as both a feature demonstration and
//! a reference implementation for building complex TUI applications.
//!
//! ## Features Demonstrated
//!
//! - **bubbletea**: Elm architecture, event loop, commands
//! - **lipgloss**: Styling, colors, borders, layout
//! - **bubbles**: Components (viewport, list, textinput, spinner, etc.)
//! - **glamour**: Markdown rendering
//! - **harmonica**: Spring animations
//! - **huh**: Interactive forms
//! - **`charmed_log`**: Structured logging
//!
//! ## Usage
//!
//! ```bash
//! # Run with defaults
//! cargo run -p demo_showcase
//!
//! # Run with specific options
//! cargo run -p demo_showcase -- --theme nord --seed 42
//!
//! # Show help
//! cargo run -p demo_showcase -- --help
//! ```

use bubbletea::Program;

// Re-export from library for use in main
use clap::Parser;
use demo_showcase::app::App;
use demo_showcase::app::{ansi_to_html, strip_ansi};
use demo_showcase::cli::{Cli, Command};
use demo_showcase::config::Config;
use demo_showcase::messages;
use demo_showcase::messages::Page;
#[cfg(feature = "ssh")]
use demo_showcase::ssh;
use demo_showcase::test_support;

#[allow(clippy::large_stack_frames)] // Program struct is large by design
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(cmd) = &cli.command {
        return handle_subcommand(cmd, &cli);
    }

    // Build runtime config from CLI
    let config = Config::from_cli(&cli);

    // Validate config
    config.validate()?;

    // Handle self-check mode
    if config.is_headless() {
        return run_self_check(&config);
    }

    // Apply max_width to PTY size if terminal is very wide
    // This works around rendering issues with ultra-wide terminals by telling
    // the PTY driver to use a narrower width for line wrapping calculations.
    // The explicit max_width CLI flag or the auto-cap at 200 columns triggers this.
    apply_pty_width_cap(&config);

    // Bootstrap app from config (canonical entrypoint)
    let app = App::from_config(&config);

    // Build program with appropriate options
    // All terminal behavior is driven from Config (single source of truth)
    let mut program = Program::new(app);

    // Alternate screen mode (default: on, override: --no-alt-screen)
    if config.alt_screen {
        program = program.with_alt_screen();
    }

    // Focus reporting: enables FocusMsg/BlurMsg when terminal gains/loses focus
    program = program.with_report_focus();

    // Mouse support: enable cell motion tracking when mouse is enabled
    // This reports clicks and drags. Config controls via --no-mouse flag.
    if config.mouse {
        program = program.with_mouse_cell_motion();
    }

    // Bracketed paste is enabled by default in bubbletea Program

    program.run()?;

    Ok(())
}

/// Handle subcommands.
///
/// # Errors
///
/// Returns an error if the subcommand fails.
#[allow(clippy::unnecessary_wraps)] // Consistent API with other command handlers
fn handle_subcommand(cmd: &Command, cli: &Cli) -> anyhow::Result<()> {
    match cmd {
        #[cfg(feature = "ssh")]
        Command::Ssh(args) => {
            let config = Config::from_cli(cli);
            let ssh_config = ssh::SshConfig::from_args(args, &config);

            // Initialize tracing for logging
            init_tracing(cli.verbose);

            // Run the SSH server using tokio runtime
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async {
                if let Err(e) = ssh::run_ssh_server(ssh_config).await {
                    // Print user-friendly error messages
                    match &e {
                        ssh::SshError::HostKeyNotFound(path) => {
                            eprintln!("Error: Host key file not found: {path}");
                            eprintln!();
                            eprintln!("To generate a host key, run:");
                            eprintln!("  ssh-keygen -t ed25519 -f {path} -N \"\"");
                            eprintln!("  chmod 600 {path}");
                        }
                        ssh::SshError::BindFailed(addr, reason) => {
                            eprintln!("Error: Failed to bind to {addr}");
                            eprintln!("  {reason}");
                        }
                        _ => {
                            eprintln!("Error: {e}");
                        }
                    }
                    std::process::exit(1);
                }
            });
        }
        Command::Export(args) => {
            let mut config = Config::from_cli(cli);
            // Export is non-interactive: force headless behavior and avoid alt-screen.
            config.self_check = true;
            config.alt_screen = false;
            config.validate()?;
            run_export(args, &config)?;
        }
        Command::Diagnostics => {
            print_diagnostics(cli);
        }
    }
    Ok(())
}

fn parse_page_arg(s: &str) -> Option<Page> {
    let s = s.trim().to_ascii_lowercase();
    if s.len() == 1 {
        let c = s.chars().next()?;
        if let Some(p) = Page::from_shortcut(c) {
            return Some(p);
        }
    }

    match s.as_str() {
        "dashboard" => Some(Page::Dashboard),
        "services" => Some(Page::Services),
        "jobs" => Some(Page::Jobs),
        "logs" => Some(Page::Logs),
        "docs" => Some(Page::Docs),
        "files" => Some(Page::Files),
        "wizard" => Some(Page::Wizard),
        "settings" => Some(Page::Settings),
        _ => None,
    }
}

const fn export_extension(format: demo_showcase::cli::ExportFormat) -> &'static str {
    match format {
        demo_showcase::cli::ExportFormat::Plain => "txt",
        demo_showcase::cli::ExportFormat::Html => "html",
        demo_showcase::cli::ExportFormat::Ansi => "ansi",
    }
}

fn export_convert(format: demo_showcase::cli::ExportFormat, ansi_view: &str) -> String {
    match format {
        demo_showcase::cli::ExportFormat::Plain => strip_ansi(ansi_view),
        demo_showcase::cli::ExportFormat::Html => ansi_to_html(ansi_view),
        demo_showcase::cli::ExportFormat::Ansi => ansi_view.to_string(),
    }
}

fn run_export(args: &demo_showcase::cli::ExportArgs, config: &Config) -> anyhow::Result<()> {
    use demo_showcase::test_support::E2ERunner;

    let pages: Vec<Page> = if let Some(page) = &args.page {
        vec![parse_page_arg(page).ok_or_else(|| {
            anyhow::anyhow!("unknown page {page:?}; expected one of: dashboard, services, jobs, logs, docs, files, wizard, settings (or 1-8)")
        })?]
    } else {
        Page::all().to_vec()
    };

    let ext = export_extension(args.format);

    // Output resolution:
    // - If exporting a single page: `--output` may be a file or a directory.
    // - If exporting multiple pages: `--output` must be a directory.
    let output_path = &args.output;
    let exporting_many = pages.len() > 1;

    if exporting_many {
        // Treat output as a directory. Create it if needed.
        if output_path.extension().is_some() {
            anyhow::bail!(
                "exporting multiple pages requires --output to be a directory path (got file-like path: {})",
                output_path.display()
            );
        }
        std::fs::create_dir_all(output_path)?;
    }

    for page in pages {
        let mut runner = E2ERunner::with_config("cli_export", config.clone());
        runner.resize(120, 40);
        runner.press_key(page.shortcut());
        runner.drain();

        let ansi_view = runner.view();
        let content = export_convert(args.format, &ansi_view);

        let out_file = if exporting_many || output_path.is_dir() {
            let name = page.name().to_ascii_lowercase();
            output_path.join(format!("{name}.{ext}"))
        } else {
            output_path.clone()
        };

        std::fs::write(&out_file, content)?;
        eprintln!("Exported {page:?} to {}", out_file.display());
    }

    Ok(())
}

/// Initialize tracing with the given verbosity level.
#[cfg(feature = "ssh")]
fn init_tracing(verbosity: u8) {
    use tracing_subscriber::EnvFilter;

    let level = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("demo_showcase={level},wish={level}")));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

/// Run headless self-check mode.
///
/// This exercises the app through multiple pages to verify core functionality
/// works without a real terminal. Uses the E2E runner infrastructure for
/// deterministic, artifact-producing test execution.
fn run_self_check(config: &Config) -> anyhow::Result<()> {
    use test_support::E2ERunner;

    eprintln!("Running self-check...");
    eprintln!(
        "Config: {}",
        config.to_diagnostic_string().replace('\n', ", ")
    );

    // Create E2E runner with self-check scenario
    let mut runner = E2ERunner::with_config("self_check", config.clone());

    // Step 1: Verify initial render
    runner.step("Verify initial render");
    if !runner.assert_view_not_empty() {
        let result = runner.finish();
        anyhow::bail!("Self-check failed: empty view\n{}", result.unwrap_err());
    }
    eprintln!("✓ App creates successfully");
    eprintln!("✓ View renders ({} chars)", runner.view().len());

    // Step 2: Navigate through pages
    runner.step("Navigate to Jobs page");
    runner.press_key('3'); // Jobs shortcut
    if !runner.assert_page(messages::Page::Jobs) {
        let result = runner.finish();
        anyhow::bail!(
            "Self-check failed: navigation to Jobs\n{}",
            result.unwrap_err()
        );
    }
    eprintln!("✓ Jobs page renders");

    runner.step("Navigate to Logs page");
    runner.press_key('4'); // Logs shortcut
    if !runner.assert_page(messages::Page::Logs) {
        let result = runner.finish();
        anyhow::bail!(
            "Self-check failed: navigation to Logs\n{}",
            result.unwrap_err()
        );
    }
    eprintln!("✓ Logs page renders");

    runner.step("Navigate to Docs page");
    runner.press_key('5'); // Docs shortcut
    if !runner.assert_page(messages::Page::Docs) {
        let result = runner.finish();
        anyhow::bail!(
            "Self-check failed: navigation to Docs\n{}",
            result.unwrap_err()
        );
    }
    eprintln!("✓ Docs page renders");

    runner.step("Return to Dashboard");
    runner.press_key('1'); // Dashboard shortcut
    if !runner.assert_page(messages::Page::Dashboard) {
        let result = runner.finish();
        anyhow::bail!(
            "Self-check failed: return to Dashboard\n{}",
            result.unwrap_err()
        );
    }
    eprintln!("✓ Dashboard page renders");

    // Step 3: Finish and verify no failures
    match runner.finish() {
        Ok(()) => {
            eprintln!("✓ Self-check passed ({} pages validated)", 4);
            Ok(())
        }
        Err(summary) => {
            anyhow::bail!("Self-check failed:\n{summary}");
        }
    }
}

/// Default maximum width for ultra-wide terminals.
///
/// Terminals wider than this get their PTY width capped at startup, and the
/// app's layout width is capped in the `WindowSizeMsg` handler. The value
/// must match the `AUTO_MAX_WIDTH` constant in `App::update`.
const AUTO_MAX_WIDTH: u16 = 200;

/// Apply PTY width cap for ultra-wide terminals.
///
/// When a terminal is very wide (> 200 columns), rendering can have issues
/// because the PTY driver's line wrapping calculations differ from the app's.
/// This function uses `stty cols` to set a narrower PTY width, which makes
/// the terminal driver handle line wrapping at the capped width.
fn apply_pty_width_cap(config: &Config) {
    let max_width = config.max_width.unwrap_or(AUTO_MAX_WIDTH);

    if let Ok((cols, _rows)) = crossterm::terminal::size()
        && cols > max_width
    {
        let _ = std::process::Command::new("stty")
            .arg("cols")
            .arg(max_width.to_string())
            .status();
    }
}

/// Print diagnostic information.
fn print_diagnostics(cli: &Cli) {
    let config = Config::from_cli(cli);

    println!("Charmed Control Center - Diagnostics");
    println!("=====================================");
    println!();
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Rust: {}", env!("CARGO_PKG_RUST_VERSION"));
    println!();
    println!("Configuration (resolved):");
    for line in config.to_diagnostic_string().lines() {
        println!("  {line}");
    }
    println!();
    println!("Features:");
    println!(
        "  syntax-highlighting: {}",
        cfg!(feature = "syntax-highlighting")
    );
    println!("  ssh: {}", cfg!(feature = "ssh"));
    println!();
    println!("Environment:");
    println!("  NO_COLOR: {:?}", std::env::var("NO_COLOR").ok());
    println!("  REDUCE_MOTION: {:?}", std::env::var("REDUCE_MOTION").ok());
    println!("  TERM: {:?}", std::env::var("TERM").ok());
    println!("  COLORTERM: {:?}", std::env::var("COLORTERM").ok());
}
