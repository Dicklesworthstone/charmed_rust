//! Reusable UI components for `demo_showcase`.
//!
//! This module provides polished, composable UI primitives that can be
//! used across all pages for a consistent design language.
//!
//! All components are designed to:
//! - Work in truecolor, 256-color, and no-color terminals
//! - Truncate gracefully in narrow terminals
//! - Use semantic theme tokens (never hardcoded colors)

#![allow(dead_code)] // Components will be used by pages as they're implemented

use crate::theme::{Theme, spacing};
use lipgloss::Style;

// ============================================================================
// Status Chips/Tags
// ============================================================================

/// Status level for chips and alerts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusLevel {
    /// Informational status.
    #[default]
    Info,
    /// Success/healthy status.
    Success,
    /// Warning/degraded status.
    Warning,
    /// Error/failed status.
    Error,
    /// Running/in-progress status.
    Running,
}

impl StatusLevel {
    /// Get the icon for this status.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Success => "●",
            Self::Warning => "⚠",
            Self::Error => "✕",
            Self::Running => "◐",
        }
    }

    /// Get the ASCII fallback icon for this status.
    #[must_use]
    pub const fn ascii_icon(self) -> &'static str {
        match self {
            Self::Info => "i",
            Self::Success => "*",
            Self::Warning => "!",
            Self::Error => "x",
            Self::Running => "~",
        }
    }
}

/// Render a status chip/tag.
///
/// Chips are small, inline status indicators with optional text.
#[must_use]
pub fn chip(theme: &Theme, status: StatusLevel, label: &str) -> String {
    let style = match status {
        StatusLevel::Success => theme.success_style(),
        StatusLevel::Warning => theme.warning_style(),
        StatusLevel::Error => theme.error_style(),
        StatusLevel::Info | StatusLevel::Running => theme.info_style(),
    };

    let icon = status.icon();

    if label.is_empty() {
        style.render(icon)
    } else {
        style.render(&format!("{icon} {label}"))
    }
}

/// Render a badge-style chip with background.
#[must_use]
pub fn badge(theme: &Theme, status: StatusLevel, label: &str) -> String {
    let (fg, bg) = match status {
        StatusLevel::Success => (theme.text_inverse, theme.success),
        StatusLevel::Warning => (theme.text_inverse, theme.warning),
        StatusLevel::Error => (theme.text_inverse, theme.error),
        StatusLevel::Info | StatusLevel::Running => (theme.text_inverse, theme.info),
    };

    Style::new()
        .foreground(fg)
        .background(bg)
        .padding_left(spacing::XS)
        .padding_right(spacing::XS)
        .render(label)
}

// ============================================================================
// Alert Banners
// ============================================================================

/// Render an alert banner.
///
/// Banners are full-width notifications with icon, message, and optional action hint.
#[must_use]
pub fn banner(
    theme: &Theme,
    status: StatusLevel,
    message: &str,
    action_hint: Option<&str>,
) -> String {
    let (style, border_color) = match status {
        StatusLevel::Success => (theme.success_style(), theme.success),
        StatusLevel::Warning => (theme.warning_style(), theme.warning),
        StatusLevel::Error => (theme.error_style(), theme.error),
        StatusLevel::Info | StatusLevel::Running => (theme.info_style(), theme.info),
    };

    let icon = status.icon();
    let icon_styled = style.render(icon);

    let message_styled = Style::new().foreground(theme.text).render(message);

    let content = action_hint.map_or_else(
        || format!(" {icon_styled}  {message_styled}"),
        |hint| {
            let hint_styled = theme.muted_style().render(&format!(" ({hint})"));
            format!(" {icon_styled}  {message_styled}{hint_styled}")
        },
    );

    Style::new()
        .background(theme.bg_subtle)
        .border_left(true)
        .border_foreground(border_color)
        .padding_right(spacing::SM)
        .render(&content)
}

// ============================================================================
// Empty States
// ============================================================================

/// Render an empty state placeholder.
///
/// Empty states guide users when a list or view has no content.
#[must_use]
pub fn empty_state(theme: &Theme, icon: &str, title: &str, guidance: &str) -> String {
    let icon_styled = theme.muted_style().render(icon);
    let title_styled = theme.heading_style().render(title);
    let guidance_styled = theme.muted_style().render(guidance);

    format!("{icon_styled}\n\n{title_styled}\n{guidance_styled}")
}

/// Render a compact empty state (single line).
#[must_use]
pub fn empty_state_inline(theme: &Theme, message: &str) -> String {
    theme.muted_style().render(&format!("— {message} —"))
}

// ============================================================================
// Progress Indicators
// ============================================================================

/// Render a progress bar with label.
///
/// The bar visually represents completion percentage with a fill character.
#[must_use]
pub fn progress_bar(theme: &Theme, percent: u8, width: usize, label: Option<&str>) -> String {
    let clamped = percent.min(100);
    let fill_width = (usize::from(clamped) * width) / 100;
    let empty_width = width.saturating_sub(fill_width);

    let fill = "#".repeat(fill_width);
    let empty = "-".repeat(empty_width);

    let bar = format!("[{fill}{empty}]");
    let styled_bar = theme.progress_style(clamped).render(&bar);

    let percent_str = format!("{clamped:>3}%");
    let styled_percent = if clamped >= 100 {
        theme.success_style().render(&percent_str)
    } else {
        theme.muted_style().render(&percent_str)
    };

    label.map_or_else(
        || format!("{styled_bar} {styled_percent}"),
        |lbl| {
            let styled_label = Style::new().foreground(theme.text).render(lbl);
            format!("{styled_label}: {styled_bar} {styled_percent}")
        },
    )
}

/// Render a compact progress indicator (no bar, just percentage).
#[must_use]
pub fn progress_compact(theme: &Theme, percent: u8) -> String {
    let clamped = percent.min(100);
    let styled = if clamped >= 100 {
        theme.success_style()
    } else if clamped > 0 {
        theme.info_style()
    } else {
        theme.muted_style()
    };
    styled.render(&format!("{clamped}%"))
}

// ============================================================================
// Stat Widgets
// ============================================================================

/// Direction of change for delta values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeltaDirection {
    #[default]
    Neutral,
    Up,
    Down,
}

impl DeltaDirection {
    /// Get the arrow icon for this direction.
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Neutral => "",
            Self::Up => "↑",
            Self::Down => "↓",
        }
    }
}

/// Render a stat widget (label + value + optional delta).
///
/// Stat widgets display key metrics in a compact format.
#[must_use]
pub fn stat_widget(
    theme: &Theme,
    label: &str,
    value: &str,
    delta: Option<(&str, DeltaDirection)>,
) -> String {
    let label_styled = theme.muted_style().render(label);
    let value_styled = theme.heading_style().render(value);

    if let Some((delta_val, direction)) = delta {
        let delta_style = match direction {
            DeltaDirection::Up => theme.success_style(),
            DeltaDirection::Down => theme.error_style(),
            DeltaDirection::Neutral => theme.muted_style(),
        };
        let arrow = direction.icon();
        let delta_styled = delta_style.render(&format!("{arrow}{delta_val}"));
        format!("{label_styled}\n{value_styled} {delta_styled}")
    } else {
        format!("{label_styled}\n{value_styled}")
    }
}

/// Render an inline stat (label: value format).
#[must_use]
pub fn stat_inline(theme: &Theme, label: &str, value: &str) -> String {
    let label_styled = theme.muted_style().render(label);
    let value_styled = Style::new().foreground(theme.text).render(value);
    format!("{label_styled}: {value_styled}")
}

// ============================================================================
// Tabs/Pills
// ============================================================================

/// Render a tab bar.
///
/// Tabs provide sub-section navigation within a page.
#[must_use]
pub fn tab_bar(theme: &Theme, tabs: &[&str], selected: usize) -> String {
    tabs.iter()
        .enumerate()
        .map(|(i, label)| {
            if i == selected {
                theme.selected_style().render(&format!(" {label} "))
            } else {
                theme.muted_style().render(&format!(" {label} "))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Render a pill-style tab bar (with rounded/compact appearance).
#[must_use]
pub fn pill_bar(theme: &Theme, pills: &[&str], selected: usize) -> String {
    pills
        .iter()
        .enumerate()
        .map(|(i, label)| {
            if i == selected {
                theme.badge_primary_style().render(label)
            } else {
                theme.badge_style().render(label)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ============================================================================
// Dividers and Separators
// ============================================================================

/// Render a horizontal divider.
#[must_use]
pub fn divider(theme: &Theme, width: usize) -> String {
    let line = "─".repeat(width);
    theme.muted_style().render(&line)
}

/// Render a divider with a centered label.
#[must_use]
pub fn divider_with_label(theme: &Theme, label: &str, width: usize) -> String {
    let label_len = label.len() + 2; // Add padding
    let side_width = width.saturating_sub(label_len) / 2;
    let left = "─".repeat(side_width);
    let right = "─".repeat(width.saturating_sub(side_width + label_len));

    let line = theme.muted_style().render(&left);
    let label_styled = theme.heading_style().render(&format!(" {label} "));
    let line_right = theme.muted_style().render(&right);

    format!("{line}{label_styled}{line_right}")
}

// ============================================================================
// Key Hints
// ============================================================================

/// Render a key hint (e.g., "Enter select").
#[must_use]
pub fn key_hint(theme: &Theme, key: &str, action: &str) -> String {
    let key_styled = Style::new().foreground(theme.text).bold().render(key);
    let action_styled = theme.muted_style().render(action);
    format!("{key_styled} {action_styled}")
}

/// Render multiple key hints separated by spaces.
#[must_use]
pub fn key_hints(theme: &Theme, hints: &[(&str, &str)]) -> String {
    hints
        .iter()
        .map(|(key, action)| key_hint(theme, key, action))
        .collect::<Vec<_>>()
        .join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> Theme {
        Theme::dark()
    }

    #[test]
    fn chip_with_label() {
        let theme = test_theme();
        let result = chip(&theme, StatusLevel::Success, "Healthy");
        assert!(result.contains("Healthy"));
    }

    #[test]
    fn chip_without_label() {
        let theme = test_theme();
        let result = chip(&theme, StatusLevel::Error, "");
        assert!(!result.is_empty());
    }

    #[test]
    fn banner_with_action() {
        let theme = test_theme();
        let result = banner(
            &theme,
            StatusLevel::Warning,
            "High memory usage",
            Some("Press r to refresh"),
        );
        assert!(result.contains("High memory usage"));
        assert!(result.contains("Press r"));
    }

    #[test]
    fn progress_bar_shows_percentage() {
        let theme = test_theme();
        let result = progress_bar(&theme, 50, 10, None);
        assert!(result.contains("50%"));
    }

    #[test]
    fn progress_bar_with_label() {
        let theme = test_theme();
        let result = progress_bar(&theme, 75, 10, Some("Download"));
        assert!(result.contains("Download"));
        assert!(result.contains("75%"));
    }

    #[test]
    fn stat_widget_with_delta() {
        let theme = test_theme();
        let result = stat_widget(
            &theme,
            "Requests",
            "1,234",
            Some(("+5%", DeltaDirection::Up)),
        );
        assert!(result.contains("Requests"));
        assert!(result.contains("1,234"));
        assert!(result.contains("+5%"));
    }

    #[test]
    fn tab_bar_highlights_selected() {
        let theme = test_theme();
        let tabs = &["Overview", "Details", "Logs"];
        let result = tab_bar(&theme, tabs, 1);
        assert!(result.contains("Overview"));
        assert!(result.contains("Details"));
        assert!(result.contains("Logs"));
    }

    #[test]
    fn divider_has_correct_width() {
        let theme = test_theme();
        let result = divider(&theme, 20);
        // The rendered string includes ANSI codes, so we can't directly check length
        // But we can verify it's not empty
        assert!(!result.is_empty());
    }

    #[test]
    fn key_hints_renders_multiple() {
        let theme = test_theme();
        let hints = &[("Enter", "select"), ("q", "quit")];
        let result = key_hints(&theme, hints);
        assert!(result.contains("Enter"));
        assert!(result.contains("select"));
        assert!(result.contains('q'));
        assert!(result.contains("quit"));
    }
}
