//! Feature-rich list component with filtering and pagination.
//!
//! This module provides a list widget with optional filtering, pagination,
//! help display, status messages, and spinner for TUI applications.
//!
//! # Example
//!
//! ```rust
//! use bubbles::list::{List, Item, DefaultDelegate};
//!
//! #[derive(Clone)]
//! struct MyItem {
//!     title: String,
//!     description: String,
//! }
//!
//! impl Item for MyItem {
//!     fn filter_value(&self) -> &str {
//!         &self.title
//!     }
//! }
//!
//! let items = vec![
//!     MyItem { title: "Apple".into(), description: "A fruit".into() },
//!     MyItem { title: "Banana".into(), description: "Another fruit".into() },
//! ];
//!
//! let list = List::new(items, DefaultDelegate::new(), 80, 24);
//! ```

use crate::help::Help;
use crate::key::{Binding, matches};
use crate::paginator::Paginator;
use crate::spinner::{SpinnerModel, TickMsg};
use crate::textinput::TextInput;
use bubbletea::{Cmd, KeyMsg, Message, Model};
use lipgloss::{Color, Style};
use std::time::Duration;

/// Trait for items that can be displayed in a list.
pub trait Item: Clone + Send + 'static {
    /// Returns the value used for filtering.
    fn filter_value(&self) -> &str;
}

/// Trait for rendering list items.
pub trait ItemDelegate<I: Item>: Clone + Send + 'static {
    /// Returns the height of each item in lines.
    fn height(&self) -> usize;

    /// Returns the spacing between items.
    fn spacing(&self) -> usize;

    /// Renders an item.
    fn render(&self, item: &I, index: usize, selected: bool, width: usize) -> String;

    /// Updates the delegate (optional).
    fn update(&mut self, _msg: &Message, _item: &mut I) -> Option<Cmd> {
        None
    }
}

/// Default delegate for simple item rendering.
#[derive(Debug, Clone)]
pub struct DefaultDelegate {
    /// Style for normal items.
    pub normal_style: Style,
    /// Style for selected items.
    pub selected_style: Style,
    /// Height of each item.
    pub item_height: usize,
    /// Spacing between items.
    pub item_spacing: usize,
}

impl Default for DefaultDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultDelegate {
    /// Creates a new default delegate.
    #[must_use]
    pub fn new() -> Self {
        Self {
            normal_style: Style::new(),
            selected_style: Style::new().foreground_color(Color::from("212")).bold(),
            item_height: 1,
            item_spacing: 0,
        }
    }

    /// Sets the item height.
    #[must_use]
    pub fn with_height(mut self, h: usize) -> Self {
        self.item_height = h;
        self
    }

    /// Sets the item spacing.
    #[must_use]
    pub fn with_spacing(mut self, s: usize) -> Self {
        self.item_spacing = s;
        self
    }
}

impl<I: Item> ItemDelegate<I> for DefaultDelegate {
    fn height(&self) -> usize {
        self.item_height
    }

    fn spacing(&self) -> usize {
        self.item_spacing
    }

    fn render(&self, item: &I, _index: usize, selected: bool, width: usize) -> String {
        let value = item.filter_value();
        let truncated = if value.len() > width {
            format!("{}…", &value[..width.saturating_sub(1)])
        } else {
            value.to_string()
        };

        if selected {
            self.selected_style.render(&truncated)
        } else {
            self.normal_style.render(&truncated)
        }
    }
}

/// Represents a match rank from filtering.
#[derive(Debug, Clone)]
pub struct Rank {
    /// Index of the item in the original list.
    pub index: usize,
    /// Indices of matched characters.
    pub matched_indices: Vec<usize>,
}

/// Filter state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterState {
    /// No filter applied.
    Unfiltered,
    /// User is actively filtering.
    Filtering,
    /// Filter has been applied.
    FilterApplied,
}

impl std::fmt::Display for FilterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unfiltered => write!(f, "unfiltered"),
            Self::Filtering => write!(f, "filtering"),
            Self::FilterApplied => write!(f, "filter applied"),
        }
    }
}

/// Type alias for filter functions.
pub type FilterFn = Box<dyn Fn(&str, &[String]) -> Vec<Rank> + Send + Sync>;

/// Default filter using simple substring matching.
pub fn default_filter(term: &str, targets: &[String]) -> Vec<Rank> {
    let term_lower = term.to_lowercase();
    targets
        .iter()
        .enumerate()
        .filter(|(_, target)| target.to_lowercase().contains(&term_lower))
        .map(|(index, target)| {
            // Find match indices
            let target_lower = target.to_lowercase();
            let start = target_lower.find(&term_lower).unwrap_or(0);
            let matched_indices: Vec<usize> = (start..start + term.len()).collect();
            Rank {
                index,
                matched_indices,
            }
        })
        .collect()
}

/// Key bindings for list navigation.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Move cursor up.
    pub cursor_up: Binding,
    /// Move cursor down.
    pub cursor_down: Binding,
    /// Next page.
    pub next_page: Binding,
    /// Previous page.
    pub prev_page: Binding,
    /// Go to start.
    pub goto_start: Binding,
    /// Go to end.
    pub goto_end: Binding,
    /// Start filtering.
    pub filter: Binding,
    /// Clear filter.
    pub clear_filter: Binding,
    /// Cancel filtering.
    pub cancel_while_filtering: Binding,
    /// Accept filter.
    pub accept_while_filtering: Binding,
    /// Show full help.
    pub show_full_help: Binding,
    /// Close full help.
    pub close_full_help: Binding,
    /// Quit.
    pub quit: Binding,
    /// Force quit.
    pub force_quit: Binding,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            cursor_up: Binding::new().keys(&["up", "k"]).help("↑/k", "up"),
            cursor_down: Binding::new().keys(&["down", "j"]).help("↓/j", "down"),
            next_page: Binding::new()
                .keys(&["right", "l", "pgdown"])
                .help("→/l", "next page"),
            prev_page: Binding::new()
                .keys(&["left", "h", "pgup"])
                .help("←/h", "prev page"),
            goto_start: Binding::new().keys(&["home", "g"]).help("g/home", "start"),
            goto_end: Binding::new().keys(&["end", "G"]).help("G/end", "end"),
            filter: Binding::new().keys(&["/"]).help("/", "filter"),
            clear_filter: Binding::new().keys(&["esc"]).help("esc", "clear filter"),
            cancel_while_filtering: Binding::new().keys(&["esc"]).help("esc", "cancel"),
            accept_while_filtering: Binding::new()
                .keys(&["enter"])
                .help("enter", "apply filter"),
            show_full_help: Binding::new().keys(&["?"]).help("?", "help"),
            close_full_help: Binding::new()
                .keys(&["esc", "?"])
                .help("?/esc", "close help"),
            quit: Binding::new().keys(&["q"]).help("q", "quit"),
            force_quit: Binding::new()
                .keys(&["ctrl+c"])
                .help("ctrl+c", "force quit"),
        }
    }
}

/// Styles for the list.
#[derive(Debug, Clone)]
pub struct Styles {
    /// Title style.
    pub title: Style,
    /// Title bar style.
    pub title_bar: Style,
    /// Filter prompt style.
    pub filter_prompt: Style,
    /// Filter cursor style.
    pub filter_cursor: Style,
    /// Status bar style.
    pub status_bar: Style,
    /// Status empty style.
    pub status_empty: Style,
    /// No items style.
    pub no_items: Style,
    /// Pagination style.
    pub pagination: Style,
    /// Help style.
    pub help: Style,
    /// Active pagination dot.
    pub active_pagination_dot: Style,
    /// Inactive pagination dot.
    pub inactive_pagination_dot: Style,
    /// Divider dot.
    pub divider_dot: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            title: Style::new().bold(),
            title_bar: Style::new(),
            filter_prompt: Style::new(),
            filter_cursor: Style::new(),
            status_bar: Style::new().foreground_color(Color::from("240")),
            status_empty: Style::new().foreground_color(Color::from("240")),
            no_items: Style::new().foreground_color(Color::from("240")),
            pagination: Style::new(),
            help: Style::new().foreground_color(Color::from("240")),
            active_pagination_dot: Style::new().foreground_color(Color::from("212")),
            inactive_pagination_dot: Style::new().foreground_color(Color::from("240")),
            divider_dot: Style::new().foreground_color(Color::from("240")),
        }
    }
}

/// Message for filter matches.
#[derive(Debug, Clone)]
pub struct FilterMatchesMsg(pub Vec<Rank>);

/// Message for status message timeout.
#[derive(Debug, Clone, Copy)]
pub struct StatusMessageTimeoutMsg;

/// List model with filtering, pagination, and more.
#[derive(Clone)]
pub struct List<I: Item, D: ItemDelegate<I>> {
    /// Title of the list.
    pub title: String,
    /// Whether to show the title.
    pub show_title: bool,
    /// Whether to show the filter input.
    pub show_filter: bool,
    /// Whether to show the status bar.
    pub show_status_bar: bool,
    /// Whether to show pagination.
    pub show_pagination: bool,
    /// Whether to show help.
    pub show_help: bool,
    /// Whether filtering is enabled.
    pub filtering_enabled: bool,
    /// Whether infinite scrolling is enabled.
    pub infinite_scrolling: bool,
    /// Singular name for items.
    pub item_name_singular: String,
    /// Plural name for items.
    pub item_name_plural: String,
    /// Key bindings.
    pub key_map: KeyMap,
    /// Styles.
    pub styles: Styles,
    /// Status message lifetime.
    pub status_message_lifetime: Duration,

    // Components
    /// Spinner for loading state.
    spinner: SpinnerModel,
    /// Paginator.
    paginator: Paginator,
    /// Help view.
    help: Help,
    /// Filter input.
    filter_input: TextInput,

    // State
    items: Vec<I>,
    filtered_indices: Vec<usize>,
    delegate: D,
    width: usize,
    height: usize,
    cursor: usize,
    filter_state: FilterState,
    show_spinner: bool,
    status_message: Option<String>,
}

impl<I: Item, D: ItemDelegate<I>> List<I, D> {
    /// Creates a new list with the given items and delegate.
    #[must_use]
    pub fn new(items: Vec<I>, delegate: D, width: usize, height: usize) -> Self {
        let items_len = items.len();
        let filtered_indices: Vec<usize> = (0..items_len).collect();

        let mut paginator = Paginator::new().per_page(10);
        paginator.set_total_pages_from_items(items_len);

        let mut filter_input = TextInput::new();
        filter_input.prompt = "Filter: ".to_string();

        Self {
            title: String::new(),
            show_title: true,
            show_filter: true,
            show_status_bar: true,
            show_pagination: true,
            show_help: true,
            filtering_enabled: true,
            infinite_scrolling: false,
            item_name_singular: "item".to_string(),
            item_name_plural: "items".to_string(),
            key_map: KeyMap::default(),
            styles: Styles::default(),
            status_message_lifetime: Duration::from_secs(1),
            spinner: SpinnerModel::new(),
            paginator,
            help: Help::new(),
            filter_input,
            items,
            filtered_indices,
            delegate,
            width,
            height,
            cursor: 0,
            filter_state: FilterState::Unfiltered,
            show_spinner: false,
            status_message: None,
        }
    }

    /// Sets the title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the items.
    pub fn set_items(&mut self, items: Vec<I>) {
        let len = items.len();
        self.items = items;
        self.filtered_indices = (0..len).collect();
        self.paginator.set_total_pages_from_items(len);
        self.cursor = 0;
    }

    /// Returns the items.
    #[must_use]
    pub fn items(&self) -> &[I] {
        &self.items
    }

    /// Returns visible items based on current filter.
    #[must_use]
    pub fn visible_items(&self) -> Vec<&I> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.items.get(i))
            .collect()
    }

    /// Returns the current cursor index in the filtered list.
    #[must_use]
    pub fn index(&self) -> usize {
        self.cursor
    }

    /// Returns the currently selected item.
    #[must_use]
    pub fn selected_item(&self) -> Option<&I> {
        self.filtered_indices
            .get(self.cursor)
            .and_then(|&i| self.items.get(i))
    }

    /// Selects an item by index.
    pub fn select(&mut self, index: usize) {
        self.cursor = index.min(self.filtered_indices.len().saturating_sub(1));
    }

    /// Moves the cursor up.
    pub fn cursor_up(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        if self.cursor == 0 {
            if self.infinite_scrolling {
                self.cursor = self.filtered_indices.len() - 1;
            }
        } else {
            self.cursor -= 1;
        }
    }

    /// Moves the cursor down.
    pub fn cursor_down(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        if self.cursor >= self.filtered_indices.len() - 1 {
            if self.infinite_scrolling {
                self.cursor = 0;
            }
        } else {
            self.cursor += 1;
        }
    }

    /// Returns the filter state.
    #[must_use]
    pub fn filter_state(&self) -> FilterState {
        self.filter_state
    }

    /// Returns the current filter value.
    #[must_use]
    pub fn filter_value(&self) -> String {
        self.filter_input.value()
    }

    /// Sets the filter value.
    pub fn set_filter_value(&mut self, value: &str) {
        self.filter_input.set_value(value);
        self.apply_filter();
    }

    /// Resets the filter.
    pub fn reset_filter(&mut self) {
        self.filter_input.reset();
        self.filter_state = FilterState::Unfiltered;
        self.filtered_indices = (0..self.items.len()).collect();
        self.paginator.set_total_pages_from_items(self.items.len());
        self.cursor = 0;
    }

    /// Applies the current filter.
    fn apply_filter(&mut self) {
        let term = self.filter_input.value();
        if term.is_empty() {
            self.reset_filter();
            return;
        }

        let targets: Vec<String> = self
            .items
            .iter()
            .map(|i| i.filter_value().to_string())
            .collect();
        let ranks = default_filter(&term, &targets);

        self.filtered_indices = ranks.iter().map(|r| r.index).collect();
        self.paginator
            .set_total_pages_from_items(self.filtered_indices.len());
        self.cursor = 0;
        self.filter_state = FilterState::FilterApplied;
    }

    /// Starts the spinner.
    /// Returns a message that should be passed to update to start the animation.
    pub fn start_spinner(&mut self) -> Option<Message> {
        self.show_spinner = true;
        Some(self.spinner.tick())
    }

    /// Stops the spinner.
    pub fn stop_spinner(&mut self) {
        self.show_spinner = false;
    }

    /// Returns whether the spinner is visible.
    #[must_use]
    pub fn spinner_visible(&self) -> bool {
        self.show_spinner
    }

    /// Sets a new status message.
    pub fn new_status_message(&mut self, msg: impl Into<String>) -> Option<Cmd> {
        self.status_message = Some(msg.into());
        let lifetime = self.status_message_lifetime;
        Some(Cmd::new(move || {
            std::thread::sleep(lifetime);
            Message::new(StatusMessageTimeoutMsg)
        }))
    }

    /// Returns the current status message.
    #[must_use]
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    /// Sets the width.
    pub fn set_width(&mut self, w: usize) {
        self.width = w;
        self.help.width = w;
    }

    /// Sets the height.
    pub fn set_height(&mut self, h: usize) {
        self.height = h;
        self.update_pagination();
    }

    /// Returns the width.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the height.
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    /// Updates pagination based on height and delegate.
    fn update_pagination(&mut self) {
        let item_height = self.delegate.height() + self.delegate.spacing();
        let available = self.height.saturating_sub(4); // Reserve space for chrome
        let per_page = available / item_height.max(1);
        // Rebuild paginator with new per_page
        self.paginator = Paginator::new().per_page(per_page);
        self.paginator
            .set_total_pages_from_items(self.filtered_indices.len());
    }

    /// Updates the list based on messages.
    pub fn update(&mut self, msg: Message) -> Option<Cmd> {
        // Handle status message timeout
        if msg.is::<StatusMessageTimeoutMsg>() {
            self.status_message = None;
            return None;
        }

        // Handle spinner updates - check for tick message first
        if self.show_spinner && msg.is::<TickMsg>() {
            return self.spinner.update(msg);
        }

        // Handle key messages
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();

            // Handle filtering state
            if self.filter_state == FilterState::Filtering {
                if matches(&key_str, &[&self.key_map.cancel_while_filtering]) {
                    self.filter_input.reset();
                    self.filter_state = FilterState::Unfiltered;
                    self.filtered_indices = (0..self.items.len()).collect();
                    return None;
                }
                if matches(&key_str, &[&self.key_map.accept_while_filtering]) {
                    self.apply_filter();
                    self.filter_state = FilterState::FilterApplied;
                    self.filter_input.blur();
                    return None;
                }

                // Pass to filter input
                return self.filter_input.update(msg);
            }

            // Normal navigation
            if matches(&key_str, &[&self.key_map.cursor_up]) {
                self.cursor_up();
            } else if matches(&key_str, &[&self.key_map.cursor_down]) {
                self.cursor_down();
            } else if matches(&key_str, &[&self.key_map.next_page]) {
                self.paginator.next_page();
                // Move cursor to first item of new page
                let start = self.paginator.page() * self.paginator.get_per_page();
                self.cursor = start.min(self.filtered_indices.len().saturating_sub(1));
            } else if matches(&key_str, &[&self.key_map.prev_page]) {
                self.paginator.prev_page();
                let start = self.paginator.page() * self.paginator.get_per_page();
                self.cursor = start.min(self.filtered_indices.len().saturating_sub(1));
            } else if matches(&key_str, &[&self.key_map.goto_start]) {
                self.cursor = 0;
                self.paginator.set_page(0);
            } else if matches(&key_str, &[&self.key_map.goto_end]) {
                self.cursor = self.filtered_indices.len().saturating_sub(1);
                self.paginator
                    .set_page(self.paginator.get_total_pages().saturating_sub(1));
            } else if matches(&key_str, &[&self.key_map.filter]) && self.filtering_enabled {
                self.filter_state = FilterState::Filtering;
                self.filter_input.focus();
            } else if matches(&key_str, &[&self.key_map.clear_filter]) {
                self.reset_filter();
            } else if matches(&key_str, &[&self.key_map.show_full_help]) {
                self.help.show_all = true;
            } else if matches(&key_str, &[&self.key_map.close_full_help]) {
                self.help.show_all = false;
            }
        }

        None
    }

    /// Renders the list.
    #[must_use]
    pub fn view(&self) -> String {
        let mut sections = Vec::new();

        // Title
        if self.show_title && !self.title.is_empty() {
            sections.push(self.styles.title.render(&self.title));
        }

        // Filter input
        if self.show_filter && self.filter_state == FilterState::Filtering {
            sections.push(self.filter_input.view());
        }

        // Items
        if self.filtered_indices.is_empty() {
            sections.push(self.styles.no_items.render("No items."));
        } else {
            let per_page = self.paginator.get_per_page();
            let start = self.paginator.page() * per_page;
            let end = (start + per_page).min(self.filtered_indices.len());

            for (view_idx, &item_idx) in self.filtered_indices[start..end].iter().enumerate() {
                let global_idx = start + view_idx;
                let selected = global_idx == self.cursor;

                if let Some(item) = self.items.get(item_idx) {
                    let rendered = self.delegate.render(item, global_idx, selected, self.width);
                    sections.push(rendered);
                }
            }
        }

        // Spinner
        if self.show_spinner {
            sections.push(self.spinner.view());
        }

        // Status bar
        if self.show_status_bar {
            let status = self.status_message.as_deref().unwrap_or_else(|| {
                let count = self.filtered_indices.len();
                if count == 1 {
                    "1 item"
                } else {
                    "" // Will be replaced below
                }
            });

            if status.is_empty() {
                let count = self.filtered_indices.len();
                sections.push(
                    self.styles
                        .status_bar
                        .render(&format!("{} {}", count, self.item_name_plural)),
                );
            } else {
                sections.push(self.styles.status_bar.render(status));
            }
        }

        // Pagination
        if self.show_pagination && self.paginator.get_total_pages() > 1 {
            sections.push(self.paginator.view());
        }

        // Help
        if self.show_help {
            let bindings: Vec<&Binding> = vec![
                &self.key_map.cursor_up,
                &self.key_map.cursor_down,
                &self.key_map.filter,
                &self.key_map.quit,
            ];
            sections.push(
                self.styles
                    .help
                    .render(&self.help.short_help_view(&bindings)),
            );
        }

        sections.join("\n")
    }

    /// Initializes the list (called when used as a standalone Model).
    ///
    /// Returns `None` by default since lists are typically initialized with items.
    /// Override or use `start_spinner()` if loading items asynchronously.
    #[must_use]
    pub fn init(&self) -> Option<Cmd> {
        None
    }
}

/// Implement the Model trait for standalone bubbletea usage.
impl<I: Item, D: ItemDelegate<I>> Model for List<I, D> {
    fn init(&self) -> Option<Cmd> {
        List::init(self)
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        List::update(self, msg)
    }

    fn view(&self) -> String {
        List::view(self)
    }
}

// Implement Debug manually since FilterFn doesn't implement Debug
impl<I: Item + std::fmt::Debug, D: ItemDelegate<I> + std::fmt::Debug> std::fmt::Debug
    for List<I, D>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("List")
            .field("title", &self.title)
            .field("items_count", &self.items.len())
            .field("cursor", &self.cursor)
            .field("filter_state", &self.filter_state)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestItem {
        name: String,
    }

    impl Item for TestItem {
        fn filter_value(&self) -> &str {
            &self.name
        }
    }

    fn test_items() -> Vec<TestItem> {
        vec![
            TestItem {
                name: "Apple".into(),
            },
            TestItem {
                name: "Banana".into(),
            },
            TestItem {
                name: "Cherry".into(),
            },
            TestItem {
                name: "Date".into(),
            },
        ]
    }

    #[test]
    fn test_list_new() {
        let list = List::new(test_items(), DefaultDelegate::new(), 80, 24);
        assert_eq!(list.items().len(), 4);
        assert_eq!(list.index(), 0);
    }

    #[test]
    fn test_list_navigation() {
        let mut list = List::new(test_items(), DefaultDelegate::new(), 80, 24);

        assert_eq!(list.index(), 0);

        list.cursor_down();
        assert_eq!(list.index(), 1);

        list.cursor_down();
        assert_eq!(list.index(), 2);

        list.cursor_up();
        assert_eq!(list.index(), 1);
    }

    #[test]
    fn test_list_selected_item() {
        let mut list = List::new(test_items(), DefaultDelegate::new(), 80, 24);

        assert_eq!(list.selected_item().map(|i| i.name.as_str()), Some("Apple"));

        list.cursor_down();
        assert_eq!(
            list.selected_item().map(|i| i.name.as_str()),
            Some("Banana")
        );
    }

    #[test]
    fn test_list_filter() {
        let mut list = List::new(test_items(), DefaultDelegate::new(), 80, 24);

        list.set_filter_value("an");

        // Should match "Banana"
        assert_eq!(list.visible_items().len(), 1);
        assert_eq!(list.visible_items()[0].name, "Banana");
    }

    #[test]
    fn test_list_reset_filter() {
        let mut list = List::new(test_items(), DefaultDelegate::new(), 80, 24);

        list.set_filter_value("an");
        assert_eq!(list.visible_items().len(), 1);

        list.reset_filter();
        assert_eq!(list.visible_items().len(), 4);
    }

    #[test]
    fn test_list_filter_state() {
        let list = List::new(test_items(), DefaultDelegate::new(), 80, 24);
        assert_eq!(list.filter_state(), FilterState::Unfiltered);
    }

    #[test]
    fn test_list_infinite_scroll() {
        let mut list = List::new(test_items(), DefaultDelegate::new(), 80, 24);
        list.infinite_scrolling = true;

        // At start, going up should wrap to end
        list.cursor_up();
        assert_eq!(list.index(), 3);

        // Going down should wrap to start
        list.cursor_down();
        assert_eq!(list.index(), 0);
    }

    #[test]
    fn test_list_status_message() {
        let mut list = List::new(test_items(), DefaultDelegate::new(), 80, 24);

        assert!(list.status_message().is_none());

        list.new_status_message("Test message");
        assert_eq!(list.status_message(), Some("Test message"));
    }

    #[test]
    fn test_list_spinner() {
        let mut list = List::new(test_items(), DefaultDelegate::new(), 80, 24);

        assert!(!list.spinner_visible());

        list.start_spinner();
        assert!(list.spinner_visible());

        list.stop_spinner();
        assert!(!list.spinner_visible());
    }

    #[test]
    fn test_list_view() {
        let list = List::new(test_items(), DefaultDelegate::new(), 80, 24).title("Fruits");

        let view = list.view();
        assert!(view.contains("Fruits"));
        assert!(view.contains("Apple"));
    }

    #[test]
    fn test_default_filter() {
        let targets = vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
        ];

        let ranks = default_filter("an", &targets);
        assert_eq!(ranks.len(), 1);
        assert_eq!(ranks[0].index, 1); // Banana
    }

    #[test]
    fn test_default_delegate() {
        let delegate = DefaultDelegate::new().with_height(2).with_spacing(1);
        assert_eq!(delegate.item_height, 2);
        assert_eq!(delegate.item_spacing, 1);
    }

    #[test]
    fn test_keymap_default() {
        let km = KeyMap::default();
        assert!(!km.cursor_up.get_keys().is_empty());
        assert!(!km.filter.get_keys().is_empty());
    }

    #[test]
    fn test_filter_state_display() {
        assert_eq!(FilterState::Unfiltered.to_string(), "unfiltered");
        assert_eq!(FilterState::Filtering.to_string(), "filtering");
        assert_eq!(FilterState::FilterApplied.to_string(), "filter applied");
    }

    // Model trait implementation tests

    #[test]
    fn test_model_trait_init_returns_none() {
        let list = List::new(test_items(), DefaultDelegate::new(), 80, 24);
        // Use the Model trait method explicitly
        let cmd = Model::init(&list);
        assert!(cmd.is_none(), "Model::init should return None for List");
    }

    #[test]
    fn test_model_trait_view_returns_content() {
        let list = List::new(test_items(), DefaultDelegate::new(), 80, 24).title("Test List");
        // Use the Model trait method explicitly
        let view = Model::view(&list);
        assert!(view.contains("Test List"), "View should contain the title");
        assert!(view.contains("Apple"), "View should contain first item");
    }

    #[test]
    fn test_model_trait_update_handles_messages() {
        let mut list = List::new(test_items(), DefaultDelegate::new(), 80, 24);
        assert_eq!(list.index(), 0);

        // Create a down key message to navigate
        let key_msg = Message::new(KeyMsg {
            key_type: bubbletea::KeyType::Runes,
            runes: vec!['j'], // 'j' is mapped to cursor_down
            alt: false,
            paste: false,
        });

        // Use the Model trait method explicitly
        let _ = Model::update(&mut list, key_msg);
        assert_eq!(list.index(), 1, "Cursor should have moved down");
    }

    #[test]
    fn test_list_satisfies_model_bounds() {
        // This test verifies List can be used where Model + Send + 'static is required
        fn accepts_model<M: Model + Send + 'static>(_model: M) {}
        let list = List::new(test_items(), DefaultDelegate::new(), 80, 24);
        accepts_model(list);
    }
}
