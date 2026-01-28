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

    // =========================================================================
    // bd-2f52: Shell-out action safety tests
    // =========================================================================

    // --- Headless safety tests ---

    #[test]
    fn headless_open_in_pager_returns_none_immediately() {
        // Headless mode must return None without delay (no process spawn).
        let start = std::time::Instant::now();
        let result = open_in_pager("some long content".to_string(), true);
        let elapsed = start.elapsed();

        assert!(result.is_none(), "headless must return None");
        assert!(
            elapsed.as_millis() < 50,
            "headless must return quickly (took {}ms)",
            elapsed.as_millis()
        );
    }

    #[test]
    fn headless_open_diagnostics_in_pager_returns_none() {
        let diag = generate_diagnostics();
        let result = open_diagnostics_in_pager(diag, true);
        assert!(
            result.is_none(),
            "open_diagnostics_in_pager must be no-op when headless"
        );
    }

    #[test]
    fn headless_repeated_calls_all_noop() {
        // Multiple headless invocations must all return None (no accumulation).
        for i in 0..100 {
            let result = open_in_pager(format!("content {i}"), true);
            assert!(result.is_none(), "iteration {i} should be None");
        }
    }

    #[test]
    fn headless_empty_content_returns_none() {
        assert!(open_in_pager(String::new(), true).is_none());
    }

    #[test]
    fn headless_large_content_returns_none_without_processing() {
        // Even with megabytes of content, headless must short-circuit.
        let large = "x".repeat(1_000_000);
        let start = std::time::Instant::now();
        let result = open_in_pager(large, true);
        let elapsed = start.elapsed();

        assert!(result.is_none());
        assert!(
            elapsed.as_millis() < 50,
            "headless large content took {}ms",
            elapsed.as_millis()
        );
    }

    // --- Cmd structure tests (non-headless) ---

    #[test]
    fn non_headless_returns_cmd() {
        let result = open_in_pager("content".to_string(), false);
        assert!(result.is_some(), "non-headless must return Some(Cmd)");
    }

    #[test]
    fn non_headless_cmd_is_sequence_with_three_steps() {
        // The Cmd returned by open_in_pager wraps a sequence of:
        //   1. release_terminal  (terminal control message)
        //   2. blocking(run_pager) -> ShellOutMsg::PagerCompleted
        //   3. restore_terminal  (terminal control message)
        //
        // Executing the outer Cmd produces a SequenceMsg containing 3 sub-commands.
        let cmd = open_in_pager("test".to_string(), false).unwrap();
        let msg = cmd.execute();
        assert!(msg.is_some(), "sequence cmd should produce a message");

        let msg = msg.unwrap();
        let seq = msg
            .downcast::<bubbletea::message::SequenceMsg>()
            .expect("message should be SequenceMsg");

        assert_eq!(
            seq.0.len(),
            3,
            "sequence must have exactly 3 commands (release, pager, restore)"
        );
    }

    #[test]
    fn sequence_first_step_is_terminal_control() {
        let cmd = open_in_pager("test".to_string(), false).unwrap();
        let seq = cmd
            .execute()
            .unwrap()
            .downcast::<bubbletea::message::SequenceMsg>()
            .unwrap();

        // Step 1: release_terminal produces a terminal control message (not ShellOutMsg).
        let release_cmd = seq.0.into_iter().next().unwrap();
        let release_msg = release_cmd.execute();
        assert!(release_msg.is_some(), "release cmd should produce a message");
        assert!(
            !release_msg.unwrap().is::<ShellOutMsg>(),
            "first step must be a terminal control message, not ShellOutMsg"
        );
    }

    #[test]
    fn sequence_last_step_is_terminal_control() {
        let cmd = open_in_pager("test".to_string(), false).unwrap();
        let seq = cmd
            .execute()
            .unwrap()
            .downcast::<bubbletea::message::SequenceMsg>()
            .unwrap();

        // Step 3 (last): restore_terminal produces a terminal control message.
        let restore_cmd = seq.0.into_iter().last().unwrap();
        let restore_msg = restore_cmd.execute();
        assert!(
            restore_msg.is_some(),
            "restore cmd should produce a message"
        );
        assert!(
            !restore_msg.unwrap().is::<ShellOutMsg>(),
            "last step must be a terminal control message, not ShellOutMsg"
        );
    }

    #[test]
    fn sequence_ordering_release_pager_restore() {
        // Verify the full ordering: release → pager → restore.
        // Steps 1 and 3 are terminal control (non-blocking, instant).
        // Step 2 is the pager (blocking, spawns a process — don't execute in CI).
        let cmd = open_in_pager("test".to_string(), false).unwrap();
        let seq = cmd
            .execute()
            .unwrap()
            .downcast::<bubbletea::message::SequenceMsg>()
            .unwrap();

        let mut cmds = seq.0.into_iter();

        // Step 1: release — executes instantly, produces a non-ShellOutMsg message
        let step1_msg = cmds.next().unwrap().execute().unwrap();
        assert!(
            !step1_msg.is::<ShellOutMsg>(),
            "step 1 must be terminal release"
        );

        // Step 2: pager command — skip execution (would block)
        let _step2_cmd = cmds.next().unwrap();

        // Step 3: restore — executes instantly, produces a non-ShellOutMsg message
        let step3_msg = cmds.next().unwrap().execute().unwrap();
        assert!(
            !step3_msg.is::<ShellOutMsg>(),
            "step 3 must be terminal restore"
        );

        assert!(cmds.next().is_none(), "no extra commands after restore");
    }

    #[test]
    fn sequence_terminal_control_steps_are_instant() {
        // Steps 1 and 3 (release/restore) should execute in microseconds.
        let cmd = open_in_pager("test".to_string(), false).unwrap();
        let seq = cmd
            .execute()
            .unwrap()
            .downcast::<bubbletea::message::SequenceMsg>()
            .unwrap();

        let mut cmds = seq.0.into_iter();
        let release_cmd = cmds.next().unwrap();
        let _pager_cmd = cmds.next().unwrap();
        let restore_cmd = cmds.next().unwrap();

        let start = std::time::Instant::now();
        let _ = release_cmd.execute();
        let _ = restore_cmd.execute();
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 10,
            "terminal control commands took {}ms — should be instant",
            elapsed.as_millis()
        );
    }

    #[test]
    fn build_pager_sequence_produces_valid_cmd() {
        // build_pager_sequence is the internal helper; verify it produces a Cmd.
        let cmd = build_pager_sequence("hello world".to_string());
        let msg = cmd.execute();
        assert!(msg.is_some(), "build_pager_sequence must produce a message");
        assert!(
            msg.unwrap().is::<bubbletea::message::SequenceMsg>(),
            "must produce SequenceMsg"
        );
    }

    // --- ShellOutMsg structure tests ---

    #[test]
    fn shell_out_msg_pager_completed_success() {
        let msg = ShellOutMsg::PagerCompleted(None).into_message();
        let shell_msg = msg.downcast::<ShellOutMsg>().unwrap();
        match shell_msg {
            ShellOutMsg::PagerCompleted(err) => assert!(err.is_none()),
            other => panic!("expected PagerCompleted(None), got {:?}", other),
        }
    }

    #[test]
    fn shell_out_msg_pager_completed_error() {
        let msg = ShellOutMsg::PagerCompleted(Some("spawn failed".into())).into_message();
        let shell_msg = msg.downcast::<ShellOutMsg>().unwrap();
        match shell_msg {
            ShellOutMsg::PagerCompleted(Some(e)) => assert_eq!(e, "spawn failed"),
            other => panic!("expected PagerCompleted(Some(..)), got {:?}", other),
        }
    }

    #[test]
    fn shell_out_msg_all_variants_roundtrip() {
        // Verify every ShellOutMsg variant can be converted to Message and back.
        let variants: Vec<ShellOutMsg> = vec![
            ShellOutMsg::OpenDiagnostics,
            ShellOutMsg::PagerCompleted(None),
            ShellOutMsg::PagerCompleted(Some("error".into())),
            ShellOutMsg::TerminalReleased,
            ShellOutMsg::TerminalRestored,
        ];

        for variant in variants {
            let label = format!("{:?}", variant);
            let msg = variant.into_message();
            assert!(
                msg.downcast::<ShellOutMsg>().is_some(),
                "{label} failed roundtrip"
            );
        }
    }

    // --- Diagnostics generation ---

    #[test]
    fn diagnostics_contains_version_info() {
        let diag = generate_diagnostics();
        assert!(diag.contains("Package Version:"));
        assert!(diag.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn diagnostics_contains_platform_info() {
        let diag = generate_diagnostics();
        assert!(diag.contains(&format!("OS: {}", std::env::consts::OS)));
        assert!(diag.contains(&format!("Arch: {}", std::env::consts::ARCH)));
    }

    #[test]
    fn diagnostics_has_consistent_structure() {
        // Diagnostics should have opening/closing banner lines.
        let diag = generate_diagnostics();
        let lines: Vec<&str> = diag.lines().collect();
        assert!(
            lines.len() >= 10,
            "diagnostics should have at least 10 lines, got {}",
            lines.len()
        );
        // First and last lines are banner separators
        assert!(
            lines.first().unwrap().contains('═'),
            "should start with banner"
        );
        assert!(
            lines.last().unwrap().contains('═'),
            "should end with banner"
        );
    }

    // --- command_exists edge cases ---

    #[test]
    fn command_exists_nonexistent_returns_false() {
        assert!(
            !command_exists("__nonexistent_command_that_does_not_exist_12345__"),
            "nonexistent command should return false"
        );
    }

    #[test]
    fn command_exists_empty_string() {
        // Empty command name should not panic.
        let _ = command_exists("");
    }
}
