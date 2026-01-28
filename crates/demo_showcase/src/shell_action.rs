//! Shell-out action for temporarily releasing the terminal.
//!
//! This module provides a showcase-grade example of the "drop to shell" pattern:
//! temporarily giving the user their terminal back to run external commands,
//! then restoring the TUI.
//!
//! # Usage
//!
//! ```rust,ignore
//! use demo_showcase::shell_action::open_in_pager;
//!
//! // In your update function:
//! if let Some(cmd) = open_in_pager(content, is_headless) {
//!     return Some(cmd);
//! }
//! ```
//!
//! # Design
//!
//! The implementation uses `bubbletea::sequence` to chain commands:
//! 1. `screen::release_terminal()` - restore cooked mode, show cursor, leave alt-screen
//! 2. Run external command (pager or fallback prompt)
//! 3. `screen::restore_terminal()` - re-enable raw mode, hide cursor, enter alt-screen
//!
//! # Headless Safety
//!
//! When running in headless/self-check mode, these functions return `None`
//! (no-op) to prevent hanging on user input.

use bubbletea::{screen, sequence, Cmd, Message};
use std::env;
use std::io::{self, Read, Write};
use std::process::Command;

use crate::messages::ShellOutMsg;

/// Open content in the system pager.
///
/// This is the primary API for shell-out actions. It uses `bubbletea::sequence`
/// to properly release and restore the terminal around the pager command.
///
/// # Arguments
///
/// * `content` - The text content to display in the pager
/// * `is_headless` - If true, returns None (no-op) for CI safety
///
/// # Returns
///
/// `Some(Cmd)` with the sequenced commands, or `None` if headless.
///
/// # Pager Selection
///
/// Tries in order:
/// 1. `$PAGER` environment variable
/// 2. `less -R` (with ANSI color support)
/// 3. `more`
/// 4. Fallback: print to stdout and wait for Enter
#[must_use]
pub fn open_in_pager(content: String, is_headless: bool) -> Option<Cmd> {
    // No-op in headless mode to prevent CI hangs
    if is_headless {
        return None;
    }

    Some(build_pager_sequence(content))
}

/// Open diagnostics information in the pager.
///
/// Collects system/app diagnostics and displays them in the pager.
///
/// # Arguments
///
/// * `diagnostics` - Pre-formatted diagnostics string
/// * `is_headless` - If true, returns None (no-op) for CI safety
#[must_use]
pub fn open_diagnostics_in_pager(diagnostics: String, is_headless: bool) -> Option<Cmd> {
    open_in_pager(diagnostics, is_headless)
}

/// Build the command sequence for pager display.
///
/// Uses `bubbletea::sequence` to chain:
/// 1. Release terminal
/// 2. Run pager (blocking)
/// 3. Restore terminal
fn build_pager_sequence(content: String) -> Cmd {
    // The sequence function chains these commands in order
    sequence(vec![
        // Step 1: Release terminal for external use
        Some(screen::release_terminal()),
        // Step 2: Run the pager command (this blocks until pager exits)
        Some(Cmd::blocking(move || run_pager(&content))),
        // Step 3: Restore terminal for TUI
        Some(screen::restore_terminal()),
    ])
    .expect("sequence should not be empty")
}

/// Run the pager with the given content.
///
/// Tries pagers in order of preference, falling back to a simple
/// "press Enter to continue" prompt if no pager is available.
fn run_pager(content: &str) -> Message {
    // Try to get preferred pager from environment
    let pager = env::var("PAGER").ok();

    let result = if let Some(pager_cmd) = pager {
        // Use $PAGER if set
        run_pager_command(&pager_cmd, content)
    } else if command_exists("less") {
        // Try less with ANSI color support
        run_pager_command("less -R", content)
    } else if command_exists("more") {
        // Fall back to more
        run_pager_command("more", content)
    } else {
        // Ultimate fallback: print and wait for Enter
        run_fallback_prompt(content)
    };

    match result {
        Ok(()) => ShellOutMsg::PagerCompleted(None).into_message(),
        Err(e) => ShellOutMsg::PagerCompleted(Some(e)).into_message(),
    }
}

/// Check if a command exists in PATH.
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Run a pager command with the given content as stdin.
fn run_pager_command(pager_cmd: &str, content: &str) -> Result<(), String> {
    // Split command and args (e.g., "less -R" -> "less", ["-R"])
    let parts: Vec<&str> = pager_cmd.split_whitespace().collect();
    let (cmd, args) = parts
        .split_first()
        .ok_or_else(|| "empty pager command".to_string())?;

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn pager: {e}"))?;

    // Write content to pager's stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| format!("failed to write to pager: {e}"))?;
    }

    // Wait for pager to exit
    let status = child
        .wait()
        .map_err(|e| format!("failed to wait for pager: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        // Non-zero exit is usually fine (e.g., user pressed 'q' in less)
        Ok(())
    }
}

/// Fallback prompt when no pager is available.
///
/// Prints content to stdout and waits for Enter.
fn run_fallback_prompt(content: &str) -> Result<(), String> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    // Print content
    writeln!(stdout, "{content}").map_err(|e| format!("write error: {e}"))?;
    writeln!(stdout).map_err(|e| format!("write error: {e}"))?;
    writeln!(stdout, "--- Press Enter to return to the application ---")
        .map_err(|e| format!("write error: {e}"))?;
    stdout.flush().map_err(|e| format!("flush error: {e}"))?;

    // Wait for Enter
    let mut buf = [0u8; 1];
    stdin
        .lock()
        .read_exact(&mut buf)
        .map_err(|e| format!("read error: {e}"))?;

    Ok(())
}

/// Generate diagnostics information about the application.
///
/// This collects useful debugging information that can be shown to the user
/// or sent to support.
#[must_use]
pub fn generate_diagnostics() -> String {
    let mut lines = Vec::new();

    lines.push("═══════════════════════════════════════════════════════════".to_string());
    lines.push("                    DEMO SHOWCASE DIAGNOSTICS               ".to_string());
    lines.push("═══════════════════════════════════════════════════════════".to_string());
    lines.push(String::new());

    // Version info
    lines.push("Version Information:".to_string());
    lines.push(format!(
        "  Package Version: {}",
        env!("CARGO_PKG_VERSION")
    ));
    lines.push(format!("  Rust Version: {}", rustc_version()));
    lines.push(String::new());

    // Environment
    lines.push("Environment:".to_string());
    lines.push(format!("  TERM: {}", env::var("TERM").unwrap_or_default()));
    lines.push(format!(
        "  COLORTERM: {}",
        env::var("COLORTERM").unwrap_or_default()
    ));
    lines.push(format!("  PAGER: {}", env::var("PAGER").unwrap_or_default()));
    lines.push(format!(
        "  NO_COLOR: {}",
        if env::var("NO_COLOR").is_ok() {
            "set"
        } else {
            "not set"
        }
    ));
    lines.push(format!(
        "  REDUCE_MOTION: {}",
        if env::var("REDUCE_MOTION").is_ok() {
            "set"
        } else {
            "not set"
        }
    ));
    lines.push(String::new());

    // Platform info
    lines.push("Platform:".to_string());
    lines.push(format!("  OS: {}", std::env::consts::OS));
    lines.push(format!("  Arch: {}", std::env::consts::ARCH));
    lines.push(String::new());

    // Current directory
    if let Ok(cwd) = env::current_dir() {
        lines.push(format!("Working Directory: {}", cwd.display()));
        lines.push(String::new());
    }

    // Charmed Rust crate versions
    lines.push("Charmed Rust Components:".to_string());
    lines.push("  bubbletea: (workspace)".to_string());
    lines.push("  lipgloss: (workspace)".to_string());
    lines.push("  bubbles: (workspace)".to_string());
    lines.push("  glamour: (workspace)".to_string());
    lines.push("  harmonica: (workspace)".to_string());
    lines.push(String::new());

    lines.push("═══════════════════════════════════════════════════════════".to_string());

    lines.join("\n")
}

/// Get rustc version (compile-time).
fn rustc_version() -> &'static str {
    // This is set by build.rs or we use a fallback
    option_env!("RUSTC_VERSION").unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_pager_headless_returns_none() {
        // In headless mode, should return None
        let result = open_in_pager("test content".to_string(), true);
        assert!(result.is_none());
    }

    #[test]
    fn test_open_in_pager_non_headless_returns_some() {
        // In non-headless mode, should return Some(Cmd)
        let result = open_in_pager("test content".to_string(), false);
        assert!(result.is_some());
    }

    #[test]
    fn test_generate_diagnostics_not_empty() {
        let diag = generate_diagnostics();
        assert!(!diag.is_empty());
        assert!(diag.contains("DEMO SHOWCASE DIAGNOSTICS"));
        assert!(diag.contains("Version Information"));
        assert!(diag.contains("Environment"));
        assert!(diag.contains("Platform"));
    }

    #[test]
    fn test_generate_diagnostics_contains_expected_sections() {
        let diag = generate_diagnostics();

        // Check for key sections
        assert!(diag.contains("TERM:"));
        assert!(diag.contains("COLORTERM:"));
        assert!(diag.contains("NO_COLOR:"));
        assert!(diag.contains("OS:"));
        assert!(diag.contains("Arch:"));
        assert!(diag.contains("Charmed Rust Components:"));
    }

    #[test]
    fn test_command_exists_known_command() {
        // 'which' itself should exist on Unix systems
        #[cfg(unix)]
        {
            // Test with a command that should always exist
            let result = command_exists("ls");
            // Don't assert true since this is environment-dependent
            // Just ensure it doesn't panic
            let _ = result;
        }
    }
}
