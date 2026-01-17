//! Keybinding definitions and matching utilities.
//!
//! This module provides types for defining keybindings and matching them against
//! key events. It's useful for creating user-configurable keymaps in TUI applications.
//!
//! # Example
//!
//! ```rust
//! use bubbles::key::{Binding, matches};
//!
//! let up = Binding::new()
//!     .keys(&["k", "up"])
//!     .help("↑/k", "move up");
//!
//! let down = Binding::new()
//!     .keys(&["j", "down"])
//!     .help("↓/j", "move down");
//!
//! // Check if a key matches
//! assert!(matches("k", &[&up, &down]));
//! assert!(matches("down", &[&up, &down]));
//! assert!(!matches("x", &[&up, &down]));
//! ```

use std::fmt;

/// Help information for a keybinding.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Help {
    /// The key(s) to display in help text (e.g., "↑/k").
    pub key: String,
    /// Description of what the binding does.
    pub desc: String,
}

impl Help {
    /// Creates new help information.
    #[must_use]
    pub fn new(key: impl Into<String>, desc: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            desc: desc.into(),
        }
    }
}

/// A keybinding with associated help text.
///
/// Bindings can be enabled/disabled and contain zero or more key sequences
/// that trigger the binding.
#[derive(Debug, Clone, Default)]
pub struct Binding {
    keys: Vec<String>,
    help: Help,
    disabled: bool,
}

impl Binding {
    /// Creates a new empty binding.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the keys for this binding.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bubbles::key::Binding;
    ///
    /// let binding = Binding::new().keys(&["k", "up", "ctrl+p"]);
    /// assert_eq!(binding.get_keys(), &["k", "up", "ctrl+p"]);
    /// ```
    #[must_use]
    pub fn keys(mut self, keys: &[&str]) -> Self {
        self.keys = keys.iter().map(|&s| s.to_string()).collect();
        self
    }

    /// Sets the help text for this binding.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bubbles::key::Binding;
    ///
    /// let binding = Binding::new()
    ///     .keys(&["q"])
    ///     .help("q", "quit");
    /// assert_eq!(binding.get_help().key, "q");
    /// assert_eq!(binding.get_help().desc, "quit");
    /// ```
    #[must_use]
    pub fn help(mut self, key: impl Into<String>, desc: impl Into<String>) -> Self {
        self.help = Help::new(key, desc);
        self
    }

    /// Creates a disabled binding.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Sets the keys for this binding (mutable version).
    pub fn set_keys(&mut self, keys: &[&str]) {
        self.keys = keys.iter().map(|&s| s.to_string()).collect();
    }

    /// Returns the keys for this binding.
    #[must_use]
    pub fn get_keys(&self) -> &[String] {
        &self.keys
    }

    /// Sets the help text for this binding (mutable version).
    pub fn set_help(&mut self, key: impl Into<String>, desc: impl Into<String>) {
        self.help = Help::new(key, desc);
    }

    /// Returns the help information for this binding.
    #[must_use]
    pub fn get_help(&self) -> &Help {
        &self.help
    }

    /// Returns whether this binding is enabled.
    ///
    /// A binding is enabled if it's not explicitly disabled and has at least one key.
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.disabled && !self.keys.is_empty()
    }

    /// Enables or disables the binding (mutable version).
    pub fn enable(&mut self, enabled: bool) {
        self.disabled = !enabled;
    }

    /// Enables or disables the binding (builder version).
    #[must_use]
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.disabled = !enabled;
        self
    }

    /// Removes the keys and help from this binding, effectively nullifying it.
    ///
    /// This is a step beyond disabling - it removes the binding entirely.
    /// Use this when you want to completely remove a keybinding from a keymap.
    pub fn unbind(&mut self) {
        self.keys.clear();
        self.help = Help::default();
    }
}

/// Checks if the given key matches any of the given bindings.
///
/// The key is compared against all keys in each binding. Only enabled bindings
/// are considered.
///
/// # Example
///
/// ```rust
/// use bubbles::key::{Binding, matches};
///
/// let quit = Binding::new().keys(&["q", "ctrl+c"]);
/// let disabled = Binding::new().keys(&["x"]).disabled();
///
/// assert!(matches("q", &[&quit]));
/// assert!(matches("ctrl+c", &[&quit]));
/// assert!(!matches("x", &[&disabled])); // Disabled bindings don't match
/// ```
pub fn matches<K: fmt::Display>(key: K, bindings: &[&Binding]) -> bool {
    let key_str = key.to_string();
    for binding in bindings {
        if binding.enabled() {
            for k in &binding.keys {
                if *k == key_str {
                    return true;
                }
            }
        }
    }
    false
}

/// Checks if the given key matches a single binding.
///
/// Convenience function for matching against a single binding.
pub fn matches_one<K: fmt::Display>(key: K, binding: &Binding) -> bool {
    matches(key, &[binding])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binding_new() {
        let binding = Binding::new();
        assert!(binding.get_keys().is_empty());
        assert!(!binding.enabled());
    }

    #[test]
    fn test_binding_with_keys() {
        let binding = Binding::new().keys(&["k", "up"]);
        assert_eq!(binding.get_keys(), &["k", "up"]);
        assert!(binding.enabled());
    }

    #[test]
    fn test_binding_with_help() {
        let binding = Binding::new()
            .keys(&["q"])
            .help("q", "quit the application");
        assert_eq!(binding.get_help().key, "q");
        assert_eq!(binding.get_help().desc, "quit the application");
    }

    #[test]
    fn test_binding_disabled() {
        let binding = Binding::new().keys(&["q"]).disabled();
        assert!(!binding.enabled());
    }

    #[test]
    fn test_binding_set_enabled() {
        let mut binding = Binding::new().keys(&["q"]).disabled();
        assert!(!binding.enabled());
        binding.enable(true);
        assert!(binding.enabled());
    }

    #[test]
    fn test_binding_set_enabled_builder() {
        let binding = Binding::new().keys(&["q"]).set_enabled(false);
        assert!(!binding.enabled());
        let binding = binding.set_enabled(true);
        assert!(binding.enabled());
    }

    #[test]
    fn test_binding_unbind() {
        let mut binding = Binding::new()
            .keys(&["q"])
            .help("q", "quit");
        binding.unbind();
        assert!(binding.get_keys().is_empty());
        assert!(binding.get_help().key.is_empty());
    }

    #[test]
    fn test_matches() {
        let up = Binding::new().keys(&["k", "up"]);
        let down = Binding::new().keys(&["j", "down"]);

        assert!(matches("k", &[&up, &down]));
        assert!(matches("up", &[&up, &down]));
        assert!(matches("j", &[&up, &down]));
        assert!(matches("down", &[&up, &down]));
        assert!(!matches("x", &[&up, &down]));
    }

    #[test]
    fn test_matches_disabled() {
        let binding = Binding::new().keys(&["q"]).disabled();
        assert!(!matches("q", &[&binding]));
    }

    #[test]
    fn test_matches_empty() {
        let binding = Binding::new();
        assert!(!matches("q", &[&binding]));
    }

    #[test]
    fn test_matches_one() {
        let quit = Binding::new().keys(&["q", "ctrl+c"]);
        assert!(matches_one("q", &quit));
        assert!(matches_one("ctrl+c", &quit));
        assert!(!matches_one("x", &quit));
    }
}
