//! Embedded assets for `demo_showcase`.
//!
//! This module provides compile-time embedded assets using `include_str!`.
//! The strategy is **hybrid**:
//!
//! - Source files live in `crates/demo_showcase/assets/` for easy editing
//! - Files are embedded at compile time (zero runtime I/O)
//! - E2E tests can rely on deterministic content
//!
//! # Asset Categories
//!
//! ## Documentation
//!
//! Markdown files for the Docs page, rendered with glamour:
//! - `welcome.md` - Getting started guide
//! - `architecture.md` - System architecture overview
//!
//! ## Fixtures
//!
//! Sample files for the FilePicker demo:
//! - Config files (TOML, YAML)
//! - Log files
//! - Nested directories with hidden files
//!
//! # Usage
//!
//! ```rust,ignore
//! use demo_showcase::assets::docs;
//!
//! let welcome = docs::WELCOME;
//! let arch = docs::ARCHITECTURE;
//! ```
//!
//! # Design Decisions
//!
//! 1. **Compile-time embedding**: Avoids runtime file I/O and path issues
//! 2. **Structured modules**: Organized by purpose (docs, fixtures)
//! 3. **Deterministic content**: Same content regardless of runtime environment
//! 4. **No external dependencies**: Ships self-contained

// =============================================================================
// Documentation Assets
// =============================================================================

/// Markdown documentation files for the Docs page.
///
/// These are rendered with glamour and displayed in a scrollable viewport.
pub mod docs {
    /// Welcome/getting started documentation.
    pub const WELCOME: &str = include_str!("../assets/docs/welcome.md");

    /// System architecture documentation.
    pub const ARCHITECTURE: &str = include_str!("../assets/docs/architecture.md");

    /// List of all available documentation pages.
    pub const ALL: &[(&str, &str)] = &[("Welcome", WELCOME), ("Architecture", ARCHITECTURE)];

    /// Get documentation by title (case-insensitive).
    #[must_use]
    pub fn get_by_title(title: &str) -> Option<&'static str> {
        let title_lower = title.to_lowercase();
        ALL.iter()
            .find(|(t, _)| t.to_lowercase() == title_lower)
            .map(|(_, content)| *content)
    }
}

// =============================================================================
// Fixture Assets
// =============================================================================

/// Sample files for the FilePicker demo.
///
/// These represent a typical project structure with config files,
/// logs, and nested directories.
pub mod fixtures {
    /// Configuration file fixtures.
    pub mod config {
        /// Sample TOML application config.
        pub const APP_TOML: &str = include_str!("../assets/fixtures/config/app.toml");

        /// Sample YAML services config.
        pub const SERVICES_YAML: &str = include_str!("../assets/fixtures/config/services.yaml");
    }

    /// Log file fixtures.
    pub mod logs {
        /// Sample application log file.
        pub const APP_LOG: &str = include_str!("../assets/fixtures/logs/app.log");
    }

    /// Root-level fixtures.
    pub const README: &str = include_str!("../assets/fixtures/README.md");

    /// Nested directory fixtures.
    pub mod nested {
        /// Example text file in nested directory.
        pub const EXAMPLE_TXT: &str = include_str!("../assets/fixtures/nested/example.txt");

        /// Deep nested fixtures.
        pub mod deep {
            /// JSON settings file.
            pub const SETTINGS_JSON: &str =
                include_str!("../assets/fixtures/nested/deep/settings.json");
        }

        /// Hidden file fixtures.
        pub mod hidden {
            /// Hidden config file (dot-prefixed).
            pub const HIDDEN_CONFIG: &str =
                include_str!("../assets/fixtures/nested/hidden/.hidden_config");
        }
    }

    /// Virtual file system entry for the demo.
    ///
    /// Represents a file or directory in the fixture tree.
    #[derive(Debug, Clone)]
    pub struct VirtualEntry {
        /// Entry name (file or directory name).
        pub name: &'static str,
        /// Entry kind.
        pub kind: EntryKind,
    }

    /// Kind of virtual entry.
    #[derive(Debug, Clone)]
    pub enum EntryKind {
        /// A file with embedded content.
        File(&'static str),
        /// A directory with child entries.
        Directory(&'static [VirtualEntry]),
    }

    impl VirtualEntry {
        /// Create a file entry.
        #[must_use]
        pub const fn file(name: &'static str, content: &'static str) -> Self {
            Self {
                name,
                kind: EntryKind::File(content),
            }
        }

        /// Create a directory entry.
        #[must_use]
        pub const fn dir(name: &'static str, children: &'static [VirtualEntry]) -> Self {
            Self {
                name,
                kind: EntryKind::Directory(children),
            }
        }

        /// Check if this is a directory.
        #[must_use]
        pub const fn is_dir(&self) -> bool {
            matches!(self.kind, EntryKind::Directory(_))
        }

        /// Check if this is a hidden file (starts with dot).
        #[must_use]
        pub fn is_hidden(&self) -> bool {
            self.name.starts_with('.')
        }

        /// Get file content if this is a file.
        #[must_use]
        pub const fn content(&self) -> Option<&'static str> {
            match self.kind {
                EntryKind::File(c) => Some(c),
                EntryKind::Directory(_) => None,
            }
        }

        /// Get children if this is a directory.
        #[must_use]
        pub const fn children(&self) -> Option<&'static [VirtualEntry]> {
            match self.kind {
                EntryKind::File(_) => None,
                EntryKind::Directory(c) => Some(c),
            }
        }
    }

    /// The complete fixture tree for FilePicker demos.
    ///
    /// This is a static representation of the fixtures directory
    /// that can be traversed without any runtime I/O.
    pub static FIXTURE_TREE: &[VirtualEntry] = &[
        VirtualEntry::file("README.md", README),
        VirtualEntry::dir(
            "config",
            &[
                VirtualEntry::file("app.toml", config::APP_TOML),
                VirtualEntry::file("services.yaml", config::SERVICES_YAML),
            ],
        ),
        VirtualEntry::dir("logs", &[VirtualEntry::file("app.log", logs::APP_LOG)]),
        VirtualEntry::dir(
            "nested",
            &[
                VirtualEntry::file("example.txt", nested::EXAMPLE_TXT),
                VirtualEntry::dir(
                    "deep",
                    &[VirtualEntry::file(
                        "settings.json",
                        nested::deep::SETTINGS_JSON,
                    )],
                ),
                VirtualEntry::dir(
                    "hidden",
                    &[VirtualEntry::file(
                        ".hidden_config",
                        nested::hidden::HIDDEN_CONFIG,
                    )],
                ),
            ],
        ),
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docs_welcome_not_empty() {
        assert!(!docs::WELCOME.is_empty());
        assert!(docs::WELCOME.contains("Welcome"));
    }

    #[test]
    fn docs_architecture_not_empty() {
        assert!(!docs::ARCHITECTURE.is_empty());
        assert!(docs::ARCHITECTURE.contains("Architecture"));
    }

    #[test]
    fn docs_get_by_title() {
        assert!(docs::get_by_title("Welcome").is_some());
        assert!(docs::get_by_title("welcome").is_some()); // case-insensitive
        assert!(docs::get_by_title("nonexistent").is_none());
    }

    #[test]
    fn fixtures_config_not_empty() {
        assert!(!fixtures::config::APP_TOML.is_empty());
        assert!(fixtures::config::APP_TOML.contains("[server]"));
    }

    #[test]
    fn fixtures_log_not_empty() {
        assert!(!fixtures::logs::APP_LOG.is_empty());
        assert!(fixtures::logs::APP_LOG.contains("INFO"));
    }

    #[test]
    fn fixture_tree_has_entries() {
        assert!(!fixtures::FIXTURE_TREE.is_empty());

        // Find config directory
        let config = fixtures::FIXTURE_TREE
            .iter()
            .find(|e| e.name == "config")
            .expect("config directory should exist");

        assert!(config.is_dir());

        // Check children
        let children = config.children().unwrap();
        assert!(children.iter().any(|e| e.name == "app.toml"));
    }

    #[test]
    fn virtual_entry_hidden_detection() {
        let hidden = fixtures::VirtualEntry::file(".hidden", "");
        let visible = fixtures::VirtualEntry::file("visible", "");

        assert!(hidden.is_hidden());
        assert!(!visible.is_hidden());
    }

    #[test]
    fn virtual_entry_content_access() {
        let file = fixtures::VirtualEntry::file("test.txt", "content");
        let dir = fixtures::VirtualEntry::dir("dir", &[]);

        assert_eq!(file.content(), Some("content"));
        assert!(dir.content().is_none());
        assert!(file.children().is_none());
        assert!(dir.children().is_some());
    }
}
