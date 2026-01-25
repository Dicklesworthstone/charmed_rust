//! Screen control commands.
//!
//! These commands control terminal display features like alternate screen,
//! cursor visibility, mouse tracking, and more.

use crate::command::Cmd;
use crate::message::Message;

// Internal message types for screen commands
pub(crate) struct ClearScreenMsg;
pub(crate) struct EnterAltScreenMsg;
pub(crate) struct ExitAltScreenMsg;
pub(crate) struct ShowCursorMsg;
pub(crate) struct HideCursorMsg;
pub(crate) struct EnableMouseCellMotionMsg;
pub(crate) struct EnableMouseAllMotionMsg;
pub(crate) struct DisableMouseMsg;
pub(crate) struct EnableBracketedPasteMsg;
pub(crate) struct DisableBracketedPasteMsg;
pub(crate) struct EnableReportFocusMsg;
pub(crate) struct DisableReportFocusMsg;
pub(crate) struct ReleaseTerminalMsg;
pub(crate) struct RestoreTerminalMsg;

/// Command to clear the screen.
pub fn clear_screen() -> Cmd {
    Cmd::new(|| Message::new(ClearScreenMsg))
}

/// Command to enter alternate screen buffer.
///
/// This provides a separate screen that preserves the original terminal
/// content when your program exits.
pub fn enter_alt_screen() -> Cmd {
    Cmd::new(|| Message::new(EnterAltScreenMsg))
}

/// Command to exit alternate screen buffer.
pub fn exit_alt_screen() -> Cmd {
    Cmd::new(|| Message::new(ExitAltScreenMsg))
}

/// Command to show the cursor.
pub fn show_cursor() -> Cmd {
    Cmd::new(|| Message::new(ShowCursorMsg))
}

/// Command to hide the cursor.
pub fn hide_cursor() -> Cmd {
    Cmd::new(|| Message::new(HideCursorMsg))
}

/// Command to enable mouse cell motion tracking.
///
/// This reports mouse clicks and drags.
pub fn enable_mouse_cell_motion() -> Cmd {
    Cmd::new(|| Message::new(EnableMouseCellMotionMsg))
}

/// Command to enable mouse all motion tracking.
///
/// This reports all mouse movement, including without button presses.
pub fn enable_mouse_all_motion() -> Cmd {
    Cmd::new(|| Message::new(EnableMouseAllMotionMsg))
}

/// Command to disable mouse tracking.
pub fn disable_mouse() -> Cmd {
    Cmd::new(|| Message::new(DisableMouseMsg))
}

/// Command to enable bracketed paste mode.
///
/// In bracketed paste mode, pasted text is wrapped in escape sequences,
/// allowing the application to distinguish typed text from pasted text.
pub fn enable_bracketed_paste() -> Cmd {
    Cmd::new(|| Message::new(EnableBracketedPasteMsg))
}

/// Command to disable bracketed paste mode.
pub fn disable_bracketed_paste() -> Cmd {
    Cmd::new(|| Message::new(DisableBracketedPasteMsg))
}

/// Command to enable focus reporting.
///
/// When enabled, the terminal sends FocusMsg and BlurMsg events.
pub fn enable_report_focus() -> Cmd {
    Cmd::new(|| Message::new(EnableReportFocusMsg))
}

/// Command to disable focus reporting.
pub fn disable_report_focus() -> Cmd {
    Cmd::new(|| Message::new(DisableReportFocusMsg))
}

/// Command to release the terminal for external processes.
///
/// This restores the terminal to its normal state:
/// - Disables raw mode
/// - Shows the cursor
/// - Exits alternate screen (if enabled)
/// - Disables mouse tracking
/// - Disables bracketed paste
/// - Disables focus reporting
///
/// Use this before spawning external processes like text editors.
/// Call `restore_terminal()` afterwards to resume the TUI.
///
/// # Example
///
/// ```rust,ignore
/// use bubbletea::{Model, Message, Cmd, screen, sequence};
/// use std::process::Command;
///
/// fn update(&mut self, msg: Message) -> Option<Cmd> {
///     if msg.is::<EditMsg>() {
///         return Some(sequence(vec![
///             Some(screen::release_terminal()),
///             Some(Cmd::new(|| {
///                 // Run editor
///                 Command::new("vim").arg("file.txt").status().ok();
///                 Message::new(EditorDoneMsg)
///             })),
///             Some(screen::restore_terminal()),
///         ]));
///     }
///     None
/// }
/// ```
pub fn release_terminal() -> Cmd {
    Cmd::new(|| Message::new(ReleaseTerminalMsg))
}

/// Command to restore the terminal after a release.
///
/// This re-enables the TUI state:
/// - Enables raw mode
/// - Hides the cursor
/// - Enters alternate screen (if originally enabled)
/// - Restores mouse tracking settings
/// - Restores bracketed paste mode
/// - Restores focus reporting
///
/// Use this after `release_terminal()` to resume the TUI.
pub fn restore_terminal() -> Cmd {
    Cmd::new(|| Message::new(RestoreTerminalMsg))
}

// Note: execute_screen_command could be used in the future for handling
// screen commands dynamically. For now, screen control is handled directly
// in the Program struct.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_commands_create() {
        // Just verify the commands can be created without panicking
        let _ = clear_screen();
        let _ = enter_alt_screen();
        let _ = exit_alt_screen();
        let _ = show_cursor();
        let _ = hide_cursor();
        let _ = enable_mouse_cell_motion();
        let _ = enable_mouse_all_motion();
        let _ = disable_mouse();
        let _ = enable_bracketed_paste();
        let _ = disable_bracketed_paste();
        let _ = enable_report_focus();
        let _ = disable_report_focus();
        let _ = release_terminal();
        let _ = restore_terminal();
    }
}
