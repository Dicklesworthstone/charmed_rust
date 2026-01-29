//! Notes scratchpad modal component.
//!
//! A modal dialog for writing incident notes, deploy notes, or any
//! multi-line text. Demonstrates `bubbles::TextArea` integration.
//!
//! # Features
//! - Multi-line text editing with line numbers
//! - Copy note content to clipboard
//! - Clear note content
//! - Save note (emits log entry + toast)
//!
//! # Example
//!
//! ```ignore
//! let mut notes = NotesModal::new();
//! notes.open();
//!
//! // In update:
//! if let Some(msg) = msg.downcast_ref::<NotesModalMsg>() {
//!     match msg {
//!         NotesModalMsg::Saved(content) => {
//!             // Handle saved note
//!         }
//!         _ => {}
//!     }
//! }
//! ```

use bubbles::textarea::TextArea;
use bubbletea::{Cmd, KeyMsg, KeyType, Message};
use lipgloss::{Border, Style};

use crate::theme::Theme;

/// Messages emitted by the NotesModal.
#[derive(Debug, Clone)]
pub enum NotesModalMsg {
    /// Note was saved with this content.
    Saved(String),
    /// Note was cleared.
    Cleared,
    /// Note content was copied.
    Copied(String),
    /// Modal was closed without saving.
    Closed,
}

impl NotesModalMsg {
    /// Convert to a bubbletea Message.
    #[must_use]
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

/// Notes scratchpad modal state.
#[derive(Debug, Clone)]
pub struct NotesModal {
    /// The text area for note editing.
    textarea: TextArea,
    /// Whether the modal is currently open.
    open: bool,
    /// Modal width.
    width: usize,
    /// Modal height.
    height: usize,
    /// Title for the modal.
    title: String,
}

impl Default for NotesModal {
    fn default() -> Self {
        Self::new()
    }
}

impl NotesModal {
    /// Create a new notes modal.
    #[must_use]
    pub fn new() -> Self {
        let mut textarea = TextArea::new();
        textarea.placeholder = "Write your note here...".to_string();
        textarea.show_line_numbers = true;
        textarea.prompt = "│ ".to_string();
        textarea.focus();

        Self {
            textarea,
            open: false,
            width: 60,
            height: 15,
            title: "Notes".to_string(),
        }
    }

    /// Open the modal.
    pub fn open(&mut self) {
        self.open = true;
        self.textarea.focus();
    }

    /// Close the modal.
    pub fn close(&mut self) {
        self.open = false;
        self.textarea.blur();
    }

    /// Check if the modal is open.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// Set the modal dimensions.
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width.max(40);
        self.height = height.max(10);
        // Leave room for border and header/footer
        self.textarea.set_width(self.width.saturating_sub(4));
        self.textarea.set_height(self.height.saturating_sub(6));
    }

    /// Set the modal title.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Get the current note content.
    #[must_use]
    pub fn content(&self) -> String {
        self.textarea.value()
    }

    /// Set the note content.
    pub fn set_content(&mut self, content: &str) {
        self.textarea.set_value(content);
    }

    /// Clear the note content.
    pub fn clear(&mut self) {
        self.textarea.reset();
    }

    /// Handle input when modal is open.
    ///
    /// Returns a command if an action was taken.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        if !self.open {
            return None;
        }

        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            match key.key_type {
                KeyType::Esc => {
                    // Close without saving
                    self.close();
                    return Some(Cmd::new(|| NotesModalMsg::Closed.into_message()));
                }
                KeyType::CtrlS => {
                    // Save note
                    let content = self.content();
                    if !content.is_empty() {
                        self.close();
                        return Some(Cmd::new(move || {
                            NotesModalMsg::Saved(content).into_message()
                        }));
                    }
                    return None;
                }
                KeyType::CtrlC => {
                    // Copy note content
                    let content = self.content();
                    if !content.is_empty() {
                        return Some(Cmd::new(move || {
                            NotesModalMsg::Copied(content).into_message()
                        }));
                    }
                    return None;
                }
                KeyType::CtrlX => {
                    // Clear note
                    self.clear();
                    return Some(Cmd::new(|| NotesModalMsg::Cleared.into_message()));
                }
                _ => {}
            }
        }

        // Pass to textarea for normal editing
        self.textarea.update(msg)
    }

    /// Render the modal.
    #[must_use]
    pub fn view(&self, theme: &Theme) -> String {
        if !self.open {
            return String::new();
        }

        // Header
        let title = theme.header_style().render(&format!(" {} ", self.title));
        let header = format!(
            "{}{}",
            title,
            theme.muted_style().render(&" ".repeat(self.width.saturating_sub(self.title.len() + 2)))
        );

        // Textarea
        let textarea_view = self.textarea.view();

        // Footer with hints
        let hints = theme.muted_style().render("Ctrl+S save  Ctrl+C copy  Ctrl+X clear  Esc close");
        let char_count = theme.muted_style().render(&format!(
            "{} chars",
            self.textarea.length()
        ));

        let footer_left = hints;
        let footer_right = char_count;
        let footer_padding = self.width.saturating_sub(
            lipgloss::visible_width(&footer_left) +
            lipgloss::visible_width(&footer_right) + 2
        );
        let footer = format!(
            "{}{}{}",
            footer_left,
            " ".repeat(footer_padding),
            footer_right
        );

        // Combine into modal box
        let content = format!("{}\n\n{}\n\n{}", header, textarea_view, footer);

        // Apply modal styling with border
        let modal_style = Style::new()
            .border(Border::rounded())
            .border_foreground(theme.border)
            .padding_left(1)
            .padding_right(1)
            .width(self.width as u16);

        modal_style.render(&content)
    }

    /// Render the modal centered on the screen.
    #[must_use]
    pub fn view_centered(&self, theme: &Theme, screen_width: usize, screen_height: usize) -> String {
        if !self.open {
            return String::new();
        }

        let modal = self.view(theme);
        let modal_lines: Vec<&str> = modal.lines().collect();
        let modal_height = modal_lines.len();
        let modal_width = modal_lines.iter()
            .map(|l| lipgloss::visible_width(l))
            .max()
            .unwrap_or(0);

        // Calculate centering offsets
        let top_padding = screen_height.saturating_sub(modal_height) / 2;
        let left_padding = screen_width.saturating_sub(modal_width) / 2;

        // Build centered view
        let mut lines = Vec::with_capacity(screen_height);

        // Top padding
        for _ in 0..top_padding {
            lines.push(String::new());
        }

        // Modal content with left padding
        let left_pad = " ".repeat(left_padding);
        for line in modal_lines {
            lines.push(format!("{}{}", left_pad, line));
        }

        // Bottom padding
        while lines.len() < screen_height {
            lines.push(String::new());
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_modal_creates() {
        let modal = NotesModal::new();
        assert!(!modal.is_open());
        assert!(modal.content().is_empty());
    }

    #[test]
    fn notes_modal_open_close() {
        let mut modal = NotesModal::new();
        assert!(!modal.is_open());

        modal.open();
        assert!(modal.is_open());

        modal.close();
        assert!(!modal.is_open());
    }

    #[test]
    fn notes_modal_content() {
        let mut modal = NotesModal::new();
        modal.set_content("Test note content");
        assert_eq!(modal.content(), "Test note content");

        modal.clear();
        assert!(modal.content().is_empty());
    }

    #[test]
    fn notes_modal_set_title() {
        let mut modal = NotesModal::new();
        modal.set_title("Incident Report");
        assert_eq!(modal.title, "Incident Report");
    }

    #[test]
    fn notes_modal_set_size() {
        let mut modal = NotesModal::new();
        modal.set_size(80, 20);
        assert_eq!(modal.width, 80);
        assert_eq!(modal.height, 20);

        // Test minimum size clamping
        modal.set_size(10, 5);
        assert_eq!(modal.width, 40);
        assert_eq!(modal.height, 10);
    }

    #[test]
    fn notes_modal_view_when_closed() {
        let modal = NotesModal::new();
        let theme = Theme::dark();
        let view = modal.view(&theme);
        assert!(view.is_empty());
    }

    #[test]
    fn notes_modal_view_when_open() {
        let mut modal = NotesModal::new();
        modal.open();
        let theme = Theme::dark();
        let view = modal.view(&theme);
        assert!(!view.is_empty());
        assert!(view.contains("Notes") || view.contains("save"));
    }

    #[test]
    fn notes_modal_update_when_closed() {
        let mut modal = NotesModal::new();
        let key = KeyMsg::from_char('a');
        let result = modal.update(Message::new(key));
        assert!(result.is_none());
    }

    #[test]
    fn notes_modal_esc_closes() {
        let mut modal = NotesModal::new();
        modal.open();
        assert!(modal.is_open());

        let key = KeyMsg::from_type(KeyType::Esc);
        let result = modal.update(Message::new(key));
        assert!(result.is_some()); // Should emit Closed message
        assert!(!modal.is_open());
    }

    #[test]
    fn notes_modal_centered_view() {
        let mut modal = NotesModal::new();
        modal.open();
        modal.set_size(40, 10);
        let theme = Theme::dark();
        let view = modal.view_centered(&theme, 80, 24);
        assert!(!view.is_empty());
        // Should have some empty lines for centering
        let lines: Vec<&str> = view.lines().collect();
        assert!(lines.len() <= 24);
    }
}
