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

    // =========================================================================
    // bd-gbps: Config parsing + env/CLI precedence tests
    // =========================================================================

    // --- Defaults are sane ---

    #[test]
    fn config_new_equals_default() {
        let new = Config::new();
        let def = Config::default();
        // Compare field-by-field (Config doesn't derive PartialEq)
        assert_eq!(new.theme_preset, def.theme_preset);
        assert_eq!(new.color_mode, def.color_mode);
        assert_eq!(new.animations, def.animations);
        assert_eq!(new.mouse, def.mouse);
        assert_eq!(new.alt_screen, def.alt_screen);
        assert_eq!(new.seed, def.seed);
        assert_eq!(new.self_check, def.self_check);
        assert_eq!(new.verbosity, def.verbosity);
        assert_eq!(new.syntax_highlighting, def.syntax_highlighting);
        assert_eq!(new.line_numbers, def.line_numbers);
    }

    #[test]
    fn default_has_no_seed() {
        let config = Config::default();
        assert!(
            config.seed.is_none(),
            "default config should have no fixed seed"
        );
    }

    #[test]
    fn default_is_not_headless() {
        let config = Config::default();
        assert!(!config.is_headless());
    }

    #[test]
    fn default_has_animations_enabled() {
        let config = Config::default();
        assert!(config.use_animations());
        assert!(!config.reduce_motion());
    }

    #[test]
    fn default_mouse_disabled_for_safety() {
        let config = Config::default();
        assert!(!config.mouse, "mouse should be off by default");
    }

    // --- CLI flag → Config field mapping ---

    #[test]
    fn cli_each_flag_maps_correctly() {
        // Combine every CLI flag and verify each maps to the right Config field.
        let cli = Cli::try_parse_from([
            "demo_showcase",
            "--theme",
            "light",
            "--seed",
            "99",
            "--no-animations",
            "--no-mouse",
            "--no-color",
            "--no-alt-screen",
            "--self-check",
            "-vvv",
        ])
        .unwrap();
        let config = Config::from_cli(&cli);

        assert_eq!(config.theme_preset, ThemePreset::Light);
        assert_eq!(config.seed, Some(99));
        assert_eq!(config.animations, AnimationMode::Disabled);
        assert!(!config.mouse, "--no-mouse → mouse=false");
        assert_eq!(config.color_mode, ColorMode::Never, "--no-color → Never");
        assert!(!config.alt_screen, "--no-alt-screen → false");
        assert!(config.self_check, "--self-check → true");
        assert_eq!(config.verbosity, 3, "-vvv → 3");
    }

    #[test]
    fn cli_force_color_overrides_no_color() {
        // --force-color and --no-color are mutually exclusive at the clap level.
        let result = Cli::try_parse_from(["demo_showcase", "--force-color", "--no-color"]);
        assert!(result.is_err(), "force-color and no-color must conflict");
    }

    #[test]
    fn cli_force_color_sets_always() {
        let cli = Cli::try_parse_from(["demo_showcase", "--force-color"]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.color_mode, ColorMode::Always);
    }

    #[test]
    fn cli_theme_file_stored_alongside_preset() {
        // --theme-file is stored in config.theme_file; the preset is also set.
        // The rendering layer resolves precedence (theme_file > theme_preset).
        let cli = Cli::try_parse_from([
            "demo_showcase",
            "--theme",
            "dracula",
            "--theme-file",
            "/tmp/custom.json",
        ])
        .unwrap();
        let config = Config::from_cli(&cli);

        assert_eq!(config.theme_preset, ThemePreset::Dracula);
        assert_eq!(
            config.theme_file,
            Some(PathBuf::from("/tmp/custom.json")),
            "theme_file must be populated"
        );
    }

    #[test]
    fn cli_unknown_theme_falls_back_to_dark() {
        // An unrecognized theme name should map to the Dark default.
        let cli = Cli::try_parse_from(["demo_showcase", "--theme", "nonexistent"]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(
            config.theme_preset,
            ThemePreset::Dark,
            "unknown theme should fall back to Dark"
        );
    }

    #[test]
    fn cli_empty_theme_falls_back_to_dark() {
        let cli = Cli::try_parse_from(["demo_showcase", "--theme", ""]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.theme_preset, ThemePreset::Dark);
    }

    // --- Precedence rules ---

    #[test]
    fn color_mode_auto_respects_use_color() {
        // ColorMode::Auto uses color by default (non-TTY check is simplified).
        let config = Config {
            color_mode: ColorMode::Auto,
            ..Default::default()
        };
        // In test environment, NO_COLOR may or may not be set;
        // just verify the method doesn't panic and returns a bool.
        let _ = config.use_color();
    }

    #[test]
    fn color_mode_always_overrides_everything() {
        let config = Config {
            color_mode: ColorMode::Always,
            ..Default::default()
        };
        assert!(config.use_color(), "Always means always");
    }

    #[test]
    fn color_mode_never_overrides_everything() {
        let config = Config {
            color_mode: ColorMode::Never,
            ..Default::default()
        };
        assert!(!config.use_color(), "Never means never");
    }

    // --- Validation edge cases ---

    #[test]
    fn validate_accepts_no_theme_file() {
        let config = Config {
            theme_file: None,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_rejects_nonexistent_theme_file() {
        let config = Config {
            theme_file: Some(PathBuf::from("/definitely/not/a/real/path.json")),
            ..Default::default()
        };
        match config.validate() {
            Err(ConfigError::ThemeFileNotFound(p)) => {
                assert_eq!(p, PathBuf::from("/definitely/not/a/real/path.json"));
            }
            other => panic!("expected ThemeFileNotFound, got {:?}", other),
        }
    }

    #[test]
    fn validate_rejects_file_as_files_root() {
        // A regular file (not a directory) should fail validation.
        let config = Config {
            files_root: Some(PathBuf::from("/dev/null")),
            ..Default::default()
        };
        let err = config.validate();
        assert!(
            matches!(err, Err(ConfigError::FilesRootNotDirectory(_))),
            "regular file as files_root should fail: {:?}",
            err
        );
    }

    #[test]
    fn validate_accepts_valid_directory() {
        let config = Config {
            files_root: Some(PathBuf::from("/tmp")),
            ..Default::default()
        };
        assert!(config.validate().is_ok(), "/tmp is a valid directory");
    }

    #[test]
    fn config_error_display_messages() {
        // Verify error Display impls produce meaningful messages.
        let err = ConfigError::ThemeFileNotFound(PathBuf::from("bad.json"));
        assert!(err.to_string().contains("bad.json"));

        let err = ConfigError::FilesRootNotFound(PathBuf::from("/missing"));
        assert!(err.to_string().contains("/missing"));

        let err = ConfigError::FilesRootNotDirectory(PathBuf::from("/dev/null"));
        assert!(err.to_string().contains("/dev/null"));

        let err = ConfigError::InvalidTheme("nope".into());
        assert!(err.to_string().contains("nope"));
    }

    // --- Serialization fidelity ---

    #[test]
    fn config_json_roundtrip_all_fields() {
        let config = Config {
            theme_preset: ThemePreset::Dracula,
            theme_file: Some(PathBuf::from("/tmp/theme.json")),
            color_mode: ColorMode::Always,
            animations: AnimationMode::Reduced,
            mouse: true,
            alt_screen: false,
            seed: Some(12345),
            files_root: Some(PathBuf::from("/data")),
            self_check: true,
            verbosity: 3,
            syntax_highlighting: false,
            line_numbers: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.theme_preset, ThemePreset::Dracula);
        assert_eq!(parsed.theme_file, Some(PathBuf::from("/tmp/theme.json")));
        assert_eq!(parsed.color_mode, ColorMode::Always);
        assert_eq!(parsed.animations, AnimationMode::Reduced);
        assert!(parsed.mouse);
        assert!(!parsed.alt_screen);
        assert_eq!(parsed.seed, Some(12345));
        assert_eq!(parsed.files_root, Some(PathBuf::from("/data")));
        assert!(parsed.self_check);
        assert_eq!(parsed.verbosity, 3);
        assert!(!parsed.syntax_highlighting);
        assert!(parsed.line_numbers);
    }

    #[test]
    fn config_default_json_roundtrip() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.theme_preset, config.theme_preset);
        assert_eq!(parsed.color_mode, config.color_mode);
        assert_eq!(parsed.seed, config.seed);
    }

    // --- Diagnostic string ---

    #[test]
    fn diagnostic_string_covers_all_fields() {
        let config = Config {
            theme_file: Some(PathBuf::from("/custom/theme.json")),
            files_root: Some(PathBuf::from("/files")),
            seed: Some(42),
            ..Default::default()
        };

        let diag = config.to_diagnostic_string();
        assert!(diag.contains("Theme:"), "missing theme");
        assert!(diag.contains("Theme file:"), "missing theme file");
        assert!(diag.contains("Color mode:"), "missing color mode");
        assert!(diag.contains("Animations:"), "missing animations");
        assert!(diag.contains("Mouse:"), "missing mouse");
        assert!(diag.contains("Alt screen:"), "missing alt screen");
        assert!(diag.contains("Seed:"), "missing seed");
        assert!(diag.contains("Files root:"), "missing files root");
        assert!(diag.contains("Self-check:"), "missing self-check");
        assert!(diag.contains("Verbosity:"), "missing verbosity");
        assert!(diag.contains("Syntax highlighting:"), "missing syntax");
        assert!(diag.contains("Line numbers:"), "missing line numbers");
    }

    #[test]
    fn diagnostic_string_omits_optional_paths_when_none() {
        let config = Config::default();
        let diag = config.to_diagnostic_string();
        assert!(!diag.contains("Theme file:"), "should omit when None");
        assert!(!diag.contains("Files root:"), "should omit when None");
    }

    // --- Seed behavior ---

    #[test]
    fn effective_seed_deterministic_when_set() {
        let config = Config {
            seed: Some(42),
            ..Default::default()
        };
        assert_eq!(config.effective_seed(), 42);
        assert_eq!(config.effective_seed(), 42);
    }

    #[test]
    fn effective_seed_nonzero_when_generated() {
        let config = Config::default();
        let seed = config.effective_seed();
        assert!(seed > 0, "generated seed should be nonzero");
    }

    // --- is_headless ---

    #[test]
    fn is_headless_tracks_self_check() {
        let config = Config {
            self_check: true,
            ..Default::default()
        };
        assert!(config.is_headless());

        let config = Config {
            self_check: false,
            ..Default::default()
        };
        assert!(!config.is_headless());
    }

    // --- Animation mode helpers ---

    #[test]
    fn reduced_motion_reports_correctly() {
        let config = Config {
            animations: AnimationMode::Reduced,
            ..Default::default()
        };
        assert!(config.use_animations(), "Reduced still means animations on");
        assert!(
            config.reduce_motion(),
            "Reduced should report reduce_motion"
        );
    }

    #[test]
    fn disabled_animations_not_reduced() {
        let config = Config {
            animations: AnimationMode::Disabled,
            ..Default::default()
        };
        assert!(!config.use_animations());
        assert!(!config.reduce_motion(), "Disabled is not Reduced");
    }

    // --- CLI seed edge cases ---

    #[test]
    fn cli_seed_zero_is_valid() {
        let cli = Cli::try_parse_from(["demo_showcase", "--seed", "0"]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.seed, Some(0));
        assert_eq!(config.effective_seed(), 0);
    }

    #[test]
    fn cli_seed_max_u64() {
        let cli = Cli::try_parse_from(["demo_showcase", "--seed", &u64::MAX.to_string()]).unwrap();
        let config = Config::from_cli(&cli);
        assert_eq!(config.seed, Some(u64::MAX));
    }

    #[test]
    fn cli_invalid_seed_rejected() {
        let result = Cli::try_parse_from(["demo_showcase", "--seed", "not_a_number"]);
        assert!(result.is_err(), "non-numeric seed should be rejected");
    }

    #[test]
    fn cli_negative_seed_rejected() {
        let result = Cli::try_parse_from(["demo_showcase", "--seed", "-1"]);
        assert!(result.is_err(), "negative seed should be rejected (u64)");
    }
}
