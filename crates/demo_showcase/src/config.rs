//! Runtime configuration for `demo_showcase`.
//!
//! This module provides the canonical representation of all runtime options.
//! The [`Config`] struct is the single source of truth for toggles and settings,
//! independent of how they were specified (CLI, environment, file).
//!
//! # Examples
//!
//! ```rust,ignore
//! // Create default config
//! let config = Config::default();
//!
//! // Create config with specific settings
//! let config = Config {
//!     seed: Some(42),
//!     color_mode: ColorMode::Auto,
//!     animations: AnimationMode::Enabled,
//!     ..Default::default()
//! };
//! ```

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::theme::ThemePreset;

/// Runtime configuration for the demo showcase.
///
/// This struct represents all configurable options, resolved from CLI args,
/// environment variables, and/or config files. It's designed to be:
///
/// - **Serializable**: Can be saved/loaded from JSON
/// - **Testable**: Tests can construct directly without CLI parsing
/// - **Complete**: All runtime toggles in one place
#[derive(Debug, Clone, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "Config naturally has boolean flags"
)]
pub struct Config {
    // ========================================================================
    // Display Settings
    // ========================================================================
    /// Theme preset to use.
    pub theme_preset: ThemePreset,

    /// Optional path to a custom theme JSON file.
    pub theme_file: Option<PathBuf>,

    /// Color output mode.
    pub color_mode: ColorMode,

    /// Animation mode.
    pub animations: AnimationMode,

    // ========================================================================
    // Input Settings
    // ========================================================================
    /// Whether mouse input is enabled.
    pub mouse: bool,

    // ========================================================================
    // Terminal Settings
    // ========================================================================
    /// Whether to use alternate screen mode.
    pub alt_screen: bool,

    // ========================================================================
    // Data Settings
    // ========================================================================
    /// Seed for deterministic data generation.
    ///
    /// If None, a random seed is generated at startup.
    pub seed: Option<u64>,

    /// Root directory for the file browser.
    pub files_root: Option<PathBuf>,

    // ========================================================================
    // Mode Settings
    // ========================================================================
    /// Whether running in headless self-check mode.
    pub self_check: bool,

    /// Log verbosity level (0=warn, 1=info, 2=debug, 3+=trace).
    pub verbosity: u8,

    // ========================================================================
    // Feature Toggles
    // ========================================================================
    /// Whether syntax highlighting is enabled (when available).
    pub syntax_highlighting: bool,

    /// Whether to show line numbers in code blocks.
    pub line_numbers: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme_preset: ThemePreset::default(),
            theme_file: None,
            color_mode: ColorMode::Auto,
            animations: AnimationMode::Enabled,
            mouse: false, // Disabled by default for safety
            alt_screen: true,
            seed: None,
            files_root: None,
            self_check: false,
            verbosity: 0,
            syntax_highlighting: true,
            line_numbers: false, // Off by default for cleaner look
        }
    }
}

impl Config {
    /// Create a new config with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create config from CLI arguments.
    ///
    /// This is the primary way to construct a Config in production.
    /// It handles all precedence rules for environment variables.
    #[must_use]
    pub fn from_cli(cli: &Cli) -> Self {
        // Determine theme preset
        let theme_preset = match cli.theme.as_str() {
            "light" => ThemePreset::Light,
            "dracula" => ThemePreset::Dracula,
            _ => ThemePreset::Dark,
        };

        // Determine color mode
        let color_mode = if cli.force_color {
            ColorMode::Always
        } else if cli.no_color {
            ColorMode::Never
        } else {
            ColorMode::Auto
        };

        // Determine animation mode
        let animations = if cli.no_animations {
            AnimationMode::Disabled
        } else if std::env::var("REDUCE_MOTION").is_ok() {
            AnimationMode::Reduced
        } else {
            AnimationMode::Enabled
        };

        Self {
            theme_preset,
            theme_file: cli.theme_file.clone(),
            color_mode,
            animations,
            mouse: !cli.no_mouse,
            alt_screen: !cli.no_alt_screen,
            seed: cli.seed,
            files_root: cli.files_root.clone(),
            self_check: cli.self_check,
            verbosity: cli.verbose,
            syntax_highlighting: true, // Depends on compile-time feature
            line_numbers: false,       // Off by default
        }
    }

    /// Get the effective seed value.
    ///
    /// If no seed was specified, generates one from the current time.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Seed truncation is acceptable"
    )]
    pub fn effective_seed(&self) -> u64 {
        self.seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(42, |d| d.as_nanos() as u64)
        })
    }

    /// Get the effective files root directory.
    ///
    /// Defaults to current working directory if not specified.
    #[must_use]
    pub fn effective_files_root(&self) -> PathBuf {
        self.files_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Check if colors should be used.
    ///
    /// Takes into account the color mode and terminal capabilities.
    #[must_use]
    pub fn use_color(&self) -> bool {
        match self.color_mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => {
                // Check environment
                if std::env::var("NO_COLOR").is_ok() {
                    return false;
                }

                // Check if stdout is a tty (simplified check)
                // In production, would use atty or similar
                true
            }
        }
    }

    /// Check if animations should be used.
    #[must_use]
    pub const fn use_animations(&self) -> bool {
        !matches!(self.animations, AnimationMode::Disabled)
    }

    /// Check if reduced motion is preferred.
    #[must_use]
    pub const fn reduce_motion(&self) -> bool {
        matches!(self.animations, AnimationMode::Reduced)
    }

    /// Check if running in headless mode.
    #[must_use]
    pub const fn is_headless(&self) -> bool {
        self.self_check
    }

    /// Validate the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate theme file exists if specified
        if let Some(ref path) = self.theme_file
            && !path.exists()
        {
            return Err(ConfigError::ThemeFileNotFound(path.clone()));
        }

        // Validate files root exists if specified
        if let Some(ref path) = self.files_root
            && !path.exists()
        {
            return Err(ConfigError::FilesRootNotFound(path.clone()));
        }

        if let Some(ref path) = self.files_root
            && !path.is_dir()
        {
            return Err(ConfigError::FilesRootNotDirectory(path.clone()));
        }

        Ok(())
    }

    /// Export configuration as a diagnostic string.
    #[must_use]
    pub fn to_diagnostic_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Theme: {:?}", self.theme_preset));
        if let Some(ref file) = self.theme_file {
            lines.push(format!("Theme file: {}", file.display()));
        }
        lines.push(format!("Color mode: {:?}", self.color_mode));
        lines.push(format!("Animations: {:?}", self.animations));
        lines.push(format!("Mouse: {}", if self.mouse { "on" } else { "off" }));
        lines.push(format!(
            "Alt screen: {}",
            if self.alt_screen { "on" } else { "off" }
        ));
        lines.push(format!("Seed: {:?}", self.seed));
        if let Some(ref path) = self.files_root {
            lines.push(format!("Files root: {}", path.display()));
        }
        lines.push(format!("Self-check: {}", self.self_check));
        lines.push(format!("Verbosity: {}", self.verbosity));
        lines.push(format!("Syntax highlighting: {}", self.syntax_highlighting));
        lines.push(format!("Line numbers: {}", self.line_numbers));

        lines.join("\n")
    }
}

/// Color output mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorMode {
    /// Automatically detect based on terminal and environment.
    #[default]
    Auto,
    /// Always use colors.
    Always,
    /// Never use colors (ASCII mode).
    Never,
}

/// Animation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AnimationMode {
    /// Enable full animations.
    #[default]
    Enabled,
    /// Reduce motion for accessibility.
    Reduced,
    /// Disable all animations.
    Disabled,
}

/// Configuration error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    /// Theme file not found.
    #[error("Theme file not found: {0}")]
    ThemeFileNotFound(PathBuf),

    /// Files root not found.
    #[error("Files root directory not found: {0}")]
    FilesRootNotFound(PathBuf),

    /// Files root is not a directory.
    #[error("Files root is not a directory: {0}")]
    FilesRootNotDirectory(PathBuf),

    /// Invalid theme name.
    #[error("Invalid theme name: {0}")]
    InvalidTheme(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default() {
        let config = Config::default();

        assert_eq!(config.theme_preset, ThemePreset::Dark);
        assert!(config.theme_file.is_none());
        assert_eq!(config.color_mode, ColorMode::Auto);
        assert_eq!(config.animations, AnimationMode::Enabled);
        assert!(!config.mouse);
        assert!(config.alt_screen);
        assert!(config.seed.is_none());
        assert!(!config.self_check);
    }

    #[test]
    fn config_from_cli_defaults() {
        let cli = Cli::try_parse_from(["demo_showcase"]).unwrap();
        let config = Config::from_cli(&cli);

        assert_eq!(config.theme_preset, ThemePreset::Dark);
        assert_eq!(config.color_mode, ColorMode::Auto);
        assert!(!config.self_check);
    }

    #[test]
    fn config_from_cli_theme() {
        let cli = Cli::try_parse_from(["demo_showcase", "--theme", "light"]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.theme_preset, ThemePreset::Light);

        let cli = Cli::try_parse_from(["demo_showcase", "--theme", "dracula"]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.theme_preset, ThemePreset::Dracula);
    }

    #[test]
    fn config_from_cli_color_modes() {
        let cli = Cli::try_parse_from(["demo_showcase", "--no-color"]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.color_mode, ColorMode::Never);

        let cli = Cli::try_parse_from(["demo_showcase", "--force-color"]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.color_mode, ColorMode::Always);
    }

    #[test]
    fn config_from_cli_flags() {
        let cli = Cli::try_parse_from([
            "demo_showcase",
            "--no-animations",
            "--no-mouse",
            "--no-alt-screen",
            "--self-check",
        ])
        .unwrap();
        let config = Config::from_cli(&cli);

        assert_eq!(config.animations, AnimationMode::Disabled);
        assert!(!config.mouse);
        assert!(!config.alt_screen);
        assert!(config.self_check);
    }

    #[test]
    fn config_from_cli_seed() {
        let cli = Cli::try_parse_from(["demo_showcase", "--seed", "42"]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.seed, Some(42));
        assert_eq!(config.effective_seed(), 42);
    }

    #[test]
    fn config_effective_seed_generates() {
        let config = Config::default();
        let seed = config.effective_seed();
        assert!(seed > 0);
    }

    #[test]
    fn config_effective_files_root() {
        let config = Config::default();
        let root = config.effective_files_root();
        assert!(root.exists() || root.as_os_str() == ".");

        let config = Config {
            files_root: Some(PathBuf::from("/tmp")),
            ..Default::default()
        };
        assert_eq!(config.effective_files_root(), PathBuf::from("/tmp"));
    }

    #[test]
    fn config_use_color() {
        let config = Config {
            color_mode: ColorMode::Always,
            ..Default::default()
        };
        assert!(config.use_color());

        let config = Config {
            color_mode: ColorMode::Never,
            ..Default::default()
        };
        assert!(!config.use_color());
    }

    #[test]
    fn config_use_animations() {
        let config = Config {
            animations: AnimationMode::Enabled,
            ..Default::default()
        };
        assert!(config.use_animations());

        let config = Config {
            animations: AnimationMode::Reduced,
            ..Default::default()
        };
        assert!(config.use_animations());
        assert!(config.reduce_motion());

        let config = Config {
            animations: AnimationMode::Disabled,
            ..Default::default()
        };
        assert!(!config.use_animations());
    }

    #[test]
    fn config_validate_success() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_validate_theme_file_not_found() {
        let config = Config {
            theme_file: Some(PathBuf::from("/nonexistent/theme.json")),
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigError::ThemeFileNotFound(_))
        ));
    }

    #[test]
    fn config_validate_files_root_not_found() {
        let config = Config {
            files_root: Some(PathBuf::from("/nonexistent/dir")),
            ..Default::default()
        };
        assert!(matches!(
            config.validate(),
            Err(ConfigError::FilesRootNotFound(_))
        ));
    }

    #[test]
    fn config_serialization() {
        let config = Config {
            seed: Some(42),
            theme_preset: ThemePreset::Light,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.seed, Some(42));
        assert_eq!(parsed.theme_preset, ThemePreset::Light);
    }

    #[test]
    fn config_diagnostic_string() {
        let config = Config {
            seed: Some(42),
            ..Default::default()
        };

        let diag = config.to_diagnostic_string();
        assert!(diag.contains("Seed: Some(42)"));
        assert!(diag.contains("Theme:"));
    }

    #[test]
    fn color_mode_default() {
        assert_eq!(ColorMode::default(), ColorMode::Auto);
    }

    #[test]
    fn animation_mode_default() {
        assert_eq!(AnimationMode::default(), AnimationMode::Enabled);
    }
}
