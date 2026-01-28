//! Message taxonomy for `demo_showcase`.
//!
//! This module defines all message types used in the application, providing
//! a clean taxonomy that minimizes ad-hoc `Any` downcasts.

use std::time::Instant;

use bubbletea::Message;

use crate::components::StatusLevel;
use crate::theme::ThemePreset;

/// Application-level messages for routing and global state changes.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants will be used in future implementations
pub enum AppMsg {
    /// Navigate to a different page.
    Navigate(Page),
    /// Toggle sidebar visibility.
    ToggleSidebar,
    /// Toggle animations on/off.
    ToggleAnimations,
    /// Toggle mouse input on/off.
    ToggleMouse,
    /// Toggle syntax highlighting on/off.
    ToggleSyntax,
    /// Force ASCII mode (no colors, ASCII borders).
    ForceAscii(bool),
    /// Change the application theme.
    SetTheme(ThemePreset),
    /// Cycle to the next theme preset.
    CycleTheme,
    /// Show help overlay.
    ShowHelp,
    /// Hide help overlay.
    HideHelp,
    /// Quit the application.
    Quit,
}

impl AppMsg {
    /// Create a bubbletea Message from this `AppMsg`.
    #[must_use]
    #[allow(dead_code)] // Will be used in future implementations
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

/// Available pages in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    /// Dashboard - platform health overview.
    #[default]
    Dashboard,
    /// Services - service catalog.
    Services,
    /// Jobs - background task monitoring.
    Jobs,
    /// Logs - aggregated log viewer.
    Logs,
    /// Docs - documentation browser.
    Docs,
    /// Wizard - multi-step workflows.
    Wizard,
    /// Settings - preferences and about.
    Settings,
}

impl Page {
    /// Get the display name for this page.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Services => "Services",
            Self::Jobs => "Jobs",
            Self::Logs => "Logs",
            Self::Docs => "Docs",
            Self::Wizard => "Wizard",
            Self::Settings => "Settings",
        }
    }

    /// Get the keyboard shortcut for this page (1-7).
    #[must_use]
    #[allow(dead_code)] // Will be used in future implementations
    pub const fn shortcut(self) -> char {
        match self {
            Self::Dashboard => '1',
            Self::Services => '2',
            Self::Jobs => '3',
            Self::Logs => '4',
            Self::Docs => '5',
            Self::Wizard => '6',
            Self::Settings => '7',
        }
    }

    /// Get page from keyboard shortcut.
    #[must_use]
    pub const fn from_shortcut(c: char) -> Option<Self> {
        match c {
            '1' => Some(Self::Dashboard),
            '2' => Some(Self::Services),
            '3' => Some(Self::Jobs),
            '4' => Some(Self::Logs),
            '5' => Some(Self::Docs),
            '6' => Some(Self::Wizard),
            '7' => Some(Self::Settings),
            _ => None,
        }
    }

    /// Get all pages in navigation order.
    #[must_use]
    pub const fn all() -> [Self; 7] {
        [
            Self::Dashboard,
            Self::Services,
            Self::Jobs,
            Self::Logs,
            Self::Docs,
            Self::Wizard,
            Self::Settings,
        ]
    }

    /// Get the icon for this page.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "[]",
            Self::Services => ">_",
            Self::Jobs => ">>",
            Self::Logs => " #",
            Self::Docs => " ?",
            Self::Wizard => " *",
            Self::Settings => " @",
        }
    }
}

/// Page-level messages that are handled by individual page models.
///
/// These are wrapped in the appropriate page context before dispatch.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in future page implementations
pub enum PageMsg {
    /// Tick for animations and updates.
    Tick,
    /// Search/filter input changed.
    FilterChanged(String),
    /// Item selected in a list.
    ItemSelected(usize),
    /// Action triggered (page-specific meaning).
    Action(String),
    /// Scroll viewport.
    Scroll(ScrollDirection),
}

/// Scroll direction for viewports.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Will be used with PageMsg
pub enum ScrollDirection {
    Up,
    Down,
    PageUp,
    PageDown,
    Top,
    Bottom,
}

// ============================================================================
// Notifications
// ============================================================================

/// A notification/toast to display to the user.
///
/// Notifications are transient messages that communicate state changes,
/// confirmations, or errors. They appear at the bottom of the content area
/// and can auto-dismiss or require manual dismissal.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields/methods will be used as pages are implemented
pub struct Notification {
    /// Unique identifier for this notification.
    pub id: u64,
    /// The message to display.
    pub message: String,
    /// Severity/type of notification.
    pub level: StatusLevel,
    /// When this notification was created.
    pub created_at: Instant,
    /// Optional action hint (e.g., "Press Enter to dismiss").
    pub action_hint: Option<String>,
}

#[allow(dead_code)] // Helpers will be used as pages are implemented
impl Notification {
    /// Create a new notification with the given parameters.
    #[must_use]
    pub fn new(id: u64, message: impl Into<String>, level: StatusLevel) -> Self {
        Self {
            id,
            message: message.into(),
            level,
            created_at: Instant::now(),
            action_hint: None,
        }
    }

    /// Create a success notification.
    #[must_use]
    pub fn success(id: u64, message: impl Into<String>) -> Self {
        Self::new(id, message, StatusLevel::Success)
    }

    /// Create a warning notification.
    #[must_use]
    pub fn warning(id: u64, message: impl Into<String>) -> Self {
        Self::new(id, message, StatusLevel::Warning)
    }

    /// Create an error notification.
    #[must_use]
    pub fn error(id: u64, message: impl Into<String>) -> Self {
        Self::new(id, message, StatusLevel::Error)
    }

    /// Create an info notification.
    #[must_use]
    pub fn info(id: u64, message: impl Into<String>) -> Self {
        Self::new(id, message, StatusLevel::Info)
    }

    /// Add an action hint to this notification.
    #[must_use]
    pub fn with_action_hint(mut self, hint: impl Into<String>) -> Self {
        self.action_hint = Some(hint.into());
        self
    }
}

/// Notification-related messages.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants will be used as pages are implemented
pub enum NotificationMsg {
    /// Show a new notification.
    Show(Notification),
    /// Dismiss a notification by ID.
    Dismiss(u64),
    /// Dismiss the oldest notification.
    DismissOldest,
    /// Clear all notifications.
    ClearAll,
}

#[allow(dead_code)] // Will be used as pages are implemented
impl NotificationMsg {
    /// Create a bubbletea Message from this `NotificationMsg`.
    #[must_use]
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}

// ============================================================================
// Export
// ============================================================================

/// Format for exporting the current view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Plain text (ANSI codes stripped).
    PlainText,
    /// HTML with inline styles.
    Html,
}

impl ExportFormat {
    /// Get the file extension for this format.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::PlainText => "txt",
            Self::Html => "html",
        }
    }
}

/// Export-related messages.
#[derive(Debug, Clone)]
pub enum ExportMsg {
    /// Export the current view to file.
    Export(ExportFormat),
    /// Export completed successfully.
    ExportCompleted(String),
    /// Export failed.
    ExportFailed(String),
}

impl ExportMsg {
    /// Create a bubbletea Message from this `ExportMsg`.
    #[must_use]
    pub fn into_message(self) -> Message {
        Message::new(self)
    }
}
