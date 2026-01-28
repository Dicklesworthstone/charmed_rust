//! Canonical keymap and interaction model for `demo_showcase`.
//!
//! This module defines the opinionated keybindings that all pages and components
//! must conform to. The goal is a consistent, discoverable, premium TUI experience.
//!
//! # Design Principles
//!
//! 1. **Vim-inspired navigation**: `j`/`k` for vertical, `h`/`l` for horizontal
//! 2. **Discoverability**: `?` always shows help, hints in footer
//! 3. **Escape hatches**: `Esc` closes overlays, `q` quits
//! 4. **Muscle memory**: Arrow keys work alongside vim keys
//! 5. **Progressive disclosure**: Simple actions first, power-user keys available
//!
//! # Key Categories
//!
//! ## Global Keys (always active, highest priority)
//!
//! | Key       | Action              | Notes                          |
//! |-----------|---------------------|--------------------------------|
//! | `?`       | Toggle help overlay | Shows all keybindings          |
//! | `q`       | Quit application    | Blocked when in input mode     |
//! | `Esc`     | Close/back/cancel   | Context-dependent escape       |
//! | `Ctrl+C`  | Force quit          | Immediate exit                 |
//! | `/`       | Open command palette| Quick navigation + search      |
//! | `t`       | Cycle theme         | Dark → Light → Dracula         |
//! | `Tab`     | Focus next pane     | Cycles through focusable areas |
//! | `Shift+Tab`| Focus prev pane    | Reverse focus cycle            |
//! | `1`-`7`   | Jump to page        | Direct page navigation         |
//! | `[`       | Toggle sidebar      | Show/hide navigation           |
//!
//! ## Navigation Keys (lists, tables, trees)
//!
//! | Key       | Action              | Notes                          |
//! |-----------|---------------------|--------------------------------|
//! | `j` / `↓` | Move down           | Next item                      |
//! | `k` / `↑` | Move up             | Previous item                  |
//! | `h` / `←` | Collapse/left       | Tree collapse or horizontal    |
//! | `l` / `→` | Expand/right        | Tree expand or horizontal      |
//! | `g`       | Go to top           | First item                     |
//! | `G`       | Go to bottom        | Last item                      |
//! | `Ctrl+d`  | Half page down      | Faster scrolling               |
//! | `Ctrl+u`  | Half page up        | Faster scrolling               |
//! | `PgDown`  | Page down           | Full page scroll               |
//! | `PgUp`    | Page up             | Full page scroll               |
//! | `Home`    | Go to top           | Alternative to `g`             |
//! | `End`     | Go to bottom        | Alternative to `G`             |
//!
//! ## Selection & Action Keys
//!
//! | Key       | Action              | Notes                          |
//! |-----------|---------------------|--------------------------------|
//! | `Enter`   | Select/activate     | Primary action on focused item |
//! | `Space`   | Toggle selection    | Multi-select in lists          |
//! | `a`       | Select all          | When multi-select enabled      |
//! | `x`       | Clear selection     | Deselect all                   |
//! | `d`       | Delete selected     | With confirmation if needed    |
//! | `e`       | Edit selected       | Open editor for item           |
//! | `c`       | Copy selected       | Copy to clipboard              |
//! | `r`       | Refresh             | Reload current view data       |
//! | `n`       | New item            | Create new entry               |
//!
//! ## Search & Filter Keys
//!
//! | Key       | Action              | Notes                          |
//! |-----------|---------------------|--------------------------------|
//! | `/`       | Open search         | Focus search input             |
//! | `n`       | Next match          | When search active             |
//! | `N`       | Previous match      | When search active             |
//! | `Esc`     | Clear search        | Exit search mode               |
//!
//! ## Form/Input Keys
//!
//! | Key       | Action              | Notes                          |
//! |-----------|---------------------|--------------------------------|
//! | `Enter`   | Submit form         | When in form context           |
//! | `Tab`     | Next field          | Move to next input             |
//! | `Shift+Tab`| Previous field     | Move to previous input         |
//! | `Esc`     | Cancel form         | Discard and close              |
//! | `Ctrl+s`  | Save draft          | Save without submitting        |
//!
//! # Mouse Interactions
//!
//! | Action       | Behavior               | Notes                       |
//! |--------------|------------------------|-----------------------------|
//! | Left click   | Focus/select           | Click to activate element   |
//! | Double click | Primary action         | Same as Enter               |
//! | Right click  | Context menu           | If available                |
//! | Scroll       | Scroll viewport        | Wheel in scrollable areas   |
//! | Drag         | Resize/reorder         | For splitters/sortable      |

// Allow dead code - this module defines the keymap spec for future implementation
#![allow(dead_code)]

use bubbletea::{KeyMsg, KeyType};

/// Key binding categories for documentation and routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCategory {
    /// Global keys that work everywhere.
    Global,
    /// Navigation keys for lists/tables/trees.
    Navigation,
    /// Selection and action keys.
    Selection,
    /// Search and filter keys.
    Search,
    /// Form and input keys.
    Form,
}

/// Actions that can be triggered by keybindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    // Global
    /// Show help overlay.
    Help,
    /// Quit the application.
    Quit,
    /// Close overlay, cancel, or go back.
    Escape,
    /// Open command palette.
    CommandPalette,
    /// Cycle to next theme.
    CycleTheme,
    /// Focus next pane.
    FocusNext,
    /// Focus previous pane.
    FocusPrev,
    /// Toggle sidebar visibility.
    ToggleSidebar,
    /// Navigate to a specific page (1-7).
    GotoPage(u8),

    // Navigation
    /// Move cursor/selection down.
    Down,
    /// Move cursor/selection up.
    Up,
    /// Move cursor/selection left (or collapse in trees).
    Left,
    /// Move cursor/selection right (or expand in trees).
    Right,
    /// Go to first item.
    GotoTop,
    /// Go to last item.
    GotoBottom,
    /// Move half page down.
    HalfPageDown,
    /// Move half page up.
    HalfPageUp,
    /// Move full page down.
    PageDown,
    /// Move full page up.
    PageUp,

    // Selection
    /// Select/activate the current item.
    Select,
    /// Toggle selection on current item.
    Toggle,
    /// Select all items.
    SelectAll,
    /// Clear all selections.
    ClearSelection,
    /// Delete selected item(s).
    Delete,
    /// Edit selected item.
    Edit,
    /// Copy selected item(s).
    Copy,
    /// Refresh current view.
    Refresh,
    /// Create new item.
    New,

    // Search
    /// Open search/filter.
    Search,
    /// Go to next search match.
    NextMatch,
    /// Go to previous search match.
    PrevMatch,
    /// Clear search.
    ClearSearch,

    // Form
    /// Submit form.
    Submit,
    /// Save draft.
    SaveDraft,
    /// Next field (same as `FocusNext` in form context).
    NextField,
    /// Previous field.
    PrevField,
}

/// Check if a key event matches a global action.
///
/// Global actions are handled at the app level before page delegation.
#[must_use]
pub fn match_global(key: &KeyMsg) -> Option<KeyAction> {
    match key.key_type {
        KeyType::CtrlC => Some(KeyAction::Quit),
        KeyType::Esc => Some(KeyAction::Escape),
        KeyType::ShiftTab => Some(KeyAction::FocusPrev),
        KeyType::Tab => Some(KeyAction::FocusNext),
        KeyType::Runes => match key.runes.as_slice() {
            ['?'] => Some(KeyAction::Help),
            ['q'] => Some(KeyAction::Quit),
            ['/'] => Some(KeyAction::CommandPalette),
            ['t'] => Some(KeyAction::CycleTheme),
            ['['] => Some(KeyAction::ToggleSidebar),
            ['1'] => Some(KeyAction::GotoPage(1)),
            ['2'] => Some(KeyAction::GotoPage(2)),
            ['3'] => Some(KeyAction::GotoPage(3)),
            ['4'] => Some(KeyAction::GotoPage(4)),
            ['5'] => Some(KeyAction::GotoPage(5)),
            ['6'] => Some(KeyAction::GotoPage(6)),
            ['7'] => Some(KeyAction::GotoPage(7)),
            _ => None,
        },
        _ => None,
    }
}

/// Check if a key event matches a navigation action.
///
/// Navigation actions are used in lists, tables, and trees.
#[must_use]
pub fn match_navigation(key: &KeyMsg) -> Option<KeyAction> {
    match key.key_type {
        KeyType::Up => Some(KeyAction::Up),
        KeyType::Down => Some(KeyAction::Down),
        KeyType::Left => Some(KeyAction::Left),
        KeyType::Right => Some(KeyAction::Right),
        KeyType::Home => Some(KeyAction::GotoTop),
        KeyType::End => Some(KeyAction::GotoBottom),
        KeyType::PgUp => Some(KeyAction::PageUp),
        KeyType::PgDown => Some(KeyAction::PageDown),
        KeyType::Runes => match key.runes.as_slice() {
            ['j'] => Some(KeyAction::Down),
            ['k'] => Some(KeyAction::Up),
            ['h'] => Some(KeyAction::Left),
            ['l'] => Some(KeyAction::Right),
            ['g'] => Some(KeyAction::GotoTop),
            ['G'] => Some(KeyAction::GotoBottom),
            _ => None,
        },
        KeyType::CtrlD => Some(KeyAction::HalfPageDown),
        KeyType::CtrlU => Some(KeyAction::HalfPageUp),
        _ => None,
    }
}

/// Check if a key event matches a selection/action binding.
#[must_use]
pub fn match_selection(key: &KeyMsg) -> Option<KeyAction> {
    match key.key_type {
        KeyType::Enter => Some(KeyAction::Select),
        KeyType::Space => Some(KeyAction::Toggle),
        KeyType::Runes => match key.runes.as_slice() {
            ['a'] => Some(KeyAction::SelectAll),
            ['x'] => Some(KeyAction::ClearSelection),
            ['d'] => Some(KeyAction::Delete),
            ['e'] => Some(KeyAction::Edit),
            ['c'] => Some(KeyAction::Copy),
            ['r'] => Some(KeyAction::Refresh),
            ['n'] => Some(KeyAction::New),
            _ => None,
        },
        _ => None,
    }
}

/// Check if a key event matches a search action.
#[must_use]
pub fn match_search(key: &KeyMsg) -> Option<KeyAction> {
    match key.key_type {
        KeyType::Esc => Some(KeyAction::ClearSearch),
        KeyType::Runes => match key.runes.as_slice() {
            ['/'] => Some(KeyAction::Search),
            ['n'] => Some(KeyAction::NextMatch),
            ['N'] => Some(KeyAction::PrevMatch),
            _ => None,
        },
        _ => None,
    }
}

/// Check if a key event matches a form action.
#[must_use]
pub const fn match_form(key: &KeyMsg) -> Option<KeyAction> {
    match key.key_type {
        KeyType::Enter => Some(KeyAction::Submit),
        KeyType::Esc => Some(KeyAction::Escape),
        KeyType::ShiftTab => Some(KeyAction::PrevField),
        KeyType::Tab => Some(KeyAction::NextField),
        KeyType::CtrlS => Some(KeyAction::SaveDraft),
        _ => None,
    }
}

/// Get a human-readable label for a key action.
#[must_use]
pub const fn action_label(action: KeyAction) -> &'static str {
    match action {
        KeyAction::Help => "Help",
        KeyAction::Quit => "Quit",
        KeyAction::Escape => "Close/Back",
        KeyAction::CommandPalette => "Command Palette",
        KeyAction::CycleTheme => "Cycle Theme",
        KeyAction::FocusNext => "Focus Next",
        KeyAction::FocusPrev => "Focus Previous",
        KeyAction::ToggleSidebar => "Toggle Sidebar",
        KeyAction::GotoPage(_) => "Go to Page",
        KeyAction::Down => "Down",
        KeyAction::Up => "Up",
        KeyAction::Left => "Left/Collapse",
        KeyAction::Right => "Right/Expand",
        KeyAction::GotoTop => "Go to Top",
        KeyAction::GotoBottom => "Go to Bottom",
        KeyAction::HalfPageDown => "Half Page Down",
        KeyAction::HalfPageUp => "Half Page Up",
        KeyAction::PageDown => "Page Down",
        KeyAction::PageUp => "Page Up",
        KeyAction::Select => "Select",
        KeyAction::Toggle => "Toggle",
        KeyAction::SelectAll => "Select All",
        KeyAction::ClearSelection => "Clear Selection",
        KeyAction::Delete => "Delete",
        KeyAction::Edit => "Edit",
        KeyAction::Copy => "Copy",
        KeyAction::Refresh => "Refresh",
        KeyAction::New => "New",
        KeyAction::Search => "Search",
        KeyAction::NextMatch => "Next Match",
        KeyAction::PrevMatch => "Previous Match",
        KeyAction::ClearSearch => "Clear Search",
        KeyAction::Submit => "Submit",
        KeyAction::SaveDraft => "Save Draft",
        KeyAction::NextField => "Next Field",
        KeyAction::PrevField => "Previous Field",
    }
}

/// Get a short key hint string for an action.
#[must_use]
#[allow(clippy::match_same_arms)] // Intentional: same keys serve different purposes in different contexts
pub const fn action_hint(action: KeyAction) -> &'static str {
    match action {
        KeyAction::Help => "?",
        KeyAction::Quit => "q",
        KeyAction::Escape | KeyAction::ClearSearch => "Esc",
        KeyAction::CommandPalette | KeyAction::Search => "/",
        KeyAction::CycleTheme => "t",
        KeyAction::FocusNext | KeyAction::NextField => "Tab",
        KeyAction::FocusPrev | KeyAction::PrevField => "S-Tab",
        KeyAction::ToggleSidebar => "[",
        KeyAction::GotoPage(_) => "1-7",
        KeyAction::Down => "j/↓",
        KeyAction::Up => "k/↑",
        KeyAction::Left => "h/←",
        KeyAction::Right => "l/→",
        KeyAction::GotoTop => "g",
        KeyAction::GotoBottom => "G",
        KeyAction::HalfPageDown => "C-d",
        KeyAction::HalfPageUp => "C-u",
        KeyAction::PageDown => "PgDn",
        KeyAction::PageUp => "PgUp",
        KeyAction::Select | KeyAction::Submit => "Enter",
        KeyAction::Toggle => "Space",
        KeyAction::SelectAll => "a",
        KeyAction::ClearSelection => "x",
        KeyAction::Delete => "d",
        KeyAction::Edit => "e",
        KeyAction::Copy => "c",
        KeyAction::Refresh => "r",
        KeyAction::New | KeyAction::NextMatch => "n",
        KeyAction::PrevMatch => "N",
        KeyAction::SaveDraft => "C-s",
    }
}

/// Standard hint strings for common component types.
pub mod hints {
    /// Hints for list-style components.
    pub const LIST: &str = "j/k navigate  Enter select  g/G top/bottom";

    /// Hints for table-style components.
    pub const TABLE: &str = "j/k navigate  Enter details  r refresh";

    /// Hints for tree-style components.
    pub const TREE: &str = "j/k navigate  h/l collapse/expand  Enter select";

    /// Hints for form components.
    pub const FORM: &str = "Tab next field  Enter submit  Esc cancel";

    /// Hints for text input components.
    pub const INPUT: &str = "Enter confirm  Esc cancel";

    /// Hints for modal dialogs.
    pub const MODAL: &str = "Enter confirm  Esc close";

    /// Hints for search mode.
    pub const SEARCH: &str = "n/N next/prev match  Esc clear";

    /// Hints for viewport/scroll components.
    pub const VIEWPORT: &str = "j/k scroll  g/G top/bottom  C-d/C-u half page";
}

// ============================================================================
// Help Overlay Content Generation
// ============================================================================

/// A single keybinding entry for help display.
#[derive(Debug, Clone)]
pub struct HelpEntry {
    /// The key combination (e.g., "j/↓", "Ctrl+C").
    pub key: &'static str,
    /// The action description (e.g., "Move down").
    pub action: &'static str,
}

impl HelpEntry {
    /// Create a new help entry.
    #[must_use]
    pub const fn new(key: &'static str, action: &'static str) -> Self {
        Self { key, action }
    }
}

/// A section of keybindings for help display.
#[derive(Debug, Clone)]
pub struct HelpSection {
    /// Section title (e.g., "Global", "Navigation").
    pub title: &'static str,
    /// Keybinding entries in this section.
    pub entries: &'static [HelpEntry],
}

/// Global keybindings section.
pub const HELP_GLOBAL: HelpSection = HelpSection {
    title: "Global",
    entries: &[
        HelpEntry::new("?", "Toggle this help"),
        HelpEntry::new("q / Esc", "Quit application"),
        HelpEntry::new("Ctrl+C", "Force quit"),
        HelpEntry::new("1-7", "Jump to page"),
        HelpEntry::new("[", "Toggle sidebar"),
        HelpEntry::new("t", "Cycle theme"),
        HelpEntry::new("Tab", "Focus next pane"),
        HelpEntry::new("S-Tab", "Focus previous pane"),
    ],
};

/// Navigation keybindings section.
pub const HELP_NAVIGATION: HelpSection = HelpSection {
    title: "Navigation",
    entries: &[
        HelpEntry::new("j / ↓", "Move down"),
        HelpEntry::new("k / ↑", "Move up"),
        HelpEntry::new("h / ←", "Left / collapse"),
        HelpEntry::new("l / →", "Right / expand"),
        HelpEntry::new("g / Home", "Go to top"),
        HelpEntry::new("G / End", "Go to bottom"),
        HelpEntry::new("Ctrl+d", "Half page down"),
        HelpEntry::new("Ctrl+u", "Half page up"),
        HelpEntry::new("PgDn", "Page down"),
        HelpEntry::new("PgUp", "Page up"),
    ],
};

/// Selection and action keybindings section.
pub const HELP_SELECTION: HelpSection = HelpSection {
    title: "Selection & Actions",
    entries: &[
        HelpEntry::new("Enter", "Select / activate"),
        HelpEntry::new("Space", "Toggle selection"),
        HelpEntry::new("a", "Select all"),
        HelpEntry::new("x", "Clear selection"),
        HelpEntry::new("d", "Delete selected"),
        HelpEntry::new("e", "Edit selected"),
        HelpEntry::new("c", "Copy selected"),
        HelpEntry::new("r", "Refresh view"),
        HelpEntry::new("n", "New item"),
    ],
};

/// Search keybindings section.
pub const HELP_SEARCH: HelpSection = HelpSection {
    title: "Search",
    entries: &[
        HelpEntry::new("/", "Open search"),
        HelpEntry::new("n", "Next match"),
        HelpEntry::new("N", "Previous match"),
        HelpEntry::new("Esc", "Clear search"),
    ],
};

/// Mouse interaction section.
pub const HELP_MOUSE: HelpSection = HelpSection {
    title: "Mouse (when enabled)",
    entries: &[
        HelpEntry::new("Click", "Focus / select"),
        HelpEntry::new("Double-click", "Primary action"),
        HelpEntry::new("Scroll", "Scroll viewport"),
    ],
};

/// All help sections in display order.
pub const HELP_SECTIONS: &[&HelpSection] = &[
    &HELP_GLOBAL,
    &HELP_NAVIGATION,
    &HELP_SELECTION,
    &HELP_SEARCH,
    &HELP_MOUSE,
];

/// Calculate the total number of lines in the help content.
///
/// Each section has: title line + entries + blank line.
#[must_use]
pub fn help_total_lines() -> usize {
    let mut lines = 0;
    for section in HELP_SECTIONS {
        lines += 1; // Section title
        lines += section.entries.len(); // Entries
        lines += 1; // Blank line after section
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_rune(c: char) -> KeyMsg {
        KeyMsg {
            key_type: KeyType::Runes,
            runes: vec![c],
            alt: false,
            paste: false,
        }
    }

    fn key_type(t: KeyType) -> KeyMsg {
        KeyMsg {
            key_type: t,
            runes: vec![],
            alt: false,
            paste: false,
        }
    }

    #[test]
    fn global_help() {
        assert_eq!(match_global(&key_rune('?')), Some(KeyAction::Help));
    }

    #[test]
    fn global_quit() {
        assert_eq!(match_global(&key_rune('q')), Some(KeyAction::Quit));
        assert_eq!(
            match_global(&key_type(KeyType::CtrlC)),
            Some(KeyAction::Quit)
        );
    }

    #[test]
    fn global_page_navigation() {
        assert_eq!(match_global(&key_rune('1')), Some(KeyAction::GotoPage(1)));
        assert_eq!(match_global(&key_rune('7')), Some(KeyAction::GotoPage(7)));
    }

    #[test]
    fn navigation_vim_keys() {
        assert_eq!(match_navigation(&key_rune('j')), Some(KeyAction::Down));
        assert_eq!(match_navigation(&key_rune('k')), Some(KeyAction::Up));
        assert_eq!(match_navigation(&key_rune('h')), Some(KeyAction::Left));
        assert_eq!(match_navigation(&key_rune('l')), Some(KeyAction::Right));
    }

    #[test]
    fn navigation_arrow_keys() {
        assert_eq!(
            match_navigation(&key_type(KeyType::Up)),
            Some(KeyAction::Up)
        );
        assert_eq!(
            match_navigation(&key_type(KeyType::Down)),
            Some(KeyAction::Down)
        );
    }

    #[test]
    fn navigation_goto() {
        assert_eq!(match_navigation(&key_rune('g')), Some(KeyAction::GotoTop));
        assert_eq!(
            match_navigation(&key_rune('G')),
            Some(KeyAction::GotoBottom)
        );
    }

    #[test]
    fn selection_actions() {
        assert_eq!(
            match_selection(&key_type(KeyType::Enter)),
            Some(KeyAction::Select)
        );
        assert_eq!(match_selection(&key_rune('r')), Some(KeyAction::Refresh));
    }

    #[test]
    fn action_labels_are_nonempty() {
        assert!(!action_label(KeyAction::Help).is_empty());
        assert!(!action_label(KeyAction::Quit).is_empty());
    }

    #[test]
    fn action_hints_are_nonempty() {
        assert!(!action_hint(KeyAction::Help).is_empty());
        assert!(!action_hint(KeyAction::Down).is_empty());
    }
}
