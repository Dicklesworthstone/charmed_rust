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

    /// Set top border foreground color.
    pub fn border_top_foreground(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_TOP_FG;
        self.border_fg[0] = Some(c.clone_box());
        self
    }

    /// Set right border foreground color.
    pub fn border_right_foreground(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_RIGHT_FG;
        self.border_fg[1] = Some(c.clone_box());
        self
    }

    /// Set bottom border foreground color.
    pub fn border_bottom_foreground(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_BOTTOM_FG;
        self.border_fg[2] = Some(c.clone_box());
        self
    }

    /// Set left border foreground color.
    pub fn border_left_foreground(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_LEFT_FG;
        self.border_fg[3] = Some(c.clone_box());
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

    /// Set top border background color.
    pub fn border_top_background(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_TOP_BG;
        self.border_bg[0] = Some(c.clone_box());
        self
    }

    /// Set right border background color.
    pub fn border_right_background(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_RIGHT_BG;
        self.border_bg[1] = Some(c.clone_box());
        self
    }

    /// Set bottom border background color.
    pub fn border_bottom_background(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_BOTTOM_BG;
        self.border_bg[2] = Some(c.clone_box());
        self
    }

    /// Set left border background color.
    pub fn border_left_background(mut self, color: impl Into<String>) -> Self {
        let c = Color::new(color);
        self.props |= Props::BORDER_LEFT_BG;
        self.border_bg[3] = Some(c.clone_box());
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
    pub(crate) fn effective_border_edges(&self) -> BorderEdges {
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

    /// Get the horizontal frame size (left/right padding + border).
    ///
    /// This is useful for calculating content width when applying styles.
    pub fn get_horizontal_frame_size(&self) -> usize {
        let edges = self.effective_border_edges();
        let border_width = edges.horizontal_size(&self.border_style);
        let padding_width = self.padding.left as usize + self.padding.right as usize;
        border_width + padding_width
    }

    /// Get the vertical frame size (top/bottom padding + border).
    ///
    /// This is useful for calculating content height when applying styles.
    pub fn get_vertical_frame_size(&self) -> usize {
        let edges = self.effective_border_edges();
        let border_height = edges.vertical_size(&self.border_style);
        let padding_height = self.padding.top as usize + self.padding.bottom as usize;
        border_height + padding_height
    }

    /// Get the horizontal border size (left + right borders if enabled).
    pub fn get_horizontal_border_size(&self) -> usize {
        let edges = self.effective_border_edges();
        edges.horizontal_size(&self.border_style)
    }

    /// Get the vertical border size (top + bottom borders if enabled).
    pub fn get_vertical_border_size(&self) -> usize {
        let edges = self.effective_border_edges();
        edges.vertical_size(&self.border_style)
    }

    /// Get the horizontal padding (left + right).
    pub fn get_horizontal_padding(&self) -> usize {
        self.padding.left as usize + self.padding.right as usize
    }

    /// Get the vertical padding (top + bottom).
    pub fn get_vertical_padding(&self) -> usize {
        self.padding.top as usize + self.padding.bottom as usize
    }

    /// Get the horizontal margin (left + right).
    pub fn get_horizontal_margin(&self) -> usize {
        self.margin.left as usize + self.margin.right as usize
    }

    /// Get the vertical margin (top + bottom).
    pub fn get_vertical_margin(&self) -> usize {
        self.margin.top as usize + self.margin.bottom as usize
    }

    // ==================== Internal Accessors ====================

    pub(crate) fn attrs(&self) -> Attrs {
        self.attrs
    }

    pub(crate) fn foreground_color_ref(&self) -> Option<&dyn TerminalColor> {
        self.fg_color.as_deref()
    }

    pub(crate) fn background_color_ref(&self) -> Option<&dyn TerminalColor> {
        self.bg_color.as_deref()
    }

    pub(crate) fn border_style_ref(&self) -> &Border {
        &self.border_style
    }

    pub(crate) fn border_fg_ref(&self, index: usize) -> Option<&dyn TerminalColor> {
        self.border_fg.get(index).and_then(|c| c.as_deref())
    }

    pub(crate) fn get_padding(&self) -> Sides<u16> {
        self.padding
    }

    pub(crate) fn get_margin(&self) -> Sides<u16> {
        self.margin
    }

    /// Returns the configured width, if explicitly set.
    #[must_use]
    pub fn get_width(&self) -> Option<u16> {
        if self.props.contains(Props::WIDTH) {
            Some(self.width)
        } else {
            None
        }
    }

    /// Returns the configured height, if explicitly set.
    #[must_use]
    pub fn get_height(&self) -> Option<u16> {
        if self.props.contains(Props::HEIGHT) {
            Some(self.height)
        } else {
            None
        }
    }

    pub(crate) fn get_align_horizontal(&self) -> Position {
        self.align_horizontal
    }

    pub(crate) fn get_tab_width(&self) -> i8 {
        self.tab_width
    }

    pub(crate) fn has_custom_tab_width(&self) -> bool {
        self.props.contains(Props::TAB_WIDTH)
    }

    pub(crate) fn transform_ref(&self) -> Option<&TransformFn> {
        self.transform.as_ref()
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
            return self.maybe_convert_tabs(str);
        }

        // Convert tabs
        str = self.maybe_convert_tabs(str);

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

    /// Convert tabs in the string according to tab_width setting.
    /// Takes ownership to avoid allocation when no conversion is needed.
    fn maybe_convert_tabs(&self, s: String) -> String {
        let tw = if self.props.contains(Props::TAB_WIDTH) {
            self.tab_width
        } else {
            4 // Default
        };

        // Fast path: no conversion requested or no tabs present
        if tw == -1 || !s.contains('\t') {
            return s; // Return owned string as-is - zero allocation
        }

        match tw {
            0 => s.replace('\t', ""),
            n => s.replace('\t', &" ".repeat(n as usize)),
        }
    }

    fn apply_padding(&self, s: &str, profile: ColorProfile, dark_bg: bool) -> String {
        let left = self.padding.left as usize;
        let right = self.padding.right as usize;
        let top = self.padding.top as usize;
        let bottom = self.padding.bottom as usize;

        // Early return if no padding
        if left == 0 && right == 0 && top == 0 && bottom == 0 {
            return s.to_string();
        }

        // Build whitespace style once
        let ws_style = self
            .bg_color
            .as_ref()
            .map(|bg| bg.to_ansi_bg(profile, dark_bg));

        // Pre-compute padding strings once (avoid repeated allocations)
        let left_pad = if left > 0 {
            match &ws_style {
                Some(style) => format!("{}{}\x1b[0m", style, " ".repeat(left)),
                None => " ".repeat(left),
            }
        } else {
            String::new()
        };

        let right_pad = if right > 0 {
            match &ws_style {
                Some(style) => format!("{}{}\x1b[0m", style, " ".repeat(right)),
                None => " ".repeat(right),
            }
        } else {
            String::new()
        };

        // Collect lines and compute widths in single pass
        let lines: Vec<&str> = s.lines().collect();
        let line_count = lines.len();

        // Pre-allocate result with estimated capacity
        // Each line gets left_pad + content + right_pad + newline
        let estimated_capacity = lines.iter().map(|l| l.len()).sum::<usize>()
            + line_count * (left_pad.len() + right_pad.len() + 1)
            + (top + bottom) * (left + right + 80); // estimate for blank lines

        let mut result = String::with_capacity(estimated_capacity);

        // Apply left+right padding in single pass (avoiding .collect().join())
        let mut max_width = 0usize;
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&left_pad);
            result.push_str(line);
            result.push_str(&right_pad);

            // Track max width for blank lines (only if we need top/bottom padding)
            if top > 0 || bottom > 0 {
                let line_width = left + visible_width(line) + right;
                max_width = max_width.max(line_width);
            }
        }

        // Handle top/bottom padding
        if top > 0 || bottom > 0 {
            let blank_line = match &ws_style {
                Some(style) => format!("{}{}\x1b[0m", style, " ".repeat(max_width)),
                None => " ".repeat(max_width),
            };

            if top > 0 {
                let mut top_result =
                    String::with_capacity(top * (blank_line.len() + 1) + result.len() + 1);
                for i in 0..top {
                    if i > 0 {
                        top_result.push('\n');
                    }
                    top_result.push_str(&blank_line);
                }
                top_result.push('\n');
                top_result.push_str(&result);
                result = top_result;
            }

            if bottom > 0 {
                for _ in 0..bottom {
                    result.push('\n');
                    result.push_str(&blank_line);
                }
            }
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

        // Pre-allocate result - avoid Vec<String> intermediate and clone()
        let estimated_len = target_height * (content_width + 1);
        let mut result = String::with_capacity(estimated_len);

        // Add top blank lines (no clone - reuse blank_line reference)
        for i in 0..top_extra {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&blank_line);
        }

        // Add content lines (no to_string - push_str from &str directly)
        for (i, line) in lines.iter().enumerate() {
            if top_extra > 0 || i > 0 {
                result.push('\n');
            }
            result.push_str(line);
        }

        // Add bottom blank lines
        for _ in 0..bottom_extra {
            result.push('\n');
            result.push_str(&blank_line);
        }

        result
    }

    fn apply_width(&self, s: &str, profile: ColorProfile, dark_bg: bool) -> String {
        let target_width = self.width as usize;

        // Build whitespace style once
        let ws_style = if let Some(ref bg) = self.bg_color {
            bg.to_ansi_bg(profile, dark_bg)
        } else {
            String::new()
        };
        let has_ws_style = !ws_style.is_empty();

        // Pre-compute alignment factor once (avoid per-line call)
        let factor = self.align_horizontal.factor();

        // Pre-allocate result string
        // Estimate: each line gets up to target_width + ANSI codes (~20 bytes) + newline
        let line_count = s.lines().count();
        let estimated_capacity = line_count * (target_width + 25);
        let mut result = String::with_capacity(estimated_capacity);

        // Single-pass: avoid Vec<String> intermediate allocation
        for (i, line) in s.lines().enumerate() {
            if i > 0 {
                result.push('\n');
            }

            let line_width = visible_width(line);
            if line_width >= target_width {
                result.push_str(line);
            } else {
                let extra = target_width - line_width;
                let left_pad = (extra as f64 * factor).round() as usize;
                let right_pad = extra - left_pad;

                if has_ws_style {
                    // With background styling
                    result.push_str(&ws_style);
                    for _ in 0..left_pad {
                        result.push(' ');
                    }
                    result.push_str("\x1b[0m");
                    result.push_str(line);
                    result.push_str(&ws_style);
                    for _ in 0..right_pad {
                        result.push(' ');
                    }
                    result.push_str("\x1b[0m");
                } else {
                    // Without styling - just spaces
                    for _ in 0..left_pad {
                        result.push(' ');
                    }
                    result.push_str(line);
                    for _ in 0..right_pad {
                        result.push(' ');
                    }
                }
            }
        }

        result
    }

    fn apply_border(&self, s: &str, profile: ColorProfile, dark_bg: bool) -> String {
        let edges = self.effective_border_edges();
        if !edges.any() || self.border_style.is_empty() {
            return s.to_string();
        }

        let border = &self.border_style;
        let lines: Vec<&str> = s.lines().collect();
        let content_width = lines.iter().map(|l| visible_width(l)).max().unwrap_or(0);

        // Helper to build styled border string
        #[inline]
        fn style_border_str(
            s: &str,
            fg: Option<&dyn TerminalColor>,
            bg: Option<&dyn TerminalColor>,
            profile: ColorProfile,
            dark_bg: bool,
        ) -> String {
            if fg.is_none() && bg.is_none() {
                return s.to_string();
            }
            let mut result = String::with_capacity(s.len() + 20);
            if let Some(c) = fg {
                result.push_str(&c.to_ansi_fg(profile, dark_bg));
            }
            if let Some(c) = bg {
                result.push_str(&c.to_ansi_bg(profile, dark_bg));
            }
            result.push_str(s);
            result.push_str("\x1b[0m");
            result
        }

        // Pre-compute styled border elements (called once, not per-line)
        let left_border = if edges.left {
            style_border_str(
                &border.left,
                self.border_fg[3].as_deref(),
                self.border_bg[3].as_deref(),
                profile,
                dark_bg,
            )
        } else {
            String::new()
        };

        let right_border = if edges.right {
            style_border_str(
                &border.right,
                self.border_fg[1].as_deref(),
                self.border_bg[1].as_deref(),
                profile,
                dark_bg,
            )
        } else {
            String::new()
        };

        // Estimate capacity for result
        let line_count = lines.len();
        let border_lines = edges.top as usize + edges.bottom as usize;
        let border_width = left_border.len() + right_border.len();
        let total_len: usize = lines.iter().map(|l| l.len()).sum();
        let avg_line_len = total_len.checked_div(line_count).unwrap_or(0);
        let estimated_capacity = (line_count + border_lines) * (avg_line_len + border_width + 1);

        let mut result = String::with_capacity(estimated_capacity);

        // Top border
        if edges.top {
            if edges.left {
                result.push_str(&style_border_str(
                    &border.top_left,
                    self.border_fg[0].as_deref(),
                    self.border_bg[0].as_deref(),
                    profile,
                    dark_bg,
                ));
            }
            let horizontal = if border.top.is_empty() {
                " "
            } else {
                &border.top
            };
            result.push_str(&style_border_str(
                &horizontal.repeat(content_width.max(1)),
                self.border_fg[0].as_deref(),
                self.border_bg[0].as_deref(),
                profile,
                dark_bg,
            ));
            if edges.right {
                result.push_str(&style_border_str(
                    &border.top_right,
                    self.border_fg[0].as_deref(),
                    self.border_bg[0].as_deref(),
                    profile,
                    dark_bg,
                ));
            }
            result.push('\n');
        }

        // Content with side borders (reuse pre-computed border strings)
        for (i, line) in lines.iter().enumerate() {
            if i > 0 || edges.top {
                if i > 0 {
                    result.push('\n');
                }
            }
            result.push_str(&left_border);
            result.push_str(line);
            result.push_str(&right_border);
        }

        // Bottom border
        if edges.bottom {
            result.push('\n');
            if edges.left {
                result.push_str(&style_border_str(
                    &border.bottom_left,
                    self.border_fg[2].as_deref(),
                    self.border_bg[2].as_deref(),
                    profile,
                    dark_bg,
                ));
            }
            let horizontal = if border.bottom.is_empty() {
                " "
            } else {
                &border.bottom
            };
            result.push_str(&style_border_str(
                &horizontal.repeat(content_width.max(1)),
                self.border_fg[2].as_deref(),
                self.border_bg[2].as_deref(),
                profile,
                dark_bg,
            ));
            if edges.right {
                result.push_str(&style_border_str(
                    &border.bottom_right,
                    self.border_fg[2].as_deref(),
                    self.border_bg[2].as_deref(),
                    profile,
                    dark_bg,
                ));
            }
        }

        result
    }

    fn apply_margin(&self, s: &str, profile: ColorProfile, dark_bg: bool) -> String {
        let left = self.margin.left as usize;
        let right = self.margin.right as usize;
        let top = self.margin.top as usize;
        let bottom = self.margin.bottom as usize;

        // Early return if no margin
        if left == 0 && right == 0 && top == 0 && bottom == 0 {
            return s.to_string();
        }

        // Build margin style once
        let margin_style = self
            .margin_bg_color
            .as_ref()
            .map(|bg| bg.to_ansi_bg(profile, dark_bg));

        // Pre-compute padding strings once (avoid repeated allocations)
        let left_pad = if left > 0 {
            match &margin_style {
                Some(style) => format!("{}{}\x1b[0m", style, " ".repeat(left)),
                None => " ".repeat(left),
            }
        } else {
            String::new()
        };

        let right_pad = if right > 0 {
            match &margin_style {
                Some(style) => format!("{}{}\x1b[0m", style, " ".repeat(right)),
                None => " ".repeat(right),
            }
        } else {
            String::new()
        };

        // Single pass: apply left/right margins and track max width
        let lines: Vec<&str> = s.lines().collect();
        let mut max_width = 0usize;

        // Estimate capacity for result
        let avg_line_len = lines.first().map(|l| l.len()).unwrap_or(40);
        let estimated_len = lines.len() * (avg_line_len + left_pad.len() + right_pad.len() + 1)
            + (top + bottom) * (avg_line_len + left + right + 20);
        let mut result = String::with_capacity(estimated_len);

        // Build content with horizontal margins in single pass
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&left_pad);
            result.push_str(line);
            result.push_str(&right_pad);

            // Track width for blank lines (only if we need top/bottom margins)
            if top > 0 || bottom > 0 {
                let line_width = left + visible_width(line) + right;
                max_width = max_width.max(line_width);
            }
        }

        // Handle top/bottom margins if needed
        if top > 0 || bottom > 0 {
            let blank_line = match &margin_style {
                Some(style) => format!("{}{}\x1b[0m", style, " ".repeat(max_width)),
                None => " ".repeat(max_width),
            };

            if top > 0 {
                let mut top_result =
                    String::with_capacity(top * (blank_line.len() + 1) + result.len() + 1);
                for i in 0..top {
                    if i > 0 {
                        top_result.push('\n');
                    }
                    top_result.push_str(&blank_line);
                }
                top_result.push('\n');
                top_result.push_str(&result);
                result = top_result;
            }

            if bottom > 0 {
                for _ in 0..bottom {
                    result.push('\n');
                    result.push_str(&blank_line);
                }
            }
        }

        result
    }

    // =============================================================================
    // Unset methods - remove style rules
    // =============================================================================

    /// Removes the bold style rule.
    pub fn unset_bold(mut self) -> Self {
        self.props.remove(Props::BOLD);
        self.attrs.remove(Attrs::BOLD);
        self
    }

    /// Removes the italic style rule.
    pub fn unset_italic(mut self) -> Self {
        self.props.remove(Props::ITALIC);
        self.attrs.remove(Attrs::ITALIC);
        self
    }

    /// Removes the underline style rule.
    pub fn unset_underline(mut self) -> Self {
        self.props.remove(Props::UNDERLINE);
        self.attrs.remove(Attrs::UNDERLINE);
        self
    }

    /// Removes the strikethrough style rule.
    pub fn unset_strikethrough(mut self) -> Self {
        self.props.remove(Props::STRIKETHROUGH);
        self.attrs.remove(Attrs::STRIKETHROUGH);
        self
    }

    /// Removes the reverse style rule.
    pub fn unset_reverse(mut self) -> Self {
        self.props.remove(Props::REVERSE);
        self.attrs.remove(Attrs::REVERSE);
        self
    }

    /// Removes the blink style rule.
    pub fn unset_blink(mut self) -> Self {
        self.props.remove(Props::BLINK);
        self.attrs.remove(Attrs::BLINK);
        self
    }

    /// Removes the faint style rule.
    pub fn unset_faint(mut self) -> Self {
        self.props.remove(Props::FAINT);
        self.attrs.remove(Attrs::FAINT);
        self
    }

    /// Removes the foreground color rule.
    pub fn unset_foreground(mut self) -> Self {
        self.props.remove(Props::FOREGROUND);
        self.fg_color = None;
        self
    }

    /// Removes the background color rule.
    pub fn unset_background(mut self) -> Self {
        self.props.remove(Props::BACKGROUND);
        self.bg_color = None;
        self
    }

    /// Removes the width style rule.
    pub fn unset_width(mut self) -> Self {
        self.props.remove(Props::WIDTH);
        self.width = 0;
        self
    }

    /// Removes the height style rule.
    pub fn unset_height(mut self) -> Self {
        self.props.remove(Props::HEIGHT);
        self.height = 0;
        self
    }

    /// Removes the max width style rule.
    pub fn unset_max_width(mut self) -> Self {
        self.props.remove(Props::MAX_WIDTH);
        self.max_width = 0;
        self
    }

    /// Removes the max height style rule.
    pub fn unset_max_height(mut self) -> Self {
        self.props.remove(Props::MAX_HEIGHT);
        self.max_height = 0;
        self
    }

    /// Removes horizontal and vertical text alignment.
    pub fn unset_align(mut self) -> Self {
        self.props.remove(Props::ALIGN_HORIZONTAL);
        self.props.remove(Props::ALIGN_VERTICAL);
        self.align_horizontal = Position::Left;
        self.align_vertical = Position::Top;
        self
    }

    /// Removes horizontal text alignment.
    pub fn unset_align_horizontal(mut self) -> Self {
        self.props.remove(Props::ALIGN_HORIZONTAL);
        self.align_horizontal = Position::Left;
        self
    }

    /// Removes vertical text alignment.
    pub fn unset_align_vertical(mut self) -> Self {
        self.props.remove(Props::ALIGN_VERTICAL);
        self.align_vertical = Position::Top;
        self
    }

    /// Removes all padding style rules.
    pub fn unset_padding(mut self) -> Self {
        self.props.remove(Props::PADDING_TOP);
        self.props.remove(Props::PADDING_RIGHT);
        self.props.remove(Props::PADDING_BOTTOM);
        self.props.remove(Props::PADDING_LEFT);
        self.padding = Sides::default();
        self
    }

    /// Removes the left padding rule.
    pub fn unset_padding_left(mut self) -> Self {
        self.props.remove(Props::PADDING_LEFT);
        self.padding.left = 0;
        self
    }

    /// Removes the right padding rule.
    pub fn unset_padding_right(mut self) -> Self {
        self.props.remove(Props::PADDING_RIGHT);
        self.padding.right = 0;
        self
    }

    /// Removes the top padding rule.
    pub fn unset_padding_top(mut self) -> Self {
        self.props.remove(Props::PADDING_TOP);
        self.padding.top = 0;
        self
    }

    /// Removes the bottom padding rule.
    pub fn unset_padding_bottom(mut self) -> Self {
        self.props.remove(Props::PADDING_BOTTOM);
        self.padding.bottom = 0;
        self
    }

    /// Removes all margin style rules.
    pub fn unset_margins(mut self) -> Self {
        self.props.remove(Props::MARGIN_TOP);
        self.props.remove(Props::MARGIN_RIGHT);
        self.props.remove(Props::MARGIN_BOTTOM);
        self.props.remove(Props::MARGIN_LEFT);
        self.margin = Sides::default();
        self
    }

    /// Removes the left margin rule.
    pub fn unset_margin_left(mut self) -> Self {
        self.props.remove(Props::MARGIN_LEFT);
        self.margin.left = 0;
        self
    }

    /// Removes the right margin rule.
    pub fn unset_margin_right(mut self) -> Self {
        self.props.remove(Props::MARGIN_RIGHT);
        self.margin.right = 0;
        self
    }

    /// Removes the top margin rule.
    pub fn unset_margin_top(mut self) -> Self {
        self.props.remove(Props::MARGIN_TOP);
        self.margin.top = 0;
        self
    }

    /// Removes the bottom margin rule.
    pub fn unset_margin_bottom(mut self) -> Self {
        self.props.remove(Props::MARGIN_BOTTOM);
        self.margin.bottom = 0;
        self
    }

    /// Removes the margin background color.
    pub fn unset_margin_background(mut self) -> Self {
        self.props.remove(Props::MARGIN_BACKGROUND);
        self.margin_bg_color = None;
        self
    }

    /// Removes the border style rule.
    pub fn unset_border_style(mut self) -> Self {
        self.props.remove(Props::BORDER_STYLE);
        self.border_style = Border::none();
        self
    }

    /// Removes the top border rule.
    pub fn unset_border_top(mut self) -> Self {
        self.props.remove(Props::BORDER_TOP);
        self.border_edges.top = false;
        self
    }

    /// Removes the right border rule.
    pub fn unset_border_right(mut self) -> Self {
        self.props.remove(Props::BORDER_RIGHT);
        self.border_edges.right = false;
        self
    }

    /// Removes the bottom border rule.
    pub fn unset_border_bottom(mut self) -> Self {
        self.props.remove(Props::BORDER_BOTTOM);
        self.border_edges.bottom = false;
        self
    }

    /// Removes the left border rule.
    pub fn unset_border_left(mut self) -> Self {
        self.props.remove(Props::BORDER_LEFT);
        self.border_edges.left = false;
        self
    }

    /// Removes all border foreground colors.
    pub fn unset_border_foreground(mut self) -> Self {
        self.props.remove(Props::BORDER_TOP_FG);
        self.props.remove(Props::BORDER_RIGHT_FG);
        self.props.remove(Props::BORDER_BOTTOM_FG);
        self.props.remove(Props::BORDER_LEFT_FG);
        self.border_fg = [None, None, None, None];
        self
    }

    /// Removes the top border foreground color.
    pub fn unset_border_top_foreground(mut self) -> Self {
        self.props.remove(Props::BORDER_TOP_FG);
        self.border_fg[0] = None;
        self
    }

    /// Removes the right border foreground color.
    pub fn unset_border_right_foreground(mut self) -> Self {
        self.props.remove(Props::BORDER_RIGHT_FG);
        self.border_fg[1] = None;
        self
    }

    /// Removes the bottom border foreground color.
    pub fn unset_border_bottom_foreground(mut self) -> Self {
        self.props.remove(Props::BORDER_BOTTOM_FG);
        self.border_fg[2] = None;
        self
    }

    /// Removes the left border foreground color.
    pub fn unset_border_left_foreground(mut self) -> Self {
        self.props.remove(Props::BORDER_LEFT_FG);
        self.border_fg[3] = None;
        self
    }

    /// Removes all border background colors.
    pub fn unset_border_background(mut self) -> Self {
        self.props.remove(Props::BORDER_TOP_BG);
        self.props.remove(Props::BORDER_RIGHT_BG);
        self.props.remove(Props::BORDER_BOTTOM_BG);
        self.props.remove(Props::BORDER_LEFT_BG);
        self.border_bg = [None, None, None, None];
        self
    }

    /// Removes the top border background color.
    pub fn unset_border_top_background(mut self) -> Self {
        self.props.remove(Props::BORDER_TOP_BG);
        self.border_bg[0] = None;
        self
    }

    /// Removes the right border background color.
    pub fn unset_border_right_background(mut self) -> Self {
        self.props.remove(Props::BORDER_RIGHT_BG);
        self.border_bg[1] = None;
        self
    }

    /// Removes the bottom border background color.
    pub fn unset_border_bottom_background(mut self) -> Self {
        self.props.remove(Props::BORDER_BOTTOM_BG);
        self.border_bg[2] = None;
        self
    }

    /// Removes the left border background color.
    pub fn unset_border_left_background(mut self) -> Self {
        self.props.remove(Props::BORDER_LEFT_BG);
        self.border_bg[3] = None;
        self
    }

    /// Removes the inline style rule.
    pub fn unset_inline(mut self) -> Self {
        self.props.remove(Props::INLINE);
        self.attrs.remove(Attrs::INLINE);
        self
    }

    /// Removes the tab width style rule.
    pub fn unset_tab_width(mut self) -> Self {
        self.props.remove(Props::TAB_WIDTH);
        self.tab_width = 4;
        self
    }

    /// Removes the underline spaces value.
    pub fn unset_underline_spaces(mut self) -> Self {
        self.props.remove(Props::UNDERLINE_SPACES);
        self.attrs.remove(Attrs::UNDERLINE_SPACES);
        self
    }

    /// Removes the strikethrough spaces value.
    pub fn unset_strikethrough_spaces(mut self) -> Self {
        self.props.remove(Props::STRIKETHROUGH_SPACES);
        self.attrs.remove(Attrs::STRIKETHROUGH_SPACES);
        self
    }

    /// Removes the color whitespace value.
    pub fn unset_color_whitespace(mut self) -> Self {
        self.props.remove(Props::COLOR_WHITESPACE);
        self.attrs.remove(Attrs::COLOR_WHITESPACE);
        self
    }

    /// Removes the transform value.
    pub fn unset_transform(mut self) -> Self {
        self.props.remove(Props::TRANSFORM);
        self.transform = None;
        self
    }
}

impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render(""))
    }
}

// Helper functions

/// Calculate the visible width of a string (excluding ANSI escapes).
///
/// Optimized with fast path for ASCII-only content (common case).
#[inline]
fn visible_width(s: &str) -> usize {
    // Fast path: check if string is ASCII-only and has no escapes
    // This is the common case for most terminal text
    if s.is_ascii() && !s.contains('\x1b') {
        return s.len();
    }

    // Slow path: handle ANSI escapes and Unicode width
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
        // Fast path for ASCII in mixed content
        if c.is_ascii() {
            width += 1;
        } else {
            width += unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        }
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
    fn test_visible_width_unicode() {
        // CJK characters are 2 display units wide
        assert_eq!(visible_width("日本"), 4);
        assert_eq!(visible_width("中文"), 4);

        // Emoji
        assert_eq!(visible_width("🦀"), 2);
        assert_eq!(visible_width("🎉"), 2);

        // Mixed content
        assert_eq!(visible_width("Hi日本"), 6); // 2 + 4

        // With ANSI codes
        assert_eq!(visible_width("\x1b[31m日本\x1b[0m"), 4);
        assert_eq!(visible_width("\x1b[1m🦀\x1b[0m"), 2);
    }

    #[test]
    fn test_visible_width_combining_chars() {
        // Combining character: e + combining acute
        let combining = "e\u{0301}"; // é as two code points
        assert_eq!(visible_width(combining), 1);

        // Precomposed form
        let precomposed = "é";
        assert_eq!(visible_width(precomposed), 1);
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
    fn test_partial_border_top_only() {
        let style = Style::new()
            .border(Border::normal())
            .border_top(true)
            .border_right(false)
            .border_bottom(false)
            .border_left(false);

        let rendered = style.render("Hello");
        // Should have top border line but no side borders
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 2); // top border + content
        assert!(lines[0].contains("─")); // top edge
        assert!(!lines[1].contains("│")); // no side borders
    }

    #[test]
    fn test_partial_border_left_right_only() {
        let style = Style::new()
            .border(Border::normal())
            .border_top(false)
            .border_right(true)
            .border_bottom(false)
            .border_left(true);

        let rendered = style.render("Hi");
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 1); // just content with side borders
        assert!(lines[0].contains("│")); // side borders present
    }

    #[test]
    fn test_frame_size_with_all_borders() {
        let style = Style::new().border(Border::normal()).padding(2);

        // All borders enabled by default when border style is set
        assert_eq!(style.get_horizontal_border_size(), 2); // left + right
        assert_eq!(style.get_vertical_border_size(), 2); // top + bottom
        assert_eq!(style.get_horizontal_padding(), 4); // 2 + 2
        assert_eq!(style.get_vertical_padding(), 4); // 2 + 2
        assert_eq!(style.get_horizontal_frame_size(), 6); // borders + padding
        assert_eq!(style.get_vertical_frame_size(), 6);
    }

    #[test]
    fn test_frame_size_with_partial_borders() {
        let style = Style::new()
            .border(Border::normal())
            .border_top(true)
            .border_right(false)
            .border_bottom(true)
            .border_left(false)
            .padding(1);

        // Only top and bottom borders
        assert_eq!(style.get_horizontal_border_size(), 0); // no left/right
        assert_eq!(style.get_vertical_border_size(), 2); // top + bottom
        assert_eq!(style.get_horizontal_frame_size(), 2); // just padding
        assert_eq!(style.get_vertical_frame_size(), 4); // borders + padding
    }

    #[test]
    fn test_frame_size_no_border() {
        let style = Style::new().padding((1, 2));

        assert_eq!(style.get_horizontal_border_size(), 0);
        assert_eq!(style.get_vertical_border_size(), 0);
        assert_eq!(style.get_horizontal_padding(), 4); // 2 + 2
        assert_eq!(style.get_vertical_padding(), 2); // 1 + 1
        assert_eq!(style.get_horizontal_frame_size(), 4);
        assert_eq!(style.get_vertical_frame_size(), 2);
    }

    #[test]
    fn test_margin_sizes() {
        let style = Style::new().margin((1, 2, 3, 4)); // top, right, bottom, left

        assert_eq!(style.get_horizontal_margin(), 6); // 2 + 4
        assert_eq!(style.get_vertical_margin(), 4); // 1 + 3
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

    #[test]
    fn test_per_edge_border_foreground() {
        let style = Style::new()
            .border(Border::normal())
            .border_top_foreground("#ff0000")
            .border_right_foreground("#00ff00")
            .border_bottom_foreground("#0000ff")
            .border_left_foreground("#ffff00");

        // Verify props are set correctly
        assert!(style.props.contains(Props::BORDER_TOP_FG));
        assert!(style.props.contains(Props::BORDER_RIGHT_FG));
        assert!(style.props.contains(Props::BORDER_BOTTOM_FG));
        assert!(style.props.contains(Props::BORDER_LEFT_FG));

        // Verify colors are stored in correct positions
        assert!(style.border_fg[0].is_some()); // top
        assert!(style.border_fg[1].is_some()); // right
        assert!(style.border_fg[2].is_some()); // bottom
        assert!(style.border_fg[3].is_some()); // left
    }

    #[test]
    fn test_per_edge_border_background() {
        let style = Style::new()
            .border(Border::normal())
            .border_top_background("#111111")
            .border_right_background("#222222")
            .border_bottom_background("#333333")
            .border_left_background("#444444");

        // Verify props are set correctly
        assert!(style.props.contains(Props::BORDER_TOP_BG));
        assert!(style.props.contains(Props::BORDER_RIGHT_BG));
        assert!(style.props.contains(Props::BORDER_BOTTOM_BG));
        assert!(style.props.contains(Props::BORDER_LEFT_BG));

        // Verify colors are stored in correct positions
        assert!(style.border_bg[0].is_some()); // top
        assert!(style.border_bg[1].is_some()); // right
        assert!(style.border_bg[2].is_some()); // bottom
        assert!(style.border_bg[3].is_some()); // left
    }

    #[test]
    fn test_mixed_border_colors() {
        // Test setting all sides then overriding one
        let style = Style::new()
            .border(Border::normal())
            .border_foreground("#ffffff")
            .border_top_foreground("#ff0000"); // Override just top

        assert!(style.border_fg[0].is_some()); // top (overridden)
        assert!(style.border_fg[1].is_some()); // right
        assert!(style.border_fg[2].is_some()); // bottom
        assert!(style.border_fg[3].is_some()); // left
    }

    #[test]
    fn test_per_edge_border_renders() {
        let style = Style::new()
            .border(Border::normal())
            .border_top_foreground("#ff0000")
            .border_left_foreground("#00ff00");

        // Should render without panicking
        let rendered = style.render("Hello");
        assert!(!rendered.is_empty());
        assert!(rendered.contains("Hello"));
    }

    #[test]
    fn test_transform_method() {
        let style = Style::new().transform(|s| s.to_uppercase());

        let rendered = style.render("hello");
        assert_eq!(rendered, "HELLO");
    }

    #[test]
    fn test_transform_with_other_styles() {
        let style = Style::new().bold().transform(|s| s.to_uppercase());

        // Transform is applied to content
        let rendered = style.render("hello");
        // Should contain uppercase HELLO (with ANSI codes for bold)
        assert!(rendered.contains("HELLO"));
    }

    #[test]
    fn test_transform_closure_captures() {
        let prefix = ">>> ";
        let style = Style::new().transform(move |s| format!("{}{}", prefix, s));

        let rendered = style.render("test");
        assert_eq!(rendered, ">>> test");
    }
}
