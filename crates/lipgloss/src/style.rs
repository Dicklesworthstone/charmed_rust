//! Style definition and builder.
//!
//! The [`Style`] struct is the core of lipgloss, providing a fluent API
//! for building terminal styles.
//!
//! # Example
//!
//! ```rust
//! use lipgloss::{Style, Color};
//!
//! let style = Style::new()
//!     .bold()
//!     .foreground("#ff0000")
//!     .padding(1);
//!
//! println!("{}", style.render("Hello!"));
//! ```

use bitflags::bitflags;
use std::sync::Arc;

use crate::border::{Border, BorderEdges};
use crate::color::{Color, ColorProfile, NoColor, TerminalColor};
use crate::position::{Position, Sides};
use crate::renderer::Renderer;
use crate::theme::{ColorSlot, Theme, ThemeRole};

bitflags! {
    /// Flags indicating which style properties are explicitly set.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Props: u64 {
        // Boolean attributes
        const BOLD = 1 << 0;
        const ITALIC = 1 << 1;
        const UNDERLINE = 1 << 2;
        const STRIKETHROUGH = 1 << 3;
        const REVERSE = 1 << 4;
        const BLINK = 1 << 5;
        const FAINT = 1 << 6;
        const UNDERLINE_SPACES = 1 << 7;
        const STRIKETHROUGH_SPACES = 1 << 8;
        const COLOR_WHITESPACE = 1 << 9;

        // Value properties
        const FOREGROUND = 1 << 10;
        const BACKGROUND = 1 << 11;
        const WIDTH = 1 << 12;
        const HEIGHT = 1 << 13;
        const ALIGN_HORIZONTAL = 1 << 14;
        const ALIGN_VERTICAL = 1 << 15;

        // Padding
        const PADDING_TOP = 1 << 16;
        const PADDING_RIGHT = 1 << 17;
        const PADDING_BOTTOM = 1 << 18;
        const PADDING_LEFT = 1 << 19;

        // Margin
        const MARGIN_TOP = 1 << 20;
        const MARGIN_RIGHT = 1 << 21;
        const MARGIN_BOTTOM = 1 << 22;
        const MARGIN_LEFT = 1 << 23;
        const MARGIN_BACKGROUND = 1 << 24;

        // Border
        const BORDER_STYLE = 1 << 25;
        const BORDER_TOP = 1 << 26;
        const BORDER_RIGHT = 1 << 27;
        const BORDER_BOTTOM = 1 << 28;
        const BORDER_LEFT = 1 << 29;

        const BORDER_TOP_FG = 1 << 30;
        const BORDER_RIGHT_FG = 1 << 31;
        const BORDER_BOTTOM_FG = 1 << 32;
        const BORDER_LEFT_FG = 1 << 33;

        const BORDER_TOP_BG = 1 << 34;
        const BORDER_RIGHT_BG = 1 << 35;
        const BORDER_BOTTOM_BG = 1 << 36;
        const BORDER_LEFT_BG = 1 << 37;

        // Other
        const INLINE = 1 << 38;
        const MAX_WIDTH = 1 << 39;
        const MAX_HEIGHT = 1 << 40;
        const TAB_WIDTH = 1 << 41;
        const TRANSFORM = 1 << 42;
    }
}

bitflags! {
    /// Boolean attribute values.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Attrs: u16 {
        const BOLD = 1 << 0;
        const ITALIC = 1 << 1;
        const UNDERLINE = 1 << 2;
        const STRIKETHROUGH = 1 << 3;
        const REVERSE = 1 << 4;
        const BLINK = 1 << 5;
        const FAINT = 1 << 6;
        const UNDERLINE_SPACES = 1 << 7;
        const STRIKETHROUGH_SPACES = 1 << 8;
        const COLOR_WHITESPACE = 1 << 9;
        const INLINE = 1 << 10;
    }
}

/// Type alias for transform functions.
pub type TransformFn = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// A terminal style definition.
#[derive(Clone, Default)]
pub struct Style {
    /// Which properties are set.
    props: Props,
    /// Boolean attribute values.
    attrs: Attrs,

    /// Foreground color.
    fg_color: Option<Box<dyn TerminalColor>>,
    /// Background color.
    bg_color: Option<Box<dyn TerminalColor>>,

    /// Fixed width.
    width: u16,
    /// Fixed height.
    height: u16,
    /// Maximum width.
    max_width: u16,
    /// Maximum height.
    max_height: u16,

    /// Horizontal alignment.
    align_horizontal: Position,
    /// Vertical alignment.
    align_vertical: Position,

    /// Padding (inner spacing).
    padding: Sides<u16>,
    /// Margin (outer spacing).
    margin: Sides<u16>,
    /// Margin background color.
    margin_bg_color: Option<Box<dyn TerminalColor>>,

    /// Border style.
    border_style: Border,
    /// Which border edges to render.
    border_edges: BorderEdges,
    /// Border foreground colors per side.
    border_fg: [Option<Box<dyn TerminalColor>>; 4],
    /// Border background colors per side.
    border_bg: [Option<Box<dyn TerminalColor>>; 4],

    /// Tab width (-1 = no conversion, 0 = remove, >0 = spaces).
    tab_width: i8,

    /// Text transform function.
    transform: Option<TransformFn>,

    /// Underlying string value (for Display impl).
    value: String,

    /// Renderer reference.
    renderer: Option<Arc<Renderer>>,
}

impl std::fmt::Debug for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Style")
            .field("props", &self.props)
            .field("attrs", &self.attrs)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Style {
    /// Creates a new empty style.
    pub fn new() -> Self {
        Self::default()
    }

    // ==================== Theme Constructors ====================

    /// Create a style with foreground color from a theme slot.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{ColorSlot, Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::from_theme(&theme, ColorSlot::Primary);
    /// ```
    pub fn from_theme(theme: &Theme, slot: ColorSlot) -> Self {
        Style::new().foreground_color(theme.get(slot))
    }

    /// Create a style with foreground and background colors from theme slots.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{ColorSlot, Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::from_theme_colors(&theme, ColorSlot::Text, ColorSlot::Background);
    /// ```
    pub fn from_theme_colors(theme: &Theme, fg: ColorSlot, bg: ColorSlot) -> Self {
        Style::new()
            .foreground_color(theme.get(fg))
            .background_color(theme.get(bg))
    }

    /// Create a style from a semantic theme role.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{Style, Theme, ThemeRole};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::from_theme_role(&theme, ThemeRole::Muted);
    /// ```
    pub fn from_theme_role(theme: &Theme, role: ThemeRole) -> Self {
        match role {
            ThemeRole::Primary => Style::new().foreground_color(theme.get(ColorSlot::Primary)),
            ThemeRole::Success => Style::new().foreground_color(theme.get(ColorSlot::Success)),
            ThemeRole::Warning => Style::new().foreground_color(theme.get(ColorSlot::Warning)),
            ThemeRole::Error => Style::new().foreground_color(theme.get(ColorSlot::Error)),
            ThemeRole::Muted => Style::new().foreground_color(theme.get(ColorSlot::TextMuted)),
            ThemeRole::Inverted => Style::new()
                .foreground_color(theme.get(ColorSlot::Background))
                .background_color(theme.get(ColorSlot::Foreground)),
        }
    }

    /// Set the underlying string value for this style.
    pub fn set_string(mut self, s: impl Into<String>) -> Self {
        self.value = s.into();
        self
    }

    /// Get the underlying string value.
    pub fn value(&self) -> &str {
        &self.value
    }

    // ==================== Boolean Attributes ====================

    /// Enable bold text.
    pub fn bold(mut self) -> Self {
        self.props |= Props::BOLD;
        self.attrs |= Attrs::BOLD;
        self
    }

    /// Enable italic text.
    pub fn italic(mut self) -> Self {
        self.props |= Props::ITALIC;
        self.attrs |= Attrs::ITALIC;
        self
    }

    /// Enable underlined text.
    pub fn underline(mut self) -> Self {
        self.props |= Props::UNDERLINE;
        self.attrs |= Attrs::UNDERLINE;
        self
    }

    /// Enable strikethrough text.
    pub fn strikethrough(mut self) -> Self {
        self.props |= Props::STRIKETHROUGH;
        self.attrs |= Attrs::STRIKETHROUGH;
        self
    }

    /// Enable reverse video (swap fg/bg).
    pub fn reverse(mut self) -> Self {
        self.props |= Props::REVERSE;
        self.attrs |= Attrs::REVERSE;
        self
    }

    /// Enable blinking text.
    pub fn blink(mut self) -> Self {
        self.props |= Props::BLINK;
        self.attrs |= Attrs::BLINK;
        self
    }

    /// Enable faint/dim text.
    pub fn faint(mut self) -> Self {
        self.props |= Props::FAINT;
        self.attrs |= Attrs::FAINT;
        self
    }

    /// Set whether to underline spaces.
    pub fn underline_spaces(mut self, v: bool) -> Self {
        self.props |= Props::UNDERLINE_SPACES;
        if v {
            self.attrs |= Attrs::UNDERLINE_SPACES;
        } else {
            self.attrs.remove(Attrs::UNDERLINE_SPACES);
        }
        self
    }

    /// Set whether to strikethrough spaces.
    pub fn strikethrough_spaces(mut self, v: bool) -> Self {
        self.props |= Props::STRIKETHROUGH_SPACES;
        if v {
            self.attrs |= Attrs::STRIKETHROUGH_SPACES;
        } else {
            self.attrs.remove(Attrs::STRIKETHROUGH_SPACES);
        }
        self
    }

    // ==================== Colors ====================

    /// Set the foreground color.
    pub fn foreground(mut self, color: impl Into<String>) -> Self {
        self.props |= Props::FOREGROUND;
        self.fg_color = Some(Box::new(Color::new(color)));
        self
    }

    /// Set the foreground to a specific color type.
    pub fn foreground_color(mut self, color: impl TerminalColor + 'static) -> Self {
        self.props |= Props::FOREGROUND;
        self.fg_color = Some(Box::new(color));
        self
    }

    /// Set the foreground color using a theme slot.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{ColorSlot, Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::new().foreground_slot(&theme, ColorSlot::Text);
    /// ```
    pub fn foreground_slot(self, theme: &Theme, slot: ColorSlot) -> Self {
        self.foreground_color(theme.get(slot))
    }

    /// Remove the foreground color.
    pub fn no_foreground(mut self) -> Self {
        self.props |= Props::FOREGROUND;
        self.fg_color = Some(Box::new(NoColor));
        self
    }

    /// Set the background color.
    pub fn background(mut self, color: impl Into<String>) -> Self {
        self.props |= Props::BACKGROUND;
        self.bg_color = Some(Box::new(Color::new(color)));
        self
    }

    /// Set the background to a specific color type.
    pub fn background_color(mut self, color: impl TerminalColor + 'static) -> Self {
        self.props |= Props::BACKGROUND;
        self.bg_color = Some(Box::new(color));
        self
    }

    /// Set the background color using a theme slot.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{ColorSlot, Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::new().background_slot(&theme, ColorSlot::Surface);
    /// ```
    pub fn background_slot(self, theme: &Theme, slot: ColorSlot) -> Self {
        self.background_color(theme.get(slot))
    }

    /// Remove the background color.
    pub fn no_background(mut self) -> Self {
        self.props |= Props::BACKGROUND;
        self.bg_color = Some(Box::new(NoColor));
        self
    }

    // ==================== Dimensions ====================

    /// Set the fixed width.
    pub fn width(mut self, w: u16) -> Self {
        self.props |= Props::WIDTH;
        self.width = w;
        self
    }

    /// Set the fixed height.
    pub fn height(mut self, h: u16) -> Self {
        self.props |= Props::HEIGHT;
        self.height = h;
        self
    }

    /// Set the maximum width.
    pub fn max_width(mut self, w: u16) -> Self {
        self.props |= Props::MAX_WIDTH;
        self.max_width = w;
        self
    }

    /// Set the maximum height.
    pub fn max_height(mut self, h: u16) -> Self {
        self.props |= Props::MAX_HEIGHT;
        self.max_height = h;
        self
    }

    // ==================== Alignment ====================

    /// Set horizontal alignment.
    pub fn align(mut self, p: Position) -> Self {
        self.props |= Props::ALIGN_HORIZONTAL;
        self.align_horizontal = p;
        self
    }

    /// Set horizontal alignment.
    pub fn align_horizontal(mut self, p: Position) -> Self {
        self.props |= Props::ALIGN_HORIZONTAL;
        self.align_horizontal = p;
        self
    }

    /// Set vertical alignment.
    pub fn align_vertical(mut self, p: Position) -> Self {
        self.props |= Props::ALIGN_VERTICAL;
        self.align_vertical = p;
        self
    }

    // ==================== Padding ====================

    /// Set padding on all sides (CSS shorthand).
    pub fn padding(mut self, sides: impl Into<Sides<u16>>) -> Self {
        let s = sides.into();
        self.props |=
            Props::PADDING_TOP | Props::PADDING_RIGHT | Props::PADDING_BOTTOM | Props::PADDING_LEFT;
        self.padding = s;
        self
    }

    /// Set top padding.
    pub fn padding_top(mut self, n: u16) -> Self {
        self.props |= Props::PADDING_TOP;
        self.padding.top = n;
        self
    }

    /// Set right padding.
    pub fn padding_right(mut self, n: u16) -> Self {
        self.props |= Props::PADDING_RIGHT;
        self.padding.right = n;
        self
    }

    /// Set bottom padding.
    pub fn padding_bottom(mut self, n: u16) -> Self {
        self.props |= Props::PADDING_BOTTOM;
        self.padding.bottom = n;
        self
    }

    /// Set left padding.
    pub fn padding_left(mut self, n: u16) -> Self {
        self.props |= Props::PADDING_LEFT;
        self.padding.left = n;
        self
    }

    // ==================== Margin ====================

    /// Set margin on all sides (CSS shorthand).
    pub fn margin(mut self, sides: impl Into<Sides<u16>>) -> Self {
        let s = sides.into();
        self.props |=
            Props::MARGIN_TOP | Props::MARGIN_RIGHT | Props::MARGIN_BOTTOM | Props::MARGIN_LEFT;
        self.margin = s;
        self
    }

    /// Set top margin.
    pub fn margin_top(mut self, n: u16) -> Self {
        self.props |= Props::MARGIN_TOP;
        self.margin.top = n;
        self
    }

    /// Set right margin.
    pub fn margin_right(mut self, n: u16) -> Self {
        self.props |= Props::MARGIN_RIGHT;
        self.margin.right = n;
        self
    }

    /// Set bottom margin.
    pub fn margin_bottom(mut self, n: u16) -> Self {
        self.props |= Props::MARGIN_BOTTOM;
        self.margin.bottom = n;
        self
    }

    /// Set left margin.
    pub fn margin_left(mut self, n: u16) -> Self {
        self.props |= Props::MARGIN_LEFT;
        self.margin.left = n;
        self
    }

    /// Set margin background color.
    pub fn margin_background(mut self, color: impl Into<String>) -> Self {
        self.props |= Props::MARGIN_BACKGROUND;
        self.margin_bg_color = Some(Box::new(Color::new(color)));
        self
    }

    // ==================== Border ====================

    /// Set border style and optionally which sides to enable.
    pub fn border(mut self, border: Border) -> Self {
        self.props |= Props::BORDER_STYLE;
        self.border_style = border;
        self
    }

    /// Set border style.
    pub fn border_style(mut self, border: Border) -> Self {
        self.props |= Props::BORDER_STYLE;
        self.border_style = border;
        self
    }

    /// Enable or disable top border.
    pub fn border_top(mut self, v: bool) -> Self {
        self.props |= Props::BORDER_TOP;
        self.border_edges.top = v;
        self
    }

    /// Enable or disable right border.
    pub fn border_right(mut self, v: bool) -> Self {
        self.props |= Props::BORDER_RIGHT;
        self.border_edges.right = v;
        self
    }

    /// Enable or disable bottom border.
    pub fn border_bottom(mut self, v: bool) -> Self {
        self.props |= Props::BORDER_BOTTOM;
        self.border_edges.bottom = v;
        self
    }

    /// Enable or disable left border.
    pub fn border_left(mut self, v: bool) -> Self {
        self.props |= Props::BORDER_LEFT;
        self.border_edges.left = v;
        self
    }

    /// Set border foreground color for all sides.
    pub fn border_foreground(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_TOP_FG
            | Props::BORDER_RIGHT_FG
            | Props::BORDER_BOTTOM_FG
            | Props::BORDER_LEFT_FG;
        self.border_fg = [
            Some(c.clone_box()),
            Some(c.clone_box()),
            Some(c.clone_box()),
            Some(c.clone_box()),
        ];
        self
    }

    /// Set border foreground color for all sides using a theme slot.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{Border, ColorSlot, Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::new()
    ///     .border(Border::rounded())
    ///     .border_foreground_slot(&theme, ColorSlot::Border);
    /// ```
    pub fn border_foreground_slot(mut self, theme: &Theme, slot: ColorSlot) -> Self {
        let c = theme.get(slot);
        self.props |= Props::BORDER_TOP_FG
            | Props::BORDER_RIGHT_FG
            | Props::BORDER_BOTTOM_FG
            | Props::BORDER_LEFT_FG;
        self.border_fg = [
            Some(c.clone_box()),
            Some(c.clone_box()),
            Some(c.clone_box()),
            Some(c.clone_box()),
        ];
        self
    }

    /// Set border background color for all sides.
    pub fn border_background(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_TOP_BG
            | Props::BORDER_RIGHT_BG
            | Props::BORDER_BOTTOM_BG
            | Props::BORDER_LEFT_BG;
        self.border_bg = [
            Some(c.clone_box()),
            Some(c.clone_box()),
            Some(c.clone_box()),
            Some(c.clone_box()),
        ];
        self
    }

    /// Set border background color for all sides using a theme slot.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{Border, ColorSlot, Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::new()
    ///     .border(Border::rounded())
    ///     .border_background_slot(&theme, ColorSlot::Surface);
    /// ```
    pub fn border_background_slot(mut self, theme: &Theme, slot: ColorSlot) -> Self {
        let c = theme.get(slot);
        self.props |= Props::BORDER_TOP_BG
            | Props::BORDER_RIGHT_BG
            | Props::BORDER_BOTTOM_BG
            | Props::BORDER_LEFT_BG;
        self.border_bg = [
            Some(c.clone_box()),
            Some(c.clone_box()),
            Some(c.clone_box()),
            Some(c.clone_box()),
        ];
        self
    }

    // ==================== Theme Presets ====================

    /// Create a button-like style from theme colors.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::button_from_theme(&theme);
    /// ```
    pub fn button_from_theme(theme: &Theme) -> Self {
        Style::new()
            .foreground_color(theme.get(ColorSlot::Background))
            .background_color(theme.get(ColorSlot::Primary))
            .padding((1, 2))
            .bold()
    }

    /// Create an error message style from theme colors.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::error_from_theme(&theme);
    /// ```
    pub fn error_from_theme(theme: &Theme) -> Self {
        Style::new()
            .foreground_color(theme.get(ColorSlot::Error))
            .bold()
    }

    /// Create a muted/secondary text style from theme colors.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::muted_from_theme(&theme);
    /// ```
    pub fn muted_from_theme(theme: &Theme) -> Self {
        Style::new()
            .foreground_color(theme.get(ColorSlot::TextMuted))
            .italic()
    }

    /// Create a highlighted/selected item style from theme colors.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::selected_from_theme(&theme);
    /// ```
    pub fn selected_from_theme(theme: &Theme) -> Self {
        Style::new()
            .foreground_color(theme.get(ColorSlot::Text))
            .background_color(theme.get(ColorSlot::Selection))
    }

    /// Create a bordered panel style from theme colors.
    ///
    /// # Example
    /// ```rust
    /// use lipgloss::{Style, Theme};
    ///
    /// let theme = Theme::dark();
    /// let style = Style::panel_from_theme(&theme);
    /// ```
    pub fn panel_from_theme(theme: &Theme) -> Self {
        Style::new()
            .border(Border::rounded())
            .border_foreground_slot(theme, ColorSlot::Border)
            .padding(1)
    }

    // ==================== Other ====================

    /// Enable inline mode (single line, no margins/padding/borders).
    pub fn inline(mut self) -> Self {
        self.props |= Props::INLINE;
        self.attrs |= Attrs::INLINE;
        self
    }

    /// Set tab width (-1 = no conversion, 0 = remove tabs).
    pub fn tab_width(mut self, n: i8) -> Self {
        self.props |= Props::TAB_WIDTH;
        self.tab_width = n.max(-1);
        self
    }

    /// Set text transform function.
    pub fn transform<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.props |= Props::TRANSFORM;
        self.transform = Some(Arc::new(f));
        self
    }

    /// Set the renderer to use.
    pub fn renderer(mut self, r: Arc<Renderer>) -> Self {
        self.renderer = Some(r);
        self
    }

    // ==================== Queries ====================

    /// Check if a property is set.
    pub fn is_set(&self, prop: Props) -> bool {
        self.props.contains(prop)
    }

    /// Get the effective border edges (all if border is set but no edges specified).
    fn effective_border_edges(&self) -> BorderEdges {
        if !self.props.contains(Props::BORDER_STYLE) {
            return BorderEdges::none();
        }

        // If border style is set but no edges are explicitly set, enable all
        let has_explicit_edges = self.props.intersects(
            Props::BORDER_TOP | Props::BORDER_RIGHT | Props::BORDER_BOTTOM | Props::BORDER_LEFT,
        );

        if has_explicit_edges {
            self.border_edges
        } else {
            BorderEdges::all()
        }
    }

    // ==================== Rendering ====================

    /// Render the given text with this style applied.
    pub fn render(&self, text: &str) -> String {
        self.render_internal(text)
    }

    /// Internal render implementation.
    fn render_internal(&self, text: &str) -> String {
        let renderer = self
            .renderer
            .as_ref()
            .map(|r| r.as_ref())
            .unwrap_or(&Renderer::DEFAULT);
        let profile = renderer.color_profile();
        let dark_bg = renderer.has_dark_background();

        // Combine with stored value
        let mut str = if self.value.is_empty() {
            text.to_string()
        } else {
            format!("{} {}", self.value, text)
        };

        // Apply transform
        if let Some(ref transform) = self.transform {
            str = transform(&str);
        }

        // Early return if no props set
        if self.props.is_empty() {
            return self.maybe_convert_tabs(&str);
        }

        // Convert tabs
        str = self.maybe_convert_tabs(&str);

        // Strip carriage returns
        str = str.replace("\r\n", "\n");

        // Handle inline mode
        let is_inline = self.attrs.contains(Attrs::INLINE);
        if is_inline {
            str = str.replace('\n', "");
        }

        // Word wrap if width is set
        if !is_inline && self.props.contains(Props::WIDTH) && self.width > 0 {
            let wrap_at =
                self.width as usize - self.padding.left as usize - self.padding.right as usize;
            str = wrap_text(&str, wrap_at);
        }

        // Build ANSI escape sequences
        let mut style_start = String::new();

        // Text attributes
        if self.attrs.contains(Attrs::BOLD) {
            style_start.push_str("\x1b[1m");
        }
        if self.attrs.contains(Attrs::FAINT) {
            style_start.push_str("\x1b[2m");
        }
        if self.attrs.contains(Attrs::ITALIC) {
            style_start.push_str("\x1b[3m");
        }
        if self.attrs.contains(Attrs::UNDERLINE) {
            style_start.push_str("\x1b[4m");
        }
        if self.attrs.contains(Attrs::BLINK) {
            style_start.push_str("\x1b[5m");
        }
        if self.attrs.contains(Attrs::REVERSE) {
            style_start.push_str("\x1b[7m");
        }
        if self.attrs.contains(Attrs::STRIKETHROUGH) {
            style_start.push_str("\x1b[9m");
        }

        // Colors
        if let Some(ref fg) = self.fg_color {
            style_start.push_str(&fg.to_ansi_fg(profile, dark_bg));
        }
        if let Some(ref bg) = self.bg_color {
            style_start.push_str(&bg.to_ansi_bg(profile, dark_bg));
        }

        // Apply style to each line
        if !style_start.is_empty() {
            let lines: Vec<&str> = str.lines().collect();
            let styled_lines: Vec<String> = lines
                .iter()
                .map(|line| format!("{style_start}{line}\x1b[0m"))
                .collect();
            str = styled_lines.join("\n");
        }

        // Apply padding (if not inline)
        if !is_inline {
            str = self.apply_padding(&str, profile, dark_bg);
        }

        // Apply height
        if self.props.contains(Props::HEIGHT) && self.height > 0 {
            str = self.apply_height(&str);
        }

        // Apply width/alignment
        if self.props.contains(Props::WIDTH) && self.width > 0 {
            str = self.apply_width(&str, profile, dark_bg);
        }

        // Apply border (if not inline)
        if !is_inline {
            str = self.apply_border(&str, profile, dark_bg);
        }

        // Apply margin (if not inline)
        if !is_inline {
            str = self.apply_margin(&str, profile, dark_bg);
        }

        // Truncate to max dimensions
        if self.props.contains(Props::MAX_WIDTH) && self.max_width > 0 {
            str = truncate_width(&str, self.max_width as usize);
        }
        if self.props.contains(Props::MAX_HEIGHT) && self.max_height > 0 {
            str = truncate_height(&str, self.max_height as usize);
        }

        str
    }

    fn maybe_convert_tabs(&self, s: &str) -> String {
        let tw = if self.props.contains(Props::TAB_WIDTH) {
            self.tab_width
        } else {
            4 // Default
        };

        match tw {
            -1 => s.to_string(),
            0 => s.replace('\t', ""),
            n => s.replace('\t', &" ".repeat(n as usize)),
        }
    }

    fn apply_padding(&self, s: &str, profile: ColorProfile, dark_bg: bool) -> String {
        let mut result = s.to_string();

        // Build whitespace style
        let ws_style = if let Some(ref bg) = self.bg_color {
            bg.to_ansi_bg(profile, dark_bg)
        } else {
            String::new()
        };

        // Left padding
        if self.padding.left > 0 {
            let pad = if ws_style.is_empty() {
                " ".repeat(self.padding.left as usize)
            } else {
                format!(
                    "{}{}\x1b[0m",
                    ws_style,
                    " ".repeat(self.padding.left as usize)
                )
            };
            result = result
                .lines()
                .map(|line| format!("{pad}{line}"))
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Right padding
        if self.padding.right > 0 {
            let pad = if ws_style.is_empty() {
                " ".repeat(self.padding.right as usize)
            } else {
                format!(
                    "{}{}\x1b[0m",
                    ws_style,
                    " ".repeat(self.padding.right as usize)
                )
            };
            result = result
                .lines()
                .map(|line| format!("{line}{pad}"))
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Calculate content width for blank lines (after horizontal padding applied)
        let content_width = result.lines().map(|l| visible_width(l)).max().unwrap_or(0);

        // Top padding - create blank lines with proper width
        if self.padding.top > 0 {
            let blank_line = if ws_style.is_empty() {
                " ".repeat(content_width)
            } else {
                format!("{}{}\x1b[0m", ws_style, " ".repeat(content_width))
            };
            let top_lines = std::iter::repeat(blank_line)
                .take(self.padding.top as usize)
                .collect::<Vec<_>>()
                .join("\n");
            result = format!("{}\n{}", top_lines, result);
        }

        // Bottom padding - create blank lines with proper width
        if self.padding.bottom > 0 {
            let blank_line = if ws_style.is_empty() {
                " ".repeat(content_width)
            } else {
                format!("{}{}\x1b[0m", ws_style, " ".repeat(content_width))
            };
            let bottom_lines = std::iter::repeat(blank_line)
                .take(self.padding.bottom as usize)
                .collect::<Vec<_>>()
                .join("\n");
            result = format!("{}\n{}", result, bottom_lines);
        }

        result
    }

    fn apply_height(&self, s: &str) -> String {
        let lines: Vec<&str> = s.lines().collect();
        let current_height = lines.len();
        let target_height = self.height as usize;

        if current_height >= target_height {
            return s.to_string();
        }

        // Calculate content width for blank lines
        let content_width = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);
        let blank_line = " ".repeat(content_width);

        let extra = target_height - current_height;
        let factor = self.align_vertical.factor();
        let top_extra = (extra as f64 * factor).round() as usize;
        let bottom_extra = extra - top_extra;

        let mut result = Vec::with_capacity(target_height);

        for _ in 0..top_extra {
            result.push(blank_line.clone());
        }
        result.extend(lines.iter().map(|l| l.to_string()));
        for _ in 0..bottom_extra {
            result.push(blank_line.clone());
        }

        result.join("\n")
    }

    fn apply_width(&self, s: &str, profile: ColorProfile, dark_bg: bool) -> String {
        let target_width = self.width as usize;

        // Build whitespace style
        let ws_style = if let Some(ref bg) = self.bg_color {
            bg.to_ansi_bg(profile, dark_bg)
        } else {
            String::new()
        };

        let lines: Vec<&str> = s.lines().collect();
        let aligned: Vec<String> = lines
            .iter()
            .map(|line| {
                let line_width = visible_width(line);
                if line_width >= target_width {
                    line.to_string()
                } else {
                    let extra = target_width - line_width;
                    let factor = self.align_horizontal.factor();
                    let left_pad = (extra as f64 * factor).round() as usize;
                    let right_pad = extra - left_pad;

                    let left_spaces = " ".repeat(left_pad);
                    let right_spaces = " ".repeat(right_pad);

                    if ws_style.is_empty() {
                        format!("{left_spaces}{line}{right_spaces}")
                    } else {
                        format!(
                            "{ws_style}{left_spaces}\x1b[0m{line}{ws_style}{right_spaces}\x1b[0m"
                        )
                    }
                }
            })
            .collect();

        aligned.join("\n")
    }

    fn apply_border(&self, s: &str, profile: ColorProfile, dark_bg: bool) -> String {
        let edges = self.effective_border_edges();
        if !edges.any() || self.border_style.is_empty() {
            return s.to_string();
        }

        let border = &self.border_style;
        let lines: Vec<&str> = s.lines().collect();
        let content_width = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);

        // Helper to style border characters
        let style_border =
            |s: &str, fg: &Option<Box<dyn TerminalColor>>, bg: &Option<Box<dyn TerminalColor>>| {
                let mut result = String::new();
                if let Some(c) = fg {
                    result.push_str(&c.to_ansi_fg(profile, dark_bg));
                }
                if let Some(c) = bg {
                    result.push_str(&c.to_ansi_bg(profile, dark_bg));
                }
                result.push_str(s);
                if fg.is_some() || bg.is_some() {
                    result.push_str("\x1b[0m");
                }
                result
            };

        let mut result = Vec::new();

        // Top border
        if edges.top {
            let mut top_line = String::new();
            if edges.left {
                top_line.push_str(&style_border(
                    &border.top_left,
                    &self.border_fg[0],
                    &self.border_bg[0],
                ));
            }
            let horizontal = if border.top.is_empty() {
                " "
            } else {
                &border.top
            };
            top_line.push_str(&style_border(
                &horizontal.repeat(content_width.max(1)),
                &self.border_fg[0],
                &self.border_bg[0],
            ));
            if edges.right {
                top_line.push_str(&style_border(
                    &border.top_right,
                    &self.border_fg[0],
                    &self.border_bg[0],
                ));
            }
            result.push(top_line);
        }

        // Content with side borders
        for line in &lines {
            let mut row = String::new();
            if edges.left {
                row.push_str(&style_border(
                    &border.left,
                    &self.border_fg[3],
                    &self.border_bg[3],
                ));
            }
            row.push_str(line);
            if edges.right {
                row.push_str(&style_border(
                    &border.right,
                    &self.border_fg[1],
                    &self.border_bg[1],
                ));
            }
            result.push(row);
        }

        // Bottom border
        if edges.bottom {
            let mut bottom_line = String::new();
            if edges.left {
                bottom_line.push_str(&style_border(
                    &border.bottom_left,
                    &self.border_fg[2],
                    &self.border_bg[2],
                ));
            }
            let horizontal = if border.bottom.is_empty() {
                " "
            } else {
                &border.bottom
            };
            bottom_line.push_str(&style_border(
                &horizontal.repeat(content_width.max(1)),
                &self.border_fg[2],
                &self.border_bg[2],
            ));
            if edges.right {
                bottom_line.push_str(&style_border(
                    &border.bottom_right,
                    &self.border_fg[2],
                    &self.border_bg[2],
                ));
            }
            result.push(bottom_line);
        }

        result.join("\n")
    }

    fn apply_margin(&self, s: &str, profile: ColorProfile, dark_bg: bool) -> String {
        let mut result = s.to_string();

        // Build margin style
        let margin_style = if let Some(ref bg) = self.margin_bg_color {
            bg.to_ansi_bg(profile, dark_bg)
        } else {
            String::new()
        };

        // Left margin
        if self.margin.left > 0 {
            let pad = if margin_style.is_empty() {
                " ".repeat(self.margin.left as usize)
            } else {
                format!(
                    "{}{}\x1b[0m",
                    margin_style,
                    " ".repeat(self.margin.left as usize)
                )
            };
            result = result
                .lines()
                .map(|line| format!("{pad}{line}"))
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Right margin
        if self.margin.right > 0 {
            let pad = if margin_style.is_empty() {
                " ".repeat(self.margin.right as usize)
            } else {
                format!(
                    "{}{}\x1b[0m",
                    margin_style,
                    " ".repeat(self.margin.right as usize)
                )
            };
            result = result
                .lines()
                .map(|line| format!("{line}{pad}"))
                .collect::<Vec<_>>()
                .join("\n");
        }

        // Calculate content width for blank lines (after horizontal margins applied)
        let content_width = result.lines().map(|l| visible_width(l)).max().unwrap_or(0);

        // Top margin - create blank lines with proper width
        if self.margin.top > 0 {
            let blank_line = if margin_style.is_empty() {
                " ".repeat(content_width)
            } else {
                format!("{}{}\x1b[0m", margin_style, " ".repeat(content_width))
            };
            let top = std::iter::repeat(blank_line)
                .take(self.margin.top as usize)
                .collect::<Vec<_>>()
                .join("\n");
            result = format!("{}\n{}", top, result);
        }

        // Bottom margin - create blank lines with proper width
        if self.margin.bottom > 0 {
            let blank_line = if margin_style.is_empty() {
                " ".repeat(content_width)
            } else {
                format!("{}{}\x1b[0m", margin_style, " ".repeat(content_width))
            };
            let bottom = std::iter::repeat(blank_line)
                .take(self.margin.bottom as usize)
                .collect::<Vec<_>>()
                .join("\n");
            result = format!("{}\n{}", result, bottom);
        }

        result
    }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render(""))
    }
}

// Helper functions

/// Calculate the visible width of a string (excluding ANSI escapes).
fn visible_width(s: &str) -> usize {
    // Strip ANSI escape sequences
    let mut width = 0;
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        width += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
    }

    width
}

/// Simple text wrapping.
fn wrap_text(s: &str, width: usize) -> String {
    if width == 0 {
        return s.to_string();
    }

    let mut result = Vec::new();
    for line in s.lines() {
        if visible_width(line) <= width {
            result.push(line.to_string());
        } else {
            // Simple word wrap
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut current_line = String::new();
            let mut current_width = 0;

            for word in words {
                let word_width = visible_width(word);
                if current_width + word_width + 1 > width && current_width > 0 {
                    result.push(current_line);
                    current_line = word.to_string();
                    current_width = word_width;
                } else {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                        current_width += 1;
                    }
                    current_line.push_str(word);
                    current_width += word_width;
                }
            }
            if !current_line.is_empty() {
                result.push(current_line);
            }
        }
    }

    result.join("\n")
}

/// Truncate each line to max width.
fn truncate_width(s: &str, max_width: usize) -> String {
    s.lines()
        .map(|line| {
            if visible_width(line) <= max_width {
                line.to_string()
            } else {
                // Simple truncation (doesn't preserve ANSI)
                let mut width = 0;
                let mut result = String::new();
                for c in line.chars() {
                    let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
                    if width + cw > max_width {
                        break;
                    }
                    result.push(c);
                    width += cw;
                }
                result
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Truncate to max height (number of lines).
fn truncate_height(s: &str, max_height: usize) -> String {
    s.lines().take(max_height).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_builder() {
        let s = Style::new().bold().foreground("#ff0000");
        assert!(s.attrs.contains(Attrs::BOLD));
        assert!(s.props.contains(Props::FOREGROUND));
    }

    #[test]
    fn test_padding() {
        let s = Style::new().padding(2);
        assert_eq!(s.padding.top, 2);
        assert_eq!(s.padding.right, 2);
        assert_eq!(s.padding.bottom, 2);
        assert_eq!(s.padding.left, 2);
    }

    #[test]
    fn test_render_basic() {
        let s = Style::new().bold();
        let rendered = s.render("Hello");
        assert!(rendered.contains("\x1b[1m"));
        assert!(rendered.contains("Hello"));
    }

    #[test]
    fn test_visible_width() {
        assert_eq!(visible_width("hello"), 5);
        assert_eq!(visible_width("\x1b[1mhello\x1b[0m"), 5);
    }

    #[test]
    fn test_from_theme_colors_sets_fg_bg() {
        let theme = Theme::dark();
        let style = Style::from_theme_colors(&theme, ColorSlot::Primary, ColorSlot::Background);
        assert!(style.props.contains(Props::FOREGROUND));
        assert!(style.props.contains(Props::BACKGROUND));

        let fg = style
            .fg_color
            .as_ref()
            .expect("foreground color missing")
            .to_ansi_fg(ColorProfile::TrueColor, theme.is_dark());
        let bg = style
            .bg_color
            .as_ref()
            .expect("background color missing")
            .to_ansi_bg(ColorProfile::TrueColor, theme.is_dark());

        let expected_fg = theme
            .get(ColorSlot::Primary)
            .to_ansi_fg(ColorProfile::TrueColor, theme.is_dark());
        let expected_bg = theme
            .get(ColorSlot::Background)
            .to_ansi_bg(ColorProfile::TrueColor, theme.is_dark());

        assert_eq!(fg, expected_fg);
        assert_eq!(bg, expected_bg);
    }

    #[test]
    fn test_from_theme_sets_foreground() {
        let theme = Theme::dark();
        let style = Style::from_theme(&theme, ColorSlot::Accent);
        assert!(style.props.contains(Props::FOREGROUND));

        let fg = style
            .fg_color
            .as_ref()
            .expect("foreground color missing")
            .to_ansi_fg(ColorProfile::TrueColor, theme.is_dark());
        let expected_fg = theme
            .get(ColorSlot::Accent)
            .to_ansi_fg(ColorProfile::TrueColor, theme.is_dark());
        assert_eq!(fg, expected_fg);
    }

    #[test]
    fn test_from_theme_role_inverted_sets_fg_bg() {
        let theme = Theme::dark();
        let style = Style::from_theme_role(&theme, ThemeRole::Inverted);
        assert!(style.props.contains(Props::FOREGROUND));
        assert!(style.props.contains(Props::BACKGROUND));
    }

    #[test]
    fn test_theme_slot_builders_set_props() {
        let theme = Theme::dark();
        let style = Style::new()
            .foreground_slot(&theme, ColorSlot::Text)
            .background_slot(&theme, ColorSlot::Background)
            .border_foreground_slot(&theme, ColorSlot::Border)
            .border_background_slot(&theme, ColorSlot::Surface);

        assert!(style.props.contains(Props::FOREGROUND));
        assert!(style.props.contains(Props::BACKGROUND));
        assert!(style.props.contains(Props::BORDER_TOP_FG));
        assert!(style.props.contains(Props::BORDER_TOP_BG));
    }

    #[test]
    fn test_button_from_theme_padding() {
        let theme = Theme::dark();
        let style = Style::button_from_theme(&theme);
        assert_eq!(style.padding.top, 1);
        assert_eq!(style.padding.right, 2);
        assert_eq!(style.padding.bottom, 1);
        assert_eq!(style.padding.left, 2);
        assert!(style.attrs.contains(Attrs::BOLD));
    }
}
