#![forbid(unsafe_code)]

//! # Demo Showcase
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

mod app;
pub mod cli;
mod components;
pub mod config;
mod data;
mod keymap;
mod messages;
mod pages;
pub mod test_support;
mod theme;

use bubbletea::{Model, Program};
use clap::Parser;

use app::{App, AppConfig};
use cli::{Cli, Command};
use config::Config;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(cmd) = &cli.command {
        handle_subcommand(cmd, &cli);
        return Ok(());
    }

    // Build runtime config from CLI
    let config = Config::from_cli(&cli);

    // Validate config
    config.validate()?;

    // Handle self-check mode
    if config.is_headless() {
        return run_self_check(&config);
    }

    // Build app config from runtime config
    let app_config = build_app_config(&config);
    let app = App::with_config(app_config);

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

/// Build application config from runtime config.
const fn build_app_config(config: &Config) -> AppConfig {
    AppConfig {
        theme: config.theme_preset,
        animations: config.use_animations(),
        mouse: config.mouse,
    }
}

/// Handle subcommands.
fn handle_subcommand(cmd: &Command, cli: &Cli) {
    match cmd {
        #[cfg(feature = "ssh")]
        Command::Ssh(args) => {
            eprintln!("SSH mode not yet implemented");
            eprintln!("Would listen on: {}", args.addr);
            eprintln!("Host key: {}", args.host_key.display());
        }
        Command::Export(args) => {
            eprintln!("Export not yet implemented");
            eprintln!("Format: {:?}", args.format);
            eprintln!("Output: {}", args.output.display());
            if let Some(page) = &args.page {
                eprintln!("Page: {page}");
            }
        }
        Command::Diagnostics => {
            print_diagnostics(cli);
        }
    }
}

/// Run headless self-check mode.
fn run_self_check(config: &Config) -> anyhow::Result<()> {
    eprintln!("Running self-check...");
    eprintln!(
        "Config: {}",
        config.to_diagnostic_string().replace('\n', ", ")
    );

    let app_config = build_app_config(config);
    let app = App::with_config(app_config);

    // Just verify we can create and view the app
    let view = app.view();
    if view.is_empty() {
        anyhow::bail!("Self-check failed: empty view");
    }

    eprintln!("✓ App creates successfully");
    eprintln!("✓ View renders ({} chars)", view.len());
    eprintln!("✓ Self-check passed");

    Ok(())
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
