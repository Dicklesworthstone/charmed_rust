//! Theme system with semantic color slots.
//!
//! The [`Theme`] struct provides semantic color slots that components can reference
//! for consistent styling across an application. Themes support light/dark variants
//! and can be serialized for user configuration.
//!
//! ## Preset Preview
//!
//! <table>
//!   <tr><th>Preset</th><th>Background</th><th>Primary</th><th>Text</th></tr>
//!   <tr><td>Dark</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#0f0f0f;border:1px solid #999"></span> `#0f0f0f`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#7c3aed;border:1px solid #999"></span> `#7c3aed`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#fafafa;border:1px solid #999"></span> `#fafafa`</td></tr>
//!   <tr><td>Light</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#ffffff;border:1px solid #999"></span> `#ffffff`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#7c3aed;border:1px solid #999"></span> `#7c3aed`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#18181b;border:1px solid #999"></span> `#18181b`</td></tr>
//!   <tr><td>Dracula</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#282a36;border:1px solid #999"></span> `#282a36`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#bd93f9;border:1px solid #999"></span> `#bd93f9`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#f8f8f2;border:1px solid #999"></span> `#f8f8f2`</td></tr>
//!   <tr><td>Nord</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#2e3440;border:1px solid #999"></span> `#2e3440`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#88c0d0;border:1px solid #999"></span> `#88c0d0`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#eceff4;border:1px solid #999"></span> `#eceff4`</td></tr>
//!   <tr><td>Catppuccin Latte</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#eff1f5;border:1px solid #999"></span> `#eff1f5`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#8839ef;border:1px solid #999"></span> `#8839ef`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#4c4f69;border:1px solid #999"></span> `#4c4f69`</td></tr>
//!   <tr><td>Catppuccin Frappe</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#303446;border:1px solid #999"></span> `#303446`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#ca9ee6;border:1px solid #999"></span> `#ca9ee6`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#c6d0f5;border:1px solid #999"></span> `#c6d0f5`</td></tr>
//!   <tr><td>Catppuccin Macchiato</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#24273a;border:1px solid #999"></span> `#24273a`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#c6a0f6;border:1px solid #999"></span> `#c6a0f6`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#cad3f5;border:1px solid #999"></span> `#cad3f5`</td></tr>
//!   <tr><td>Catppuccin Mocha</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#1e1e2e;border:1px solid #999"></span> `#1e1e2e`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#cba6f7;border:1px solid #999"></span> `#cba6f7`</td><td><span style="display:inline-block;width:0.9em;height:0.9em;background:#cdd6f4;border:1px solid #999"></span> `#cdd6f4`</td></tr>
//! </table>
//!
//! # Example
//!
//! ```rust
//! use lipgloss::theme::{Theme, ThemeColors};
//!
//! // Use the default dark theme
//! let theme = Theme::dark();
//!
//! // Create a style using theme colors
//! let style = theme.style()
//!     .foreground_color(theme.colors().primary.clone())
//!     .background_color(theme.colors().background.clone());
//! ```

use crate::border::Border;
use crate::color::{AdaptiveColor, Color, ansi256_to_rgb};
use crate::position::{Position, Sides};
use crate::renderer::Renderer;
use crate::style::Style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
#[cfg(feature = "native")]
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(feature = "native")]
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, RwLock};
use thiserror::Error;
use tracing::{debug, info, trace, warn};

/// A complete theme with semantic color slots.
///
/// Themes provide a consistent color palette that components can reference
/// by semantic meaning (e.g., "primary", "error") rather than raw color values.
/// This enables easy theme switching and ensures visual consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Human-readable name for this theme.
    #[serde(default)]
    name: String,

    /// Whether this is a dark theme (affects adaptive color selection).
    #[serde(default)]
    is_dark: bool,

    /// The color palette.
    #[serde(default)]
    colors: ThemeColors,

    /// Optional theme description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    /// Optional theme author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    author: Option<String>,

    /// Optional theme metadata.
    #[serde(default, skip_serializing_if = "ThemeMeta::is_empty")]
    meta: ThemeMeta,
}

/// Semantic color slots for a theme.
///
/// Each slot represents a semantic purpose rather than a specific color.
/// This allows the same code to work with different themes while maintaining
/// appropriate visual meaning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // ========================
    // Primary Palette
    // ========================
    /// Primary brand/accent color. Used for primary actions, links, and emphasis.
    pub primary: Color,

    /// Secondary color. Used for secondary actions and less prominent elements.
    pub secondary: Color,

    /// Accent color. Used for highlights, indicators, and visual interest.
    pub accent: Color,

    // ========================
    // Background Colors
    // ========================
    /// Main background color.
    pub background: Color,

    /// Elevated surface color (cards, dialogs, popups).
    pub surface: Color,

    /// Alternative surface for visual layering.
    pub surface_alt: Color,

    // ========================
    // Text Colors
    // ========================
    /// Primary text color (high contrast, main content).
    pub text: Color,

    /// Muted text color (secondary content, descriptions).
    pub text_muted: Color,

    /// Disabled text color (inactive elements).
    pub text_disabled: Color,

    // ========================
    // Semantic Colors
    // ========================
    /// Success/positive color (confirmations, success states).
    pub success: Color,

    /// Warning color (cautions, alerts).
    pub warning: Color,

    /// Error/danger color (errors, destructive actions).
    pub error: Color,

    /// Info color (informational messages, neutral highlights).
    pub info: Color,

    // ========================
    // UI Element Colors
    // ========================
    /// Border color for UI elements.
    pub border: Color,

    /// Subtle border color (dividers, separators).
    pub border_muted: Color,

    /// Separator/divider color.
    pub separator: Color,

    // ========================
    // Interactive States
    // ========================
    /// Focus indicator color.
    pub focus: Color,

    /// Selection/highlight background color.
    pub selection: Color,

    /// Hover state color.
    pub hover: Color,

    // ========================
    // Code/Syntax Colors
    // ========================
    /// Code/syntax: Keywords (if, else, fn, etc.)
    pub code_keyword: Color,

    /// Code/syntax: Strings
    pub code_string: Color,

    /// Code/syntax: Numbers
    pub code_number: Color,

    /// Code/syntax: Comments
    pub code_comment: Color,

    /// Code/syntax: Function names
    pub code_function: Color,

    /// Code/syntax: Types/classes
    pub code_type: Color,

    /// Code/syntax: Variables
    pub code_variable: Color,

    /// Code/syntax: Operators
    pub code_operator: Color,

    /// Custom color slots.
    #[serde(default, flatten)]
    pub custom: HashMap<String, Color>,
}

/// Named color slots for referencing theme colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSlot {
    /// Primary brand/accent color.
    Primary,
    /// Secondary accent color.
    Secondary,
    /// Highlight/accent color.
    Accent,
    /// Background color.
    Background,
    /// Foreground/text color (alias of `Text`).
    Foreground,
    /// Primary text color.
    Text,
    /// Muted/secondary text color.
    TextMuted,
    /// Disabled text color.
    TextDisabled,
    /// Elevated surface color.
    Surface,
    /// Alternative surface color.
    SurfaceAlt,
    /// Success/positive color.
    Success,
    /// Warning color.
    Warning,
    /// Error/danger color.
    Error,
    /// Informational color.
    Info,
    /// Border color.
    Border,
    /// Subtle border color.
    BorderMuted,
    /// Divider/separator color.
    Separator,
    /// Focus indicator color.
    Focus,
    /// Selection/background highlight color.
    Selection,
    /// Hover state color.
    Hover,
    /// Code/syntax keyword color.
    CodeKeyword,
    /// Code/syntax string color.
    CodeString,
    /// Code/syntax number color.
    CodeNumber,
    /// Code/syntax comment color.
    CodeComment,
    /// Code/syntax function color.
    CodeFunction,
    /// Code/syntax type color.
    CodeType,
    /// Code/syntax variable color.
    CodeVariable,
    /// Code/syntax operator color.
    CodeOperator,
}

/// Semantic roles for quick style creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeRole {
    /// Primary/brand role.
    Primary,
    /// Success/positive role.
    Success,
    /// Warning role.
    Warning,
    /// Error/danger role.
    Error,
    /// Muted/secondary role.
    Muted,
    /// Inverted role (swap foreground/background).
    Inverted,
}

/// Built-in theme presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemePreset {
    Dark,
    Light,
    Dracula,
    Nord,
    Catppuccin(CatppuccinFlavor),
}

/// Catppuccin palette flavors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatppuccinFlavor {
    /// Light flavor.
    Latte,
    /// Medium-light flavor.
    Frappe,
    /// Medium-dark flavor.
    Macchiato,
    /// Dark flavor.
    Mocha,
}

impl fmt::Display for CatppuccinFlavor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Latte => "Latte",
            Self::Frappe => "Frappe",
            Self::Macchiato => "Macchiato",
            Self::Mocha => "Mocha",
        };
        f.write_str(name)
    }
}

impl fmt::Display for ThemePreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dark => f.write_str("Dark"),
            Self::Light => f.write_str("Light"),
            Self::Dracula => f.write_str("Dracula"),
            Self::Nord => f.write_str("Nord"),
            Self::Catppuccin(flavor) => write!(f, "Catppuccin {flavor}"),
        }
    }
}

/// Theme variant for light/dark themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    Light,
    Dark,
}

/// Optional metadata for themes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<ThemeVariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl ThemeMeta {
    fn is_empty(&self) -> bool {
        self.version.is_none() && self.variant.is_none() && self.source.is_none()
    }
}

/// Identifier for a registered theme change listener.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(u64);

/// Listener callback for theme changes.
pub trait ThemeChangeListener: Send + Sync {
    fn on_theme_change(&self, theme: &Theme);
}

impl<F> ThemeChangeListener for F
where
    F: Fn(&Theme) + Send + Sync,
{
    fn on_theme_change(&self, theme: &Theme) {
        self(theme);
    }
}

/// Thread-safe context for runtime theme switching.
#[derive(Clone)]
pub struct ThemeContext {
    current: Arc<RwLock<Theme>>,
    listeners: Arc<RwLock<HashMap<ListenerId, Arc<dyn ThemeChangeListener>>>>,
    next_listener_id: Arc<AtomicU64>,
}

impl fmt::Debug for ThemeContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ThemeContext")
            .field("current", &"<RwLock<Theme>>")
            .field(
                "listeners",
                &format!(
                    "{} listeners",
                    self.listeners.read().map(|l| l.len()).unwrap_or(0)
                ),
            )
            .field("next_listener_id", &self.next_listener_id)
            .finish()
    }
}

impl ThemeContext {
    /// Create a new context with the provided theme.
    pub fn new(initial: Theme) -> Self {
        Self {
            current: Arc::new(RwLock::new(initial)),
            listeners: Arc::new(RwLock::new(HashMap::new())),
            next_listener_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Create a new context from a preset.
    pub fn from_preset(preset: ThemePreset) -> Self {
        Self::new(preset.to_theme())
    }

    /// Returns a read guard for the current theme.
    pub fn current(&self) -> std::sync::RwLockReadGuard<'_, Theme> {
        let guard = self.current.read().expect("theme context lock poisoned");
        trace!(theme.name = %guard.name(), "Theme read");
        guard
    }

    /// Switch to a new theme and notify listeners.
    pub fn set_theme(&self, theme: Theme) {
        let from = {
            let current = self.current.read().expect("theme context lock poisoned");
            current.name().to_string()
        };
        let to = theme.name().to_string();
        let snapshot = theme.clone();
        {
            let mut current = self.current.write().expect("theme context lock poisoned");
            *current = theme;
        }

        info!(theme.from = %from, theme.to = %to, "Theme switched");
        self.notify_listeners(&snapshot);
    }

    /// Switch to a preset theme and notify listeners.
    pub fn set_preset(&self, preset: ThemePreset) {
        self.set_theme(preset.to_theme());
    }

    /// Register a listener for theme changes.
    pub fn on_change<F>(&self, callback: F) -> ListenerId
    where
        F: Fn(&Theme) + Send + Sync + 'static,
    {
        let id = ListenerId(self.next_listener_id.fetch_add(1, Ordering::Relaxed));
        self.listeners
            .write()
            .expect("theme listener lock poisoned")
            .insert(id, Arc::new(callback));
        debug!(theme.listener_id = id.0, "Theme listener registered");
        id
    }

    /// Remove a listener by id.
    pub fn remove_listener(&self, id: ListenerId) {
        let mut listeners = self
            .listeners
            .write()
            .expect("theme listener lock poisoned");
        if listeners.remove(&id).is_some() {
            debug!(theme.listener_id = id.0, "Theme listener removed");
        }
    }

    fn notify_listeners(&self, theme: &Theme) {
        let listeners: Vec<(ListenerId, Arc<dyn ThemeChangeListener>)> = {
            let listeners = self.listeners.read().expect("theme listener lock poisoned");
            listeners
                .iter()
                .map(|(id, listener)| (*id, Arc::clone(listener)))
                .collect()
        };

        for (id, listener) in listeners {
            let result = catch_unwind(AssertUnwindSafe(|| listener.on_theme_change(theme)));
            if result.is_err() {
                warn!(
                    theme.listener_id = id.0,
                    theme.name = %theme.name(),
                    "Theme listener panicked"
                );
            }
        }
    }
}

static GLOBAL_THEME_CONTEXT: LazyLock<ThemeContext> =
    LazyLock::new(|| ThemeContext::from_preset(ThemePreset::Dark));

/// Returns the global theme context.
pub fn global_theme() -> &'static ThemeContext {
    &GLOBAL_THEME_CONTEXT
}

/// Replace the global theme.
pub fn set_global_theme(theme: Theme) {
    GLOBAL_THEME_CONTEXT.set_theme(theme);
}

/// Replace the global theme using a preset.
pub fn set_global_preset(preset: ThemePreset) {
    GLOBAL_THEME_CONTEXT.set_preset(preset);
}

// -----------------------------------------------------------------------------
// Themed Styles (auto-resolve colors from ThemeContext)
// -----------------------------------------------------------------------------

/// A style that automatically resolves colors from the current theme.
#[derive(Clone, Debug)]
pub struct ThemedStyle {
    context: Arc<ThemeContext>,
    foreground: Option<ThemedColor>,
    background: Option<ThemedColor>,
    border_foreground: Option<ThemedColor>,
    border_background: Option<ThemedColor>,
    base_style: Style,
}

/// Color that can be fixed, sourced from a theme slot, or computed.
#[derive(Clone, Debug)]
pub enum ThemedColor {
    /// Fixed color value.
    Fixed(Color),
    /// Color resolved from theme at render time.
    Slot(ColorSlot),
    /// Computed color based on a theme slot.
    Computed(ColorSlot, ColorTransform),
}

/// Transformations applied to theme colors at resolve time.
#[derive(Clone, Copy, Debug)]
pub enum ColorTransform {
    /// Lighten by 0.0-1.0.
    Lighten(f32),
    /// Darken by 0.0-1.0.
    Darken(f32),
    /// Increase saturation by 0.0-1.0.
    Saturate(f32),
    /// Decrease saturation by 0.0-1.0.
    Desaturate(f32),
    /// Apply alpha (approximated for terminal colors).
    Alpha(f32),
}

impl ThemedStyle {
    /// Create a new themed style with a context.
    pub fn new(context: Arc<ThemeContext>) -> Self {
        Self {
            context,
            foreground: None,
            background: None,
            border_foreground: None,
            border_background: None,
            base_style: Style::new(),
        }
    }

    /// Create from the global theme context.
    pub fn global() -> Self {
        Self::new(Arc::new(global_theme().clone()))
    }

    /// Resolve the themed style to a concrete Style using the current theme.
    pub fn resolve(&self) -> Style {
        let Ok(theme) = catch_unwind(AssertUnwindSafe(|| self.context.current())) else {
            warn!("themed_style.resolve called without a valid theme context");
            return self.base_style.clone();
        };

        let mut style = self.base_style.clone();

        if let Some(ref fg) = self.foreground {
            let color = Self::resolve_color(fg, &theme);
            style = style.foreground_color(color);
        }
        if let Some(ref bg) = self.background {
            let color = Self::resolve_color(bg, &theme);
            style = style.background_color(color);
        }
        if let Some(ref bfg) = self.border_foreground {
            let color = Self::resolve_color(bfg, &theme);
            style = style.border_foreground(color.0);
        }
        if let Some(ref bbg) = self.border_background {
            let color = Self::resolve_color(bbg, &theme);
            style = style.border_background(color.0);
        }

        drop(theme);
        style
    }

    /// Render text with the themed style (resolves at call time).
    pub fn render(&self, text: &str) -> String {
        self.resolve().render(text)
    }

    fn resolve_color(themed: &ThemedColor, theme: &Theme) -> Color {
        match themed {
            ThemedColor::Fixed(color) => {
                debug!(themed_style.resolve = true, color_kind = "fixed", color = %color.0);
                color.clone()
            }
            ThemedColor::Slot(slot) => {
                let color = theme.get(*slot);
                debug!(
                    themed_style.resolve = true,
                    color_kind = "slot",
                    color_slot = ?slot,
                    color = %color.0
                );
                color
            }
            ThemedColor::Computed(slot, transform) => {
                let base = theme.get(*slot);
                let color = transform.apply(base);
                debug!(
                    themed_style.resolve = true,
                    color_kind = "computed",
                    color_slot = ?slot,
                    transform = ?transform,
                    color = %color.0
                );
                color
            }
        }
    }

    // ---------------------------------------------------------------------
    // Theme-aware color setters
    // ---------------------------------------------------------------------

    /// Set foreground to a theme color slot.
    pub fn foreground(mut self, slot: ColorSlot) -> Self {
        self.foreground = Some(ThemedColor::Slot(slot));
        self
    }

    /// Set foreground to a fixed color (ignores theme).
    pub fn foreground_fixed(mut self, color: impl Into<Color>) -> Self {
        self.foreground = Some(ThemedColor::Fixed(color.into()));
        self
    }

    /// Set foreground to a computed theme color.
    pub fn foreground_computed(mut self, slot: ColorSlot, transform: ColorTransform) -> Self {
        self.foreground = Some(ThemedColor::Computed(slot, transform));
        self
    }

    /// Clear any themed foreground.
    pub fn no_foreground(mut self) -> Self {
        self.foreground = None;
        self.base_style = self.base_style.no_foreground();
        self
    }

    /// Set background to a theme color slot.
    pub fn background(mut self, slot: ColorSlot) -> Self {
        self.background = Some(ThemedColor::Slot(slot));
        self
    }

    /// Set background to a fixed color (ignores theme).
    pub fn background_fixed(mut self, color: impl Into<Color>) -> Self {
        self.background = Some(ThemedColor::Fixed(color.into()));
        self
    }

    /// Set background to a computed theme color.
    pub fn background_computed(mut self, slot: ColorSlot, transform: ColorTransform) -> Self {
        self.background = Some(ThemedColor::Computed(slot, transform));
        self
    }

    /// Clear any themed background.
    pub fn no_background(mut self) -> Self {
        self.background = None;
        self.base_style = self.base_style.no_background();
        self
    }

    /// Set border foreground to a theme color slot.
    pub fn border_foreground(mut self, slot: ColorSlot) -> Self {
        self.border_foreground = Some(ThemedColor::Slot(slot));
        self
    }

    /// Set border foreground to a fixed color.
    pub fn border_foreground_fixed(mut self, color: impl Into<Color>) -> Self {
        self.border_foreground = Some(ThemedColor::Fixed(color.into()));
        self
    }

    /// Set border foreground to a computed theme color.
    pub fn border_foreground_computed(
        mut self,
        slot: ColorSlot,
        transform: ColorTransform,
    ) -> Self {
        self.border_foreground = Some(ThemedColor::Computed(slot, transform));
        self
    }

    /// Set border background to a theme color slot.
    pub fn border_background(mut self, slot: ColorSlot) -> Self {
        self.border_background = Some(ThemedColor::Slot(slot));
        self
    }

    /// Set border background to a fixed color.
    pub fn border_background_fixed(mut self, color: impl Into<Color>) -> Self {
        self.border_background = Some(ThemedColor::Fixed(color.into()));
        self
    }

    /// Set border background to a computed theme color.
    pub fn border_background_computed(
        mut self,
        slot: ColorSlot,
        transform: ColorTransform,
    ) -> Self {
        self.border_background = Some(ThemedColor::Computed(slot, transform));
        self
    }

    // ---------------------------------------------------------------------
    // Delegated non-color style methods
    // ---------------------------------------------------------------------

    /// Set the underlying string value for this style.
    pub fn set_string(mut self, s: impl Into<String>) -> Self {
        self.base_style = self.base_style.set_string(s);
        self
    }

    /// Get the underlying string value.
    pub fn value(&self) -> &str {
        self.base_style.value()
    }

    /// Enable bold text.
    pub fn bold(mut self) -> Self {
        self.base_style = self.base_style.bold();
        self
    }

    /// Enable italic text.
    pub fn italic(mut self) -> Self {
        self.base_style = self.base_style.italic();
        self
    }

    /// Enable underline text.
    pub fn underline(mut self) -> Self {
        self.base_style = self.base_style.underline();
        self
    }

    /// Enable strikethrough text.
    pub fn strikethrough(mut self) -> Self {
        self.base_style = self.base_style.strikethrough();
        self
    }

    /// Enable reverse video.
    pub fn reverse(mut self) -> Self {
        self.base_style = self.base_style.reverse();
        self
    }

    /// Enable blinking.
    pub fn blink(mut self) -> Self {
        self.base_style = self.base_style.blink();
        self
    }

    /// Enable faint text.
    pub fn faint(mut self) -> Self {
        self.base_style = self.base_style.faint();
        self
    }

    /// Toggle underline spaces.
    pub fn underline_spaces(mut self, v: bool) -> Self {
        self.base_style = self.base_style.underline_spaces(v);
        self
    }

    /// Toggle strikethrough spaces.
    pub fn strikethrough_spaces(mut self, v: bool) -> Self {
        self.base_style = self.base_style.strikethrough_spaces(v);
        self
    }

    /// Set fixed width.
    pub fn width(mut self, w: u16) -> Self {
        self.base_style = self.base_style.width(w);
        self
    }

    /// Set fixed height.
    pub fn height(mut self, h: u16) -> Self {
        self.base_style = self.base_style.height(h);
        self
    }

    /// Set maximum width.
    pub fn max_width(mut self, w: u16) -> Self {
        self.base_style = self.base_style.max_width(w);
        self
    }

    /// Set maximum height.
    pub fn max_height(mut self, h: u16) -> Self {
        self.base_style = self.base_style.max_height(h);
        self
    }

    /// Set both horizontal and vertical alignment.
    pub fn align(mut self, p: Position) -> Self {
        self.base_style = self.base_style.align(p);
        self
    }

    /// Set horizontal alignment.
    pub fn align_horizontal(mut self, p: Position) -> Self {
        self.base_style = self.base_style.align_horizontal(p);
        self
    }

    /// Set vertical alignment.
    pub fn align_vertical(mut self, p: Position) -> Self {
        self.base_style = self.base_style.align_vertical(p);
        self
    }

    /// Set padding.
    pub fn padding(mut self, sides: impl Into<Sides<u16>>) -> Self {
        self.base_style = self.base_style.padding(sides);
        self
    }

    /// Set padding top.
    pub fn padding_top(mut self, n: u16) -> Self {
        self.base_style = self.base_style.padding_top(n);
        self
    }

    /// Set padding right.
    pub fn padding_right(mut self, n: u16) -> Self {
        self.base_style = self.base_style.padding_right(n);
        self
    }

    /// Set padding bottom.
    pub fn padding_bottom(mut self, n: u16) -> Self {
        self.base_style = self.base_style.padding_bottom(n);
        self
    }

    /// Set padding left.
    pub fn padding_left(mut self, n: u16) -> Self {
        self.base_style = self.base_style.padding_left(n);
        self
    }

    /// Set margin.
    pub fn margin(mut self, sides: impl Into<Sides<u16>>) -> Self {
        self.base_style = self.base_style.margin(sides);
        self
    }

    /// Set margin top.
    pub fn margin_top(mut self, n: u16) -> Self {
        self.base_style = self.base_style.margin_top(n);
        self
    }

    /// Set margin right.
    pub fn margin_right(mut self, n: u16) -> Self {
        self.base_style = self.base_style.margin_right(n);
        self
    }

    /// Set margin bottom.
    pub fn margin_bottom(mut self, n: u16) -> Self {
        self.base_style = self.base_style.margin_bottom(n);
        self
    }

    /// Set margin left.
    pub fn margin_left(mut self, n: u16) -> Self {
        self.base_style = self.base_style.margin_left(n);
        self
    }

    /// Set margin background (fixed color).
    pub fn margin_background(mut self, color: impl Into<String>) -> Self {
        self.base_style = self.base_style.margin_background(color);
        self
    }

    /// Set border style.
    pub fn border(mut self, border: Border) -> Self {
        self.base_style = self.base_style.border(border);
        self
    }

    /// Set border style (alias).
    pub fn border_style(mut self, border: Border) -> Self {
        self.base_style = self.base_style.border_style(border);
        self
    }

    /// Enable or disable top border.
    pub fn border_top(mut self, v: bool) -> Self {
        self.base_style = self.base_style.border_top(v);
        self
    }

    /// Enable or disable right border.
    pub fn border_right(mut self, v: bool) -> Self {
        self.base_style = self.base_style.border_right(v);
        self
    }

    /// Enable or disable bottom border.
    pub fn border_bottom(mut self, v: bool) -> Self {
        self.base_style = self.base_style.border_bottom(v);
        self
    }

    /// Enable or disable left border.
    pub fn border_left(mut self, v: bool) -> Self {
        self.base_style = self.base_style.border_left(v);
        self
    }

    /// Inline mode.
    pub fn inline(mut self) -> Self {
        self.base_style = self.base_style.inline();
        self
    }

    /// Set tab width.
    pub fn tab_width(mut self, n: i8) -> Self {
        self.base_style = self.base_style.tab_width(n);
        self
    }

    /// Apply a transform function to the rendered string.
    pub fn transform<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.base_style = self.base_style.transform(f);
        self
    }

    /// Set the renderer.
    pub fn renderer(mut self, r: Arc<Renderer>) -> Self {
        self.base_style = self.base_style.renderer(r);
        self
    }

    /// Check if a property is set on the base style.
    pub fn is_set(&self, prop: crate::style::Props) -> bool {
        self.base_style.is_set(prop)
    }
}

#[allow(clippy::many_single_char_names)]
impl ColorTransform {
    fn apply(self, color: Color) -> Color {
        let (r, g, b) = if let Some((r, g, b)) = color.as_rgb() {
            (r, g, b)
        } else if let Some(n) = color.as_ansi() {
            ansi256_to_rgb(n)
        } else {
            return color;
        };

        let (h, mut s, mut l) = rgb_to_hsl(r, g, b);
        let amount = |v: f32| v.clamp(0.0, 1.0);

        match self {
            ColorTransform::Lighten(a) => l = (l + amount(a)).min(1.0),
            ColorTransform::Darken(a) => l = (l - amount(a)).max(0.0),
            ColorTransform::Saturate(a) => s = (s + amount(a)).min(1.0),
            ColorTransform::Desaturate(a) => s = (s - amount(a)).max(0.0),
            ColorTransform::Alpha(a) => {
                let a = amount(a);
                l = (l * a).min(1.0);
            }
        }

        let (nr, ng, nb) = hsl_to_rgb(h, s, l);
        Color::from(format!("#{:02x}{:02x}{:02x}", nr, ng, nb))
    }
}

#[allow(clippy::many_single_char_names)]
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = f32::midpoint(max, min);

    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let mut h = if (max - r).abs() < f32::EPSILON {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    h /= 6.0;
    (h * 360.0, s, l)
}

#[allow(clippy::many_single_char_names, clippy::suboptimal_flops)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let h = h / 360.0;
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    }

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

/// Cached themed style that invalidates on theme changes.
pub struct CachedThemedStyle {
    themed: ThemedStyle,
    cache: Arc<RwLock<Option<Style>>>,
    listener_id: ListenerId,
}

impl CachedThemedStyle {
    /// Create a cached themed style.
    pub fn new(themed: ThemedStyle) -> Self {
        let cache = Arc::new(RwLock::new(None));
        let cache_ref = Arc::clone(&cache);
        let listener_id = themed.context.on_change(move |_theme| {
            if let Ok(mut guard) = cache_ref.write() {
                *guard = None;
            }
            trace!("cached_themed_style cache invalidated");
        });

        Self {
            themed,
            cache,
            listener_id,
        }
    }

    /// Resolve with caching.
    pub fn resolve(&self) -> Style {
        if let Ok(cache) = self.cache.read() {
            if let Some(style) = cache.as_ref() {
                trace!("cached_themed_style cache hit");
                return style.clone();
            }
        }

        trace!("cached_themed_style cache miss");
        let resolved = self.themed.resolve();
        if let Ok(mut cache) = self.cache.write() {
            *cache = Some(resolved.clone());
        }
        resolved
    }

    /// Render text using the cached themed style.
    pub fn render(&self, text: &str) -> String {
        self.resolve().render(text)
    }

    /// Manually invalidate the cache.
    pub fn invalidate(&self) {
        if let Ok(mut cache) = self.cache.write() {
            *cache = None;
        }
    }
}

impl Drop for CachedThemedStyle {
    fn drop(&mut self) {
        self.themed.context.remove_listener(self.listener_id);
    }
}

/// Async theme context backed by a tokio watch channel.
#[cfg(feature = "tokio")]
pub struct AsyncThemeContext {
    sender: tokio::sync::watch::Sender<Theme>,
    receiver: tokio::sync::watch::Receiver<Theme>,
}

#[cfg(feature = "tokio")]
impl AsyncThemeContext {
    /// Create a new async context with the provided theme.
    pub fn new(initial: Theme) -> Self {
        let (sender, receiver) = tokio::sync::watch::channel(initial);
        Self { sender, receiver }
    }

    /// Create a new async context from a preset.
    pub fn from_preset(preset: ThemePreset) -> Self {
        Self::new(preset.to_theme())
    }

    /// Returns the current theme snapshot.
    pub fn current(&self) -> Theme {
        self.receiver.borrow().clone()
    }

    /// Switch to a new theme.
    pub fn set_theme(&self, theme: Theme) {
        let from = self.receiver.borrow().name().to_string();
        let to = theme.name().to_string();
        let _ = self.sender.send(theme);
        info!(theme.from = %from, theme.to = %to, "Theme switched (async)");
    }

    /// Switch to a preset theme.
    pub fn set_preset(&self, preset: ThemePreset) {
        self.set_theme(preset.to_theme());
    }

    /// Subscribe to theme changes.
    pub fn subscribe(&self) -> tokio::sync::watch::Receiver<Theme> {
        self.receiver.clone()
    }

    /// Await the next theme change.
    ///
    /// # Errors
    ///
    /// Returns an error if the sender has been dropped.
    pub async fn changed(&mut self) -> Result<(), tokio::sync::watch::error::RecvError> {
        self.receiver.changed().await
    }
}

impl Theme {
    /// Creates a new theme with the given name, dark mode flag, and colors.
    pub fn new(name: impl Into<String>, is_dark: bool, colors: ThemeColors) -> Self {
        let meta = ThemeMeta {
            variant: Some(if is_dark {
                ThemeVariant::Dark
            } else {
                ThemeVariant::Light
            }),
            ..ThemeMeta::default()
        };
        Self {
            name: name.into(),
            is_dark,
            colors,
            description: None,
            author: None,
            meta,
        }
    }

    /// Returns the theme name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns true if this is a dark theme.
    pub fn is_dark(&self) -> bool {
        self.is_dark
    }

    /// Returns the optional theme description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the optional theme author.
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    /// Returns the theme metadata.
    pub fn meta(&self) -> &ThemeMeta {
        &self.meta
    }

    /// Returns a mutable reference to the theme metadata.
    pub fn meta_mut(&mut self) -> &mut ThemeMeta {
        &mut self.meta
    }

    /// Returns the theme's color palette.
    pub fn colors(&self) -> &ThemeColors {
        &self.colors
    }

    /// Returns a mutable reference to the theme's color palette.
    pub fn colors_mut(&mut self) -> &mut ThemeColors {
        &mut self.colors
    }

    /// Returns the color for the given slot.
    pub fn get(&self, slot: ColorSlot) -> Color {
        self.colors.get(slot).clone()
    }

    /// Creates a new Style configured to use this theme.
    ///
    /// The returned style has no properties set but is configured to use
    /// this theme's renderer settings.
    pub fn style(&self) -> Style {
        Style::new()
    }

    // ========================
    // Builder Methods
    // ========================

    /// Sets the theme name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets whether this is a dark theme.
    pub fn with_dark(mut self, is_dark: bool) -> Self {
        self.is_dark = is_dark;
        self.meta.variant = Some(if is_dark {
            ThemeVariant::Dark
        } else {
            ThemeVariant::Light
        });
        self
    }

    /// Replaces the color palette.
    pub fn with_colors(mut self, colors: ThemeColors) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the theme description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the theme author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Replaces theme metadata.
    pub fn with_meta(mut self, meta: ThemeMeta) -> Self {
        self.meta = meta;
        self
    }

    /// Validate that this theme has usable color values.
    ///
    /// # Errors
    /// Returns `ThemeValidationError` if any color slot is empty or invalid.
    pub fn validate(&self) -> Result<(), ThemeValidationError> {
        self.colors.validate()?;
        let _ = self.check_contrast_aa(ColorSlot::Foreground, ColorSlot::Background);
        Ok(())
    }

    /// Calculate contrast ratio between two theme slots.
    pub fn contrast_ratio(&self, fg: ColorSlot, bg: ColorSlot) -> f64 {
        let fg_lum = self.get(fg).relative_luminance();
        let bg_lum = self.get(bg).relative_luminance();
        let lighter = fg_lum.max(bg_lum);
        let darker = fg_lum.min(bg_lum);
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Check if a slot combination meets WCAG AA contrast (>= 4.5:1).
    pub fn check_contrast_aa(&self, fg: ColorSlot, bg: ColorSlot) -> bool {
        let ratio = self.contrast_ratio(fg, bg);
        let ok = ratio >= 4.5;
        if !ok {
            warn!(
                theme.contrast_ratio = ratio,
                theme.fg = ?fg,
                theme.bg = ?bg,
                theme.name = %self.name(),
                "Theme contrast below WCAG AA"
            );
        }
        ok
    }

    /// Check if a slot combination meets WCAG AAA contrast (>= 7.0:1).
    pub fn check_contrast_aaa(&self, fg: ColorSlot, bg: ColorSlot) -> bool {
        self.contrast_ratio(fg, bg) >= 7.0
    }

    fn normalize(&mut self) {
        if let Some(variant) = self.meta.variant {
            self.is_dark = matches!(variant, ThemeVariant::Dark);
        } else {
            self.meta.variant = Some(if self.is_dark {
                ThemeVariant::Dark
            } else {
                ThemeVariant::Light
            });
        }
    }

    /// Load a theme from JSON text.
    ///
    /// # Errors
    /// Returns `ThemeLoadError` if JSON parsing or validation fails.
    pub fn from_json(json: &str) -> Result<Self, ThemeLoadError> {
        let mut theme: Theme = serde_json::from_str(json)?;
        theme.normalize();
        theme.validate()?;
        Ok(theme)
    }

    /// Load a theme from TOML text.
    ///
    /// # Errors
    /// Returns `ThemeLoadError` if TOML parsing or validation fails.
    pub fn from_toml(toml: &str) -> Result<Self, ThemeLoadError> {
        let mut theme: Theme = toml::from_str(toml)?;
        theme.normalize();
        theme.validate()?;
        Ok(theme)
    }

    /// Load a theme from YAML text.
    ///
    /// # Errors
    /// Returns `ThemeLoadError` if YAML parsing or validation fails.
    #[cfg(feature = "yaml")]
    pub fn from_yaml(yaml: &str) -> Result<Self, ThemeLoadError> {
        let mut theme: Theme = serde_yaml::from_str(yaml)?;
        theme.normalize();
        theme.validate()?;
        Ok(theme)
    }

    /// Load a theme from a file (format inferred by extension).
    ///
    /// # Errors
    /// Returns `ThemeLoadError` if reading, parsing, or validation fails.
    ///
    /// # Availability
    /// This method is only available with the `native` feature (not on WASM).
    #[cfg(feature = "native")]
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ThemeLoadError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Self::from_json(&content),
            Some("toml") => Self::from_toml(&content),
            Some("yaml" | "yml") => {
                #[cfg(feature = "yaml")]
                {
                    Self::from_yaml(&content)
                }
                #[cfg(not(feature = "yaml"))]
                {
                    Err(ThemeLoadError::UnsupportedFormat("yaml".into()))
                }
            }
            Some(ext) => Err(ThemeLoadError::UnsupportedFormat(ext.into())),
            None => Err(ThemeLoadError::UnsupportedFormat("unknown".into())),
        }
    }

    /// Serialize this theme to JSON.
    ///
    /// # Errors
    /// Returns `ThemeSaveError` if serialization fails.
    pub fn to_json(&self) -> Result<String, ThemeSaveError> {
        serde_json::to_string_pretty(self).map_err(ThemeSaveError::Json)
    }

    /// Serialize this theme to TOML.
    ///
    /// # Errors
    /// Returns `ThemeSaveError` if serialization fails.
    pub fn to_toml(&self) -> Result<String, ThemeSaveError> {
        toml::to_string_pretty(self).map_err(ThemeSaveError::Toml)
    }

    /// Serialize this theme to YAML.
    ///
    /// # Errors
    /// Returns `ThemeSaveError` if serialization fails.
    #[cfg(feature = "yaml")]
    pub fn to_yaml(&self) -> Result<String, ThemeSaveError> {
        serde_yaml::to_string(self).map_err(ThemeSaveError::Yaml)
    }

    /// Save this theme to a file (format inferred by extension).
    ///
    /// # Errors
    /// Returns `ThemeSaveError` if serialization or writing fails.
    ///
    /// # Availability
    /// This method is only available with the `native` feature (not on WASM).
    #[cfg(feature = "native")]
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), ThemeSaveError> {
        let path = path.as_ref();
        let content = match path.extension().and_then(|e| e.to_str()) {
            Some("json") | None => self.to_json()?,
            Some("toml") => self.to_toml()?,
            Some("yaml" | "yml") => {
                #[cfg(feature = "yaml")]
                {
                    self.to_yaml()?
                }
                #[cfg(not(feature = "yaml"))]
                {
                    return Err(ThemeSaveError::UnsupportedFormat("yaml".into()));
                }
            }
            Some(ext) => return Err(ThemeSaveError::UnsupportedFormat(ext.into())),
        };

        fs::write(path, content).map_err(ThemeSaveError::Io)
    }

    // ========================
    // Default Themes
    // ========================

    /// Returns the default dark theme.
    ///
    /// This theme uses colors suitable for dark terminal backgrounds.
    pub fn dark() -> Self {
        Self::new("Dark", true, ThemeColors::dark())
    }

    /// Returns the default light theme.
    ///
    /// This theme uses colors suitable for light terminal backgrounds.
    pub fn light() -> Self {
        Self::new("Light", false, ThemeColors::light())
    }

    /// Returns the Dracula theme.
    ///
    /// A popular dark theme with purple accents.
    /// <https://draculatheme.com>
    pub fn dracula() -> Self {
        Self::new("Dracula", true, ThemeColors::dracula())
    }

    /// Returns the Nord theme.
    ///
    /// An arctic, north-bluish color palette.
    /// <https://www.nordtheme.com>
    pub fn nord() -> Self {
        Self::new("Nord", true, ThemeColors::nord())
    }

    /// Returns a Catppuccin theme for the requested flavor.
    ///
    /// A soothing pastel theme with warm tones.
    /// <https://catppuccin.com>
    pub fn catppuccin(flavor: CatppuccinFlavor) -> Self {
        match flavor {
            CatppuccinFlavor::Latte => {
                Self::new("Catppuccin Latte", false, ThemeColors::catppuccin_latte())
            }
            CatppuccinFlavor::Frappe => {
                Self::new("Catppuccin Frappe", true, ThemeColors::catppuccin_frappe())
            }
            CatppuccinFlavor::Macchiato => Self::new(
                "Catppuccin Macchiato",
                true,
                ThemeColors::catppuccin_macchiato(),
            ),
            CatppuccinFlavor::Mocha => {
                Self::new("Catppuccin Mocha", true, ThemeColors::catppuccin_mocha())
            }
        }
    }

    /// Returns the Catppuccin Latte theme.
    pub fn catppuccin_latte() -> Self {
        Self::catppuccin(CatppuccinFlavor::Latte)
    }

    /// Returns the Catppuccin Frappe theme.
    pub fn catppuccin_frappe() -> Self {
        Self::catppuccin(CatppuccinFlavor::Frappe)
    }

    /// Returns the Catppuccin Macchiato theme.
    pub fn catppuccin_macchiato() -> Self {
        Self::catppuccin(CatppuccinFlavor::Macchiato)
    }

    /// Returns the Catppuccin Mocha theme.
    pub fn catppuccin_mocha() -> Self {
        Self::catppuccin(CatppuccinFlavor::Mocha)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemePreset {
    /// Convert this preset into a concrete theme instance.
    pub fn to_theme(&self) -> Theme {
        let theme = match *self {
            ThemePreset::Dark => Theme::dark(),
            ThemePreset::Light => Theme::light(),
            ThemePreset::Dracula => Theme::dracula(),
            ThemePreset::Nord => Theme::nord(),
            ThemePreset::Catppuccin(flavor) => Theme::catppuccin(flavor),
        };

        info!(theme.preset = %self, theme.name = %theme.name(), "Loaded theme preset");
        theme
    }
}

impl ThemeColors {
    /// Returns the color for the given slot.
    pub fn get(&self, slot: ColorSlot) -> &Color {
        let color = match slot {
            ColorSlot::Primary => &self.primary,
            ColorSlot::Secondary => &self.secondary,
            ColorSlot::Accent => &self.accent,
            ColorSlot::Background => &self.background,
            ColorSlot::Foreground | ColorSlot::Text => &self.text,
            ColorSlot::TextMuted => &self.text_muted,
            ColorSlot::TextDisabled => &self.text_disabled,
            ColorSlot::Surface => &self.surface,
            ColorSlot::SurfaceAlt => &self.surface_alt,
            ColorSlot::Success => &self.success,
            ColorSlot::Warning => &self.warning,
            ColorSlot::Error => &self.error,
            ColorSlot::Info => &self.info,
            ColorSlot::Border => &self.border,
            ColorSlot::BorderMuted => &self.border_muted,
            ColorSlot::Separator => &self.separator,
            ColorSlot::Focus => &self.focus,
            ColorSlot::Selection => &self.selection,
            ColorSlot::Hover => &self.hover,
            ColorSlot::CodeKeyword => &self.code_keyword,
            ColorSlot::CodeString => &self.code_string,
            ColorSlot::CodeNumber => &self.code_number,
            ColorSlot::CodeComment => &self.code_comment,
            ColorSlot::CodeFunction => &self.code_function,
            ColorSlot::CodeType => &self.code_type,
            ColorSlot::CodeVariable => &self.code_variable,
            ColorSlot::CodeOperator => &self.code_operator,
        };

        debug!(theme.slot = ?slot, theme.value = %color.0, "Theme color lookup");
        color
    }

    /// Returns the custom color slots.
    pub fn custom(&self) -> &HashMap<String, Color> {
        &self.custom
    }

    /// Returns a mutable reference to the custom color slots.
    pub fn custom_mut(&mut self) -> &mut HashMap<String, Color> {
        &mut self.custom
    }

    /// Returns a custom color by name.
    pub fn get_custom(&self, name: &str) -> Option<&Color> {
        self.custom.get(name)
    }

    /// Validate that all color slots are usable.
    ///
    /// # Errors
    /// Returns `ThemeValidationError` if any color slot is empty or invalid.
    pub fn validate(&self) -> Result<(), ThemeValidationError> {
        fn validate_color(slot: &'static str, color: &Color) -> Result<(), ThemeValidationError> {
            if color.0.trim().is_empty() {
                return Err(ThemeValidationError::EmptyColor(slot));
            }
            if !color.is_valid() {
                return Err(ThemeValidationError::InvalidColor {
                    slot,
                    value: color.0.clone(),
                });
            }
            Ok(())
        }

        validate_color("primary", &self.primary)?;
        validate_color("secondary", &self.secondary)?;
        validate_color("accent", &self.accent)?;
        validate_color("background", &self.background)?;
        validate_color("surface", &self.surface)?;
        validate_color("surface_alt", &self.surface_alt)?;
        validate_color("text", &self.text)?;
        validate_color("text_muted", &self.text_muted)?;
        validate_color("text_disabled", &self.text_disabled)?;
        validate_color("success", &self.success)?;
        validate_color("warning", &self.warning)?;
        validate_color("error", &self.error)?;
        validate_color("info", &self.info)?;
        validate_color("border", &self.border)?;
        validate_color("border_muted", &self.border_muted)?;
        validate_color("separator", &self.separator)?;
        validate_color("focus", &self.focus)?;
        validate_color("selection", &self.selection)?;
        validate_color("hover", &self.hover)?;
        validate_color("code_keyword", &self.code_keyword)?;
        validate_color("code_string", &self.code_string)?;
        validate_color("code_number", &self.code_number)?;
        validate_color("code_comment", &self.code_comment)?;
        validate_color("code_function", &self.code_function)?;
        validate_color("code_type", &self.code_type)?;
        validate_color("code_variable", &self.code_variable)?;
        validate_color("code_operator", &self.code_operator)?;

        for (name, color) in &self.custom {
            if name.trim().is_empty() {
                return Err(ThemeValidationError::InvalidCustomName);
            }
            if !color.is_valid() {
                return Err(ThemeValidationError::InvalidCustomColor {
                    name: name.clone(),
                    value: color.0.clone(),
                });
            }
        }

        Ok(())
    }

    /// Creates a new `ThemeColors` with all slots set to the same color.
    ///
    /// Useful as a starting point for building custom themes.
    pub fn uniform(color: impl Into<Color>) -> Self {
        let c = color.into();
        Self {
            primary: c.clone(),
            secondary: c.clone(),
            accent: c.clone(),
            background: c.clone(),
            surface: c.clone(),
            surface_alt: c.clone(),
            text: c.clone(),
            text_muted: c.clone(),
            text_disabled: c.clone(),
            success: c.clone(),
            warning: c.clone(),
            error: c.clone(),
            info: c.clone(),
            border: c.clone(),
            border_muted: c.clone(),
            separator: c.clone(),
            focus: c.clone(),
            selection: c.clone(),
            hover: c.clone(),
            code_keyword: c.clone(),
            code_string: c.clone(),
            code_number: c.clone(),
            code_comment: c.clone(),
            code_function: c.clone(),
            code_type: c.clone(),
            code_variable: c.clone(),
            code_operator: c,
            custom: HashMap::new(),
        }
    }

    /// Returns the default dark color palette.
    pub fn dark() -> Self {
        Self {
            // Primary palette
            primary: Color::from("#7c3aed"),   // Violet
            secondary: Color::from("#6366f1"), // Indigo
            accent: Color::from("#22d3ee"),    // Cyan

            // Backgrounds
            background: Color::from("#0f0f0f"),  // Near black
            surface: Color::from("#1a1a1a"),     // Dark gray
            surface_alt: Color::from("#262626"), // Slightly lighter

            // Text
            text: Color::from("#fafafa"),          // Near white
            text_muted: Color::from("#a1a1aa"),    // Gray
            text_disabled: Color::from("#52525b"), // Darker gray

            // Semantic
            success: Color::from("#22c55e"), // Green
            warning: Color::from("#f59e0b"), // Amber
            error: Color::from("#ef4444"),   // Red
            info: Color::from("#3b82f6"),    // Blue

            // UI elements
            border: Color::from("#3f3f46"),       // Zinc-700
            border_muted: Color::from("#27272a"), // Zinc-800
            separator: Color::from("#27272a"),    // Same as border_muted

            // Interactive
            focus: Color::from("#7c3aed"),     // Same as primary
            selection: Color::from("#4c1d95"), // Dark violet
            hover: Color::from("#27272a"),     // Subtle highlight

            // Code/syntax (based on popular dark themes)
            code_keyword: Color::from("#c678dd"),  // Purple
            code_string: Color::from("#98c379"),   // Green
            code_number: Color::from("#d19a66"),   // Orange
            code_comment: Color::from("#5c6370"),  // Gray
            code_function: Color::from("#61afef"), // Blue
            code_type: Color::from("#e5c07b"),     // Yellow
            code_variable: Color::from("#e06c75"), // Red/pink
            code_operator: Color::from("#56b6c2"), // Cyan
            custom: HashMap::new(),
        }
    }

    /// Returns the default light color palette.
    pub fn light() -> Self {
        Self {
            // Primary palette
            primary: Color::from("#7c3aed"),   // Violet
            secondary: Color::from("#4f46e5"), // Indigo
            accent: Color::from("#0891b2"),    // Cyan (darker for light bg)

            // Backgrounds
            background: Color::from("#ffffff"),  // White
            surface: Color::from("#f4f4f5"),     // Zinc-100
            surface_alt: Color::from("#e4e4e7"), // Zinc-200

            // Text
            text: Color::from("#18181b"),          // Zinc-900
            text_muted: Color::from("#71717a"),    // Zinc-500
            text_disabled: Color::from("#a1a1aa"), // Zinc-400

            // Semantic
            success: Color::from("#16a34a"), // Green-600
            warning: Color::from("#d97706"), // Amber-600
            error: Color::from("#dc2626"),   // Red-600
            info: Color::from("#2563eb"),    // Blue-600

            // UI elements
            border: Color::from("#d4d4d8"),       // Zinc-300
            border_muted: Color::from("#e4e4e7"), // Zinc-200
            separator: Color::from("#e4e4e7"),    // Same as border_muted

            // Interactive
            focus: Color::from("#7c3aed"),     // Same as primary
            selection: Color::from("#ddd6fe"), // Light violet
            hover: Color::from("#f4f4f5"),     // Subtle highlight

            // Code/syntax (based on popular light themes)
            code_keyword: Color::from("#a626a4"),  // Purple
            code_string: Color::from("#50a14f"),   // Green
            code_number: Color::from("#986801"),   // Orange/brown
            code_comment: Color::from("#a0a1a7"),  // Gray
            code_function: Color::from("#4078f2"), // Blue
            code_type: Color::from("#c18401"),     // Yellow/gold
            code_variable: Color::from("#e45649"), // Red
            code_operator: Color::from("#0184bc"), // Cyan
            custom: HashMap::new(),
        }
    }

    /// Returns the Dracula color palette.
    pub fn dracula() -> Self {
        // Dracula theme colors from https://draculatheme.com
        Self {
            primary: Color::from("#bd93f9"),   // Purple
            secondary: Color::from("#ff79c6"), // Pink
            accent: Color::from("#8be9fd"),    // Cyan

            background: Color::from("#282a36"),  // Background
            surface: Color::from("#44475a"),     // Current Line
            surface_alt: Color::from("#6272a4"), // Comment

            text: Color::from("#f8f8f2"),          // Foreground
            text_muted: Color::from("#6272a4"),    // Comment
            text_disabled: Color::from("#44475a"), // Current Line

            success: Color::from("#50fa7b"), // Green
            warning: Color::from("#ffb86c"), // Orange
            error: Color::from("#ff5555"),   // Red
            info: Color::from("#8be9fd"),    // Cyan

            border: Color::from("#44475a"),       // Current Line
            border_muted: Color::from("#282a36"), // Background
            separator: Color::from("#44475a"),    // Current Line

            focus: Color::from("#bd93f9"),     // Purple
            selection: Color::from("#44475a"), // Current Line
            hover: Color::from("#44475a"),     // Current Line

            code_keyword: Color::from("#ff79c6"),  // Pink
            code_string: Color::from("#f1fa8c"),   // Yellow
            code_number: Color::from("#bd93f9"),   // Purple
            code_comment: Color::from("#6272a4"),  // Comment
            code_function: Color::from("#50fa7b"), // Green
            code_type: Color::from("#8be9fd"),     // Cyan
            code_variable: Color::from("#f8f8f2"), // Foreground
            code_operator: Color::from("#ff79c6"), // Pink
            custom: HashMap::new(),
        }
    }

    /// Returns the Nord color palette.
    pub fn nord() -> Self {
        // Nord theme colors from https://www.nordtheme.com
        Self {
            primary: Color::from("#88c0d0"),   // Nord8 (cyan)
            secondary: Color::from("#81a1c1"), // Nord9 (blue)
            accent: Color::from("#b48ead"),    // Nord15 (purple)

            background: Color::from("#2e3440"),  // Nord0
            surface: Color::from("#3b4252"),     // Nord1
            surface_alt: Color::from("#434c5e"), // Nord2

            text: Color::from("#eceff4"),          // Nord6
            text_muted: Color::from("#d8dee9"),    // Nord4
            text_disabled: Color::from("#4c566a"), // Nord3

            success: Color::from("#a3be8c"), // Nord14 (green)
            warning: Color::from("#ebcb8b"), // Nord13 (yellow)
            error: Color::from("#bf616a"),   // Nord11 (red)
            info: Color::from("#5e81ac"),    // Nord10 (blue)

            border: Color::from("#4c566a"),       // Nord3
            border_muted: Color::from("#3b4252"), // Nord1
            separator: Color::from("#3b4252"),    // Nord1

            focus: Color::from("#88c0d0"),     // Nord8
            selection: Color::from("#434c5e"), // Nord2
            hover: Color::from("#3b4252"),     // Nord1

            code_keyword: Color::from("#81a1c1"),  // Nord9
            code_string: Color::from("#a3be8c"),   // Nord14
            code_number: Color::from("#b48ead"),   // Nord15
            code_comment: Color::from("#616e88"),  // Muted Nord
            code_function: Color::from("#88c0d0"), // Nord8
            code_type: Color::from("#8fbcbb"),     // Nord7
            code_variable: Color::from("#d8dee9"), // Nord4
            code_operator: Color::from("#81a1c1"), // Nord9
            custom: HashMap::new(),
        }
    }

    /// Returns the Catppuccin Mocha color palette.
    pub fn catppuccin_mocha() -> Self {
        // Catppuccin Mocha colors from https://catppuccin.com/palette
        Self {
            primary: Color::from("#cba6f7"),   // Mauve
            secondary: Color::from("#89b4fa"), // Blue
            accent: Color::from("#f5c2e7"),    // Pink

            background: Color::from("#1e1e2e"),  // Base
            surface: Color::from("#313244"),     // Surface0
            surface_alt: Color::from("#45475a"), // Surface1

            text: Color::from("#cdd6f4"),          // Text
            text_muted: Color::from("#a6adc8"),    // Subtext0
            text_disabled: Color::from("#6c7086"), // Overlay0

            success: Color::from("#a6e3a1"), // Green
            warning: Color::from("#f9e2af"), // Yellow
            error: Color::from("#f38ba8"),   // Red
            info: Color::from("#89dceb"),    // Sky

            border: Color::from("#45475a"),       // Surface1
            border_muted: Color::from("#313244"), // Surface0
            separator: Color::from("#313244"),    // Surface0

            focus: Color::from("#cba6f7"),     // Mauve
            selection: Color::from("#45475a"), // Surface1
            hover: Color::from("#313244"),     // Surface0

            code_keyword: Color::from("#cba6f7"),  // Mauve
            code_string: Color::from("#a6e3a1"),   // Green
            code_number: Color::from("#fab387"),   // Peach
            code_comment: Color::from("#6c7086"),  // Overlay0
            code_function: Color::from("#89b4fa"), // Blue
            code_type: Color::from("#f9e2af"),     // Yellow
            code_variable: Color::from("#f5c2e7"), // Pink
            code_operator: Color::from("#89dceb"), // Sky
            custom: HashMap::new(),
        }
    }

    /// Returns the Catppuccin Latte color palette.
    pub fn catppuccin_latte() -> Self {
        // Catppuccin Latte colors from https://catppuccin.com/palette
        Self {
            primary: Color::from("#8839ef"),   // Mauve
            secondary: Color::from("#1e66f5"), // Blue
            accent: Color::from("#ea76cb"),    // Pink

            background: Color::from("#eff1f5"),  // Base
            surface: Color::from("#ccd0da"),     // Surface0
            surface_alt: Color::from("#bcc0cc"), // Surface1

            text: Color::from("#4c4f69"),          // Text
            text_muted: Color::from("#6c6f85"),    // Subtext0
            text_disabled: Color::from("#9ca0b0"), // Overlay0

            success: Color::from("#40a02b"), // Green
            warning: Color::from("#df8e1d"), // Yellow
            error: Color::from("#d20f39"),   // Red
            info: Color::from("#04a5e5"),    // Sky

            border: Color::from("#bcc0cc"),       // Surface1
            border_muted: Color::from("#ccd0da"), // Surface0
            separator: Color::from("#ccd0da"),    // Surface0

            focus: Color::from("#8839ef"),     // Mauve
            selection: Color::from("#bcc0cc"), // Surface1
            hover: Color::from("#ccd0da"),     // Surface0

            code_keyword: Color::from("#8839ef"),  // Mauve
            code_string: Color::from("#40a02b"),   // Green
            code_number: Color::from("#fe640b"),   // Peach
            code_comment: Color::from("#9ca0b0"),  // Overlay0
            code_function: Color::from("#1e66f5"), // Blue
            code_type: Color::from("#df8e1d"),     // Yellow
            code_variable: Color::from("#ea76cb"), // Pink
            code_operator: Color::from("#04a5e5"), // Sky
            custom: HashMap::new(),
        }
    }

    /// Returns the Catppuccin Frappe color palette.
    pub fn catppuccin_frappe() -> Self {
        // Catppuccin Frappe colors from https://catppuccin.com/palette
        Self {
            primary: Color::from("#ca9ee6"),   // Mauve
            secondary: Color::from("#8caaee"), // Blue
            accent: Color::from("#f4b8e4"),    // Pink

            background: Color::from("#303446"),  // Base
            surface: Color::from("#414559"),     // Surface0
            surface_alt: Color::from("#51576d"), // Surface1

            text: Color::from("#c6d0f5"),          // Text
            text_muted: Color::from("#a5adce"),    // Subtext0
            text_disabled: Color::from("#737994"), // Overlay0

            success: Color::from("#a6d189"), // Green
            warning: Color::from("#e5c890"), // Yellow
            error: Color::from("#e78284"),   // Red
            info: Color::from("#99d1db"),    // Sky

            border: Color::from("#51576d"),       // Surface1
            border_muted: Color::from("#414559"), // Surface0
            separator: Color::from("#414559"),    // Surface0

            focus: Color::from("#ca9ee6"),     // Mauve
            selection: Color::from("#51576d"), // Surface1
            hover: Color::from("#414559"),     // Surface0

            code_keyword: Color::from("#ca9ee6"),  // Mauve
            code_string: Color::from("#a6d189"),   // Green
            code_number: Color::from("#ef9f76"),   // Peach
            code_comment: Color::from("#737994"),  // Overlay0
            code_function: Color::from("#8caaee"), // Blue
            code_type: Color::from("#e5c890"),     // Yellow
            code_variable: Color::from("#f4b8e4"), // Pink
            code_operator: Color::from("#99d1db"), // Sky
            custom: HashMap::new(),
        }
    }

    /// Returns the Catppuccin Macchiato color palette.
    pub fn catppuccin_macchiato() -> Self {
        // Catppuccin Macchiato colors from https://catppuccin.com/palette
        Self {
            primary: Color::from("#c6a0f6"),   // Mauve
            secondary: Color::from("#8aadf4"), // Blue
            accent: Color::from("#f5bde6"),    // Pink

            background: Color::from("#24273a"),  // Base
            surface: Color::from("#363a4f"),     // Surface0
            surface_alt: Color::from("#494d64"), // Surface1

            text: Color::from("#cad3f5"),          // Text
            text_muted: Color::from("#a5adcb"),    // Subtext0
            text_disabled: Color::from("#6e738d"), // Overlay0

            success: Color::from("#a6da95"), // Green
            warning: Color::from("#eed49f"), // Yellow
            error: Color::from("#ed8796"),   // Red
            info: Color::from("#91d7e3"),    // Sky

            border: Color::from("#494d64"),       // Surface1
            border_muted: Color::from("#363a4f"), // Surface0
            separator: Color::from("#363a4f"),    // Surface0

            focus: Color::from("#c6a0f6"),     // Mauve
            selection: Color::from("#494d64"), // Surface1
            hover: Color::from("#363a4f"),     // Surface0

            code_keyword: Color::from("#c6a0f6"),  // Mauve
            code_string: Color::from("#a6da95"),   // Green
            code_number: Color::from("#f5a97f"),   // Peach
            code_comment: Color::from("#6e738d"),  // Overlay0
            code_function: Color::from("#8aadf4"), // Blue
            code_type: Color::from("#eed49f"),     // Yellow
            code_variable: Color::from("#f5bde6"), // Pink
            code_operator: Color::from("#91d7e3"), // Sky
            custom: HashMap::new(),
        }
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark()
    }
}

/// Creates an adaptive color from a theme's light and dark colors.
///
/// This is useful for creating colors that work correctly in both
/// light and dark terminal environments.
pub fn adaptive(
    light: &ThemeColors,
    dark: &ThemeColors,
    slot: impl Fn(&ThemeColors) -> &Color,
) -> AdaptiveColor {
    AdaptiveColor {
        light: slot(light).clone(),
        dark: slot(dark).clone(),
    }
}

/// Error validating theme colors.
#[derive(Error, Debug)]
pub enum ThemeValidationError {
    #[error("Color slot '{0}' is empty")]
    EmptyColor(&'static str),
    #[error("Invalid color value '{value}' for slot '{slot}'")]
    InvalidColor { slot: &'static str, value: String },
    #[error("Custom color name cannot be empty")]
    InvalidCustomName,
    #[error("Invalid custom color '{value}' for '{name}'")]
    InvalidCustomColor { name: String, value: String },
}

/// Error loading a theme.
#[derive(Error, Debug)]
pub enum ThemeLoadError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[cfg(feature = "yaml")]
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Validation error: {0}")]
    Validation(#[from] ThemeValidationError),
}

/// Error saving a theme.
#[derive(Error, Debug)]
pub enum ThemeSaveError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::ser::Error),
    #[cfg(feature = "yaml")]
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::Renderer;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_theme_dark_default() {
        let theme = Theme::dark();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Dark");
    }

    #[test]
    fn test_theme_light_default() {
        let theme = Theme::light();
        assert!(!theme.is_dark());
        assert_eq!(theme.name(), "Light");
    }

    #[test]
    fn test_theme_dracula() {
        let theme = Theme::dracula();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Dracula");
        // Dracula's background is #282a36
        assert_eq!(theme.colors().background.0, "#282a36");
    }

    #[test]
    fn test_theme_nord() {
        let theme = Theme::nord();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Nord");
        // Nord's background is #2e3440
        assert_eq!(theme.colors().background.0, "#2e3440");
    }

    #[test]
    fn test_theme_catppuccin() {
        let theme = Theme::catppuccin_mocha();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Catppuccin Mocha");
        // Catppuccin Mocha's background is #1e1e2e
        assert_eq!(theme.colors().background.0, "#1e1e2e");
    }

    #[test]
    fn test_theme_catppuccin_latte() {
        let theme = Theme::catppuccin_latte();
        assert!(!theme.is_dark());
        assert_eq!(theme.name(), "Catppuccin Latte");
        assert_eq!(theme.colors().background.0, "#eff1f5");
        assert_eq!(theme.colors().primary.0, "#8839ef");
    }

    #[test]
    fn test_theme_catppuccin_frappe() {
        let theme = Theme::catppuccin_frappe();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Catppuccin Frappe");
        assert_eq!(theme.colors().background.0, "#303446");
        assert_eq!(theme.colors().primary.0, "#ca9ee6");
    }

    #[test]
    fn test_theme_catppuccin_macchiato() {
        let theme = Theme::catppuccin_macchiato();
        assert!(theme.is_dark());
        assert_eq!(theme.name(), "Catppuccin Macchiato");
        assert_eq!(theme.colors().background.0, "#24273a");
        assert_eq!(theme.colors().primary.0, "#c6a0f6");
    }

    #[test]
    fn test_theme_preset_to_theme() {
        let theme = ThemePreset::Catppuccin(CatppuccinFlavor::Latte).to_theme();
        assert_eq!(theme.name(), "Catppuccin Latte");
        assert!(!theme.is_dark());
    }

    #[test]
    fn test_theme_contrast_aa() {
        let theme = Theme::dark();
        assert!(theme.check_contrast_aa(ColorSlot::Foreground, ColorSlot::Background));
    }

    #[test]
    fn test_theme_context_switch() {
        let ctx = ThemeContext::from_preset(ThemePreset::Dark);
        assert_eq!(ctx.current().name(), "Dark");
        ctx.set_preset(ThemePreset::Light);
        assert_eq!(ctx.current().name(), "Light");
    }

    #[test]
    fn test_theme_context_listener() {
        let ctx = ThemeContext::from_preset(ThemePreset::Dark);
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_ref = Arc::clone(&hits);
        let id = ctx.on_change(move |_theme| {
            hits_ref.fetch_add(1, Ordering::SeqCst);
        });

        ctx.set_preset(ThemePreset::Light);
        assert_eq!(hits.load(Ordering::SeqCst), 1);

        ctx.remove_listener(id);
        ctx.set_preset(ThemePreset::Dark);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_theme_context_thread_safe() {
        use std::thread;

        let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dark));
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let ctx = Arc::clone(&ctx);
                thread::spawn(move || {
                    if i % 2 == 0 {
                        ctx.set_preset(ThemePreset::Light);
                    } else {
                        ctx.set_preset(ThemePreset::Dark);
                    }
                    let _current = ctx.current();
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread join");
        }
    }

    #[test]
    fn test_theme_builder() {
        let theme = Theme::dark().with_name("Custom Dark").with_dark(true);
        assert_eq!(theme.name(), "Custom Dark");
        assert!(theme.is_dark());
    }

    #[test]
    fn test_theme_colors_uniform() {
        let colors = ThemeColors::uniform("#ff0000");
        assert_eq!(colors.primary.0, "#ff0000");
        assert_eq!(colors.background.0, "#ff0000");
        assert_eq!(colors.text.0, "#ff0000");
    }

    #[test]
    fn test_adaptive_color() {
        let light = ThemeColors::light();
        let dark = ThemeColors::dark();

        let adaptive_text = adaptive(&light, &dark, |c| &c.text);

        // Light theme text is dark, dark theme text is light
        assert_eq!(adaptive_text.light.0, light.text.0);
        assert_eq!(adaptive_text.dark.0, dark.text.0);
    }

    #[test]
    fn test_theme_style() {
        let theme = Theme::dark();
        let style = theme.style();
        // Style should be empty/default
        assert!(style.value().is_empty());
    }

    #[test]
    fn test_theme_get_slot() {
        let theme = Theme::dark();
        assert_eq!(theme.get(ColorSlot::Primary).0, theme.colors().primary.0);
        assert_eq!(
            theme.get(ColorSlot::TextMuted).0,
            theme.colors().text_muted.0
        );
        assert_eq!(theme.get(ColorSlot::Foreground).0, theme.colors().text.0);
        assert_eq!(theme.get(ColorSlot::Text).0, theme.colors().text.0);
    }

    #[test]
    fn test_theme_json_roundtrip() {
        let theme = Theme::dark()
            .with_description("A dark theme")
            .with_author("charmed_rust");
        let json = theme.to_json().expect("serialize theme");
        let loaded = Theme::from_json(&json).expect("deserialize theme");
        assert_eq!(loaded.colors().primary.0, theme.colors().primary.0);
        assert_eq!(loaded.description(), Some("A dark theme"));
        assert_eq!(loaded.author(), Some("charmed_rust"));
        assert!(loaded.is_dark());
    }

    #[test]
    fn test_theme_toml_roundtrip() {
        let theme = Theme::dark().with_description("TOML theme");
        let toml = theme.to_toml().expect("serialize theme to toml");
        let loaded = Theme::from_toml(&toml).expect("deserialize theme from toml");
        assert_eq!(loaded.colors().primary.0, theme.colors().primary.0);
        assert_eq!(loaded.description(), Some("TOML theme"));
        assert!(loaded.is_dark());
    }

    #[test]
    fn test_theme_custom_colors_serde() {
        let mut theme = Theme::dark();
        theme
            .colors_mut()
            .custom_mut()
            .insert("brand".to_string(), Color::from("#123456"));
        let json = theme.to_json().expect("serialize theme");
        let loaded = Theme::from_json(&json).expect("deserialize theme");
        assert_eq!(
            loaded.colors().get_custom("brand").expect("custom color"),
            &Color::from("#123456")
        );
    }

    #[test]
    fn test_color_slots_all_defined() {
        // Ensure all themes have all color slots defined (not empty)
        for theme in [
            Theme::dark(),
            Theme::light(),
            Theme::dracula(),
            Theme::nord(),
            Theme::catppuccin_mocha(),
            Theme::catppuccin_latte(),
            Theme::catppuccin_frappe(),
            Theme::catppuccin_macchiato(),
        ] {
            let c = theme.colors();

            // All colors should have non-empty values
            assert!(!c.primary.0.is_empty(), "{}: primary empty", theme.name());
            assert!(
                !c.secondary.0.is_empty(),
                "{}: secondary empty",
                theme.name()
            );
            assert!(!c.accent.0.is_empty(), "{}: accent empty", theme.name());
            assert!(
                !c.background.0.is_empty(),
                "{}: background empty",
                theme.name()
            );
            assert!(!c.surface.0.is_empty(), "{}: surface empty", theme.name());
            assert!(!c.text.0.is_empty(), "{}: text empty", theme.name());
            assert!(!c.error.0.is_empty(), "{}: error empty", theme.name());
        }
    }

    #[test]
    fn test_color_transform_lighten_darken() {
        let black = Color::from("#000000");
        let lighter = ColorTransform::Lighten(0.2).apply(black);
        assert_eq!(lighter.0, "#333333");

        let white = Color::from("#ffffff");
        let darker = ColorTransform::Darken(0.2).apply(white);
        assert_eq!(darker.0, "#cccccc");
    }

    #[test]
    fn test_color_transform_desaturate_and_alpha() {
        let red = Color::from("#ff0000");
        let gray = ColorTransform::Desaturate(1.0).apply(red);
        assert_eq!(gray.0, "#808080");

        let white = Color::from("#ffffff");
        let alpha = ColorTransform::Alpha(0.5).apply(white);
        assert_eq!(alpha.0, "#808080");
    }

    #[test]
    fn test_cached_themed_style_invalidation() {
        let ctx = Arc::new(ThemeContext::from_preset(ThemePreset::Dark));
        let themed = ThemedStyle::new(Arc::clone(&ctx))
            .background(ColorSlot::Background)
            .renderer(Arc::new(Renderer::DEFAULT));
        let cached = CachedThemedStyle::new(themed);

        let first = cached.render("x");
        ctx.set_preset(ThemePreset::Light);
        let second = cached.render("x");

        assert_ne!(first, second);
    }
}
