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
mod data;
mod keymap;
mod messages;
mod pages;
mod theme;

use bubbletea::{Model, Program};
use clap::Parser;

use app::{App, AppConfig};
use cli::{Cli, Command};
use theme::ThemePreset;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(cmd) = &cli.command {
        handle_subcommand(cmd, &cli);
        return Ok(());
    }

    // Handle self-check mode
    if cli.self_check {
        return run_self_check(&cli);
    }

    // Build config from CLI args
    let config = build_config(&cli);
    let app = App::with_config(config);

    // Build program with appropriate options
    let mut program = Program::new(app);

    if !cli.no_alt_screen {
        program = program.with_alt_screen();
    }

    // Note: mouse support is handled by the Program defaults
    // The no_mouse flag will be used by the App to ignore mouse events

    program.run()?;

    Ok(())
}

/// Build application config from CLI arguments.
fn build_config(cli: &Cli) -> AppConfig {
    // Determine theme preset
    let theme = match cli.theme.as_str() {
        "light" => ThemePreset::Light,
        "dracula" => ThemePreset::Dracula,
        // dark, auto, or any unknown value -> Dark
        _ => ThemePreset::Dark,
    };

    AppConfig {
        theme,
        animations: cli.use_animations(),
        mouse: !cli.no_mouse,
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
fn run_self_check(cli: &Cli) -> anyhow::Result<()> {
    eprintln!("Running self-check...");

    let config = build_config(cli);
    let app = App::with_config(config);

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
    println!("Charmed Control Center - Diagnostics");
    println!("=====================================");
    println!();
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Rust: {}", env!("CARGO_PKG_RUST_VERSION"));
    println!();
    println!("Configuration:");
    println!("  Theme: {}", cli.theme);
    if let Some(ref file) = cli.theme_file {
        println!("  Theme file: {}", file.display());
    }
    println!("  Seed: {:?}", cli.seed);
    println!(
        "  Animations: {}",
        if cli.use_animations() { "on" } else { "off" }
    );
    println!("  Mouse: {}", if cli.no_mouse { "off" } else { "on" });
    println!("  Color: {}", if cli.use_color() { "on" } else { "off" });
    println!(
        "  Alt screen: {}",
        if cli.no_alt_screen { "off" } else { "on" }
    );
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
