//! Pagination component for navigating through pages.
//!
//! This module provides pagination state management and display, useful for
//! navigating lists, tables, or any paginated content.
//!
//! # Example
//!
//! ```rust
//! use bubbles::paginator::{Paginator, Type};
//!
//! let mut paginator = Paginator::new()
//!     .per_page(10)
//!     .total_pages(5);
//!
//! // Navigate
//! paginator.next_page();
//! assert_eq!(paginator.page(), 1);
//!
//! // Get slice bounds for rendering
//! let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
//! let (start, end) = paginator.get_slice_bounds(items.len());
//! let visible = &items[start..end];
//! ```

use crate::key::{matches, Binding};
use bubbletea::{KeyMsg, Message};

/// Pagination display type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Type {
    /// Arabic numerals: "1/5"
    #[default]
    Arabic,
    /// Dot indicators: "●○○○○"
    Dots,
}

/// Key bindings for pagination navigation.
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Binding to go to previous page.
    pub prev_page: Binding,
    /// Binding to go to next page.
    pub next_page: Binding,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            prev_page: Binding::new()
                .keys(&["pgup", "left", "h"])
                .help("←/h", "prev page"),
            next_page: Binding::new()
                .keys(&["pgdown", "right", "l"])
                .help("→/l", "next page"),
        }
    }
}

/// Pagination model.
#[derive(Debug, Clone)]
pub struct Paginator {
    /// Display type (Arabic or Dots).
    pub display_type: Type,
    /// Current page (0-indexed).
    page: usize,
    /// Items per page.
    per_page: usize,
    /// Total number of pages.
    total_pages: usize,
    /// Character for active page in Dots mode.
    pub active_dot: String,
    /// Character for inactive pages in Dots mode.
    pub inactive_dot: String,
    /// Format string for Arabic mode.
    pub arabic_format: String,
    /// Key bindings.
    pub key_map: KeyMap,
}

impl Default for Paginator {
    fn default() -> Self {
        Self::new()
    }
}

impl Paginator {
    /// Creates a new paginator with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            display_type: Type::Arabic,
            page: 0,
            per_page: 1,
            total_pages: 1,
            active_dot: "•".to_string(),
            inactive_dot: "○".to_string(),
            arabic_format: "{}/{}".to_string(),
            key_map: KeyMap::default(),
        }
    }

    /// Sets the display type.
    #[must_use]
    pub fn display_type(mut self, t: Type) -> Self {
        self.display_type = t;
        self
    }

    /// Sets the number of items per page.
    #[must_use]
    pub fn per_page(mut self, n: usize) -> Self {
        self.per_page = n.max(1);
        self
    }

    /// Sets the total number of pages.
    #[must_use]
    pub fn total_pages(mut self, n: usize) -> Self {
        self.total_pages = n.max(1);
        self
    }

    /// Returns the current page (0-indexed).
    #[must_use]
    pub fn page(&self) -> usize {
        self.page
    }

    /// Sets the current page.
    pub fn set_page(&mut self, page: usize) {
        self.page = page.min(self.total_pages.saturating_sub(1));
    }

    /// Returns the items per page.
    #[must_use]
    pub fn get_per_page(&self) -> usize {
        self.per_page
    }

    /// Returns the total number of pages.
    #[must_use]
    pub fn get_total_pages(&self) -> usize {
        self.total_pages
    }

    /// Calculates and sets the total pages from item count.
    ///
    /// Returns the calculated total pages.
    pub fn set_total_pages_from_items(&mut self, items: usize) -> usize {
        if items < 1 {
            return self.total_pages;
        }

        let mut n = items / self.per_page;
        if items % self.per_page > 0 {
            n += 1;
        }
        self.total_pages = n;
        n
    }

    /// Returns the number of items on the current page.
    #[must_use]
    pub fn items_on_page(&self, total_items: usize) -> usize {
        if total_items < 1 {
            return 0;
        }
        let (start, end) = self.get_slice_bounds(total_items);
        end - start
    }

    /// Returns slice bounds for the current page.
    ///
    /// Use this to get the start and end indices for slicing a collection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bubbles::paginator::Paginator;
    ///
    /// let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    /// let mut paginator = Paginator::new().per_page(3);
    /// paginator.set_total_pages_from_items(items.len());
    ///
    /// let (start, end) = paginator.get_slice_bounds(items.len());
    /// assert_eq!(&items[start..end], &[1, 2, 3]);
    /// ```
    #[must_use]
    pub fn get_slice_bounds(&self, length: usize) -> (usize, usize) {
        let start = self.page * self.per_page;
        let end = (self.page * self.per_page + self.per_page).min(length);
        (start, end)
    }

    /// Navigates to the previous page.
    pub fn prev_page(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }

    /// Navigates to the next page.
    pub fn next_page(&mut self) {
        if !self.on_last_page() {
            self.page += 1;
        }
    }

    /// Returns whether we're on the last page.
    #[must_use]
    pub fn on_last_page(&self) -> bool {
        self.page == self.total_pages.saturating_sub(1)
    }

    /// Returns whether we're on the first page.
    #[must_use]
    pub fn on_first_page(&self) -> bool {
        self.page == 0
    }

    /// Updates the paginator based on key input.
    pub fn update(&mut self, msg: Message) {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            let key_str = key.to_string();
            if matches(&key_str, &[&self.key_map.next_page]) {
                self.next_page();
            } else if matches(&key_str, &[&self.key_map.prev_page]) {
                self.prev_page();
            }
        }
    }

    /// Renders the pagination display.
    #[must_use]
    pub fn view(&self) -> String {
        match self.display_type {
            Type::Dots => self.dots_view(),
            Type::Arabic => self.arabic_view(),
        }
    }

    fn dots_view(&self) -> String {
        let mut s = String::new();
        for i in 0..self.total_pages {
            if i == self.page {
                s.push_str(&self.active_dot);
            } else {
                s.push_str(&self.inactive_dot);
            }
        }
        s
    }

    fn arabic_view(&self) -> String {
        // Replace first {} with current page, second {} with total pages
        self.arabic_format
            .replacen("{}", &(self.page + 1).to_string(), 1)
            .replacen("{}", &self.total_pages.to_string(), 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginator_new() {
        let p = Paginator::new();
        assert_eq!(p.page(), 0);
        assert_eq!(p.get_per_page(), 1);
        assert_eq!(p.get_total_pages(), 1);
    }

    #[test]
    fn test_paginator_builder() {
        let p = Paginator::new().per_page(10).total_pages(5);
        assert_eq!(p.get_per_page(), 10);
        assert_eq!(p.get_total_pages(), 5);
    }

    #[test]
    fn test_paginator_navigation() {
        let mut p = Paginator::new().total_pages(5);

        assert!(p.on_first_page());
        assert!(!p.on_last_page());

        p.next_page();
        assert_eq!(p.page(), 1);

        p.next_page();
        p.next_page();
        p.next_page();
        assert_eq!(p.page(), 4);
        assert!(p.on_last_page());

        // Should not go past last page
        p.next_page();
        assert_eq!(p.page(), 4);

        p.prev_page();
        assert_eq!(p.page(), 3);

        // Go back to first
        p.set_page(0);
        assert!(p.on_first_page());

        // Should not go before first page
        p.prev_page();
        assert_eq!(p.page(), 0);
    }

    #[test]
    fn test_paginator_slice_bounds() {
        let mut p = Paginator::new().per_page(3);
        p.set_total_pages_from_items(10);

        assert_eq!(p.get_slice_bounds(10), (0, 3));

        p.next_page();
        assert_eq!(p.get_slice_bounds(10), (3, 6));

        p.next_page();
        assert_eq!(p.get_slice_bounds(10), (6, 9));

        p.next_page();
        assert_eq!(p.get_slice_bounds(10), (9, 10));
    }

    #[test]
    fn test_paginator_items_on_page() {
        let mut p = Paginator::new().per_page(3);
        p.set_total_pages_from_items(10);

        assert_eq!(p.items_on_page(10), 3);

        p.set_page(3); // Last page
        assert_eq!(p.items_on_page(10), 1); // Only 1 item on last page
    }

    #[test]
    fn test_paginator_arabic_view() {
        let p = Paginator::new().total_pages(5);
        assert_eq!(p.view(), "1/5");
    }

    #[test]
    fn test_paginator_dots_view() {
        let mut p = Paginator::new().display_type(Type::Dots).total_pages(5);
        assert_eq!(p.view(), "•○○○○");

        p.next_page();
        assert_eq!(p.view(), "○•○○○");
    }

    #[test]
    fn test_set_total_pages_from_items() {
        let mut p = Paginator::new().per_page(10);

        assert_eq!(p.set_total_pages_from_items(25), 3);
        assert_eq!(p.get_total_pages(), 3);

        assert_eq!(p.set_total_pages_from_items(20), 2);
        assert_eq!(p.get_total_pages(), 2);

        assert_eq!(p.set_total_pages_from_items(0), 2); // Doesn't change on 0
    }
}
