//! Mouse input handling.
//!
//! This module provides types for representing mouse events including clicks,
//! scrolls, and motion.

use std::fmt;

/// Mouse event message.
///
/// MouseMsg is sent to the program's update function when mouse activity occurs.
/// Note: Mouse events must be enabled using `Program::with_mouse_cell_motion()`
/// or `Program::with_mouse_all_motion()`.
///
/// # Example
///
/// ```rust
/// use bubbletea::{MouseMsg, MouseButton, MouseAction};
///
/// fn handle_mouse(mouse: MouseMsg) {
///     if mouse.button == MouseButton::Left && mouse.action == MouseAction::Press {
///         println!("Left click at ({}, {})", mouse.x, mouse.y);
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseMsg {
    /// X coordinate (column), 0-indexed.
    pub x: u16,
    /// Y coordinate (row), 0-indexed.
    pub y: u16,
    /// Whether Shift was held.
    pub shift: bool,
    /// Whether Alt was held.
    pub alt: bool,
    /// Whether Ctrl was held.
    pub ctrl: bool,
    /// The action that occurred.
    pub action: MouseAction,
    /// The button involved.
    pub button: MouseButton,
}

impl MouseMsg {
    /// Check if this is a wheel event.
    pub fn is_wheel(&self) -> bool {
        matches!(
            self.button,
            MouseButton::WheelUp
                | MouseButton::WheelDown
                | MouseButton::WheelLeft
                | MouseButton::WheelRight
        )
    }
}

impl Default for MouseMsg {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            shift: false,
            alt: false,
            ctrl: false,
            action: MouseAction::Press,
            button: MouseButton::None,
        }
    }
}

impl fmt::Display for MouseMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            write!(f, "ctrl+")?;
        }
        if self.alt {
            write!(f, "alt+")?;
        }
        if self.shift {
            write!(f, "shift+")?;
        }

        if self.button == MouseButton::None {
            if self.action == MouseAction::Motion || self.action == MouseAction::Release {
                write!(f, "{}", self.action)?;
            } else {
                write!(f, "unknown")?;
            }
        } else if self.is_wheel() {
            write!(f, "{}", self.button)?;
        } else {
            write!(f, "{}", self.button)?;
            if self.action != MouseAction::Press {
                write!(f, " {}", self.action)?;
            }
        }
        Ok(())
    }
}

/// Mouse action type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MouseAction {
    /// Mouse button pressed.
    #[default]
    Press,
    /// Mouse button released.
    Release,
    /// Mouse moved.
    Motion,
}

impl fmt::Display for MouseAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            MouseAction::Press => "press",
            MouseAction::Release => "release",
            MouseAction::Motion => "motion",
        };
        write!(f, "{}", name)
    }
}

/// Mouse button identifier.
///
/// Based on X11 mouse button codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MouseButton {
    /// No button (motion only).
    #[default]
    None,
    /// Left button (button 1).
    Left,
    /// Middle button (button 2, scroll wheel click).
    Middle,
    /// Right button (button 3).
    Right,
    /// Scroll wheel up (button 4).
    WheelUp,
    /// Scroll wheel down (button 5).
    WheelDown,
    /// Scroll wheel left (button 6).
    WheelLeft,
    /// Scroll wheel right (button 7).
    WheelRight,
    /// Browser backward (button 8).
    Backward,
    /// Browser forward (button 9).
    Forward,
    /// Button 10.
    Button10,
    /// Button 11.
    Button11,
}

impl fmt::Display for MouseButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            MouseButton::None => "none",
            MouseButton::Left => "left",
            MouseButton::Middle => "middle",
            MouseButton::Right => "right",
            MouseButton::WheelUp => "wheel up",
            MouseButton::WheelDown => "wheel down",
            MouseButton::WheelLeft => "wheel left",
            MouseButton::WheelRight => "wheel right",
            MouseButton::Backward => "backward",
            MouseButton::Forward => "forward",
            MouseButton::Button10 => "button 10",
            MouseButton::Button11 => "button 11",
        };
        write!(f, "{}", name)
    }
}

/// Convert a crossterm mouse event to our MouseMsg.
pub fn from_crossterm_mouse(event: crossterm::event::MouseEvent) -> MouseMsg {
    use crossterm::event::{MouseButton as CtButton, MouseEventKind};

    let action = match event.kind {
        MouseEventKind::Down(_) => MouseAction::Press,
        MouseEventKind::Up(_) => MouseAction::Release,
        MouseEventKind::Drag(_) => MouseAction::Motion,
        MouseEventKind::Moved => MouseAction::Motion,
        MouseEventKind::ScrollUp => MouseAction::Press,
        MouseEventKind::ScrollDown => MouseAction::Press,
        MouseEventKind::ScrollLeft => MouseAction::Press,
        MouseEventKind::ScrollRight => MouseAction::Press,
    };

    let button = match event.kind {
        MouseEventKind::Down(b) | MouseEventKind::Up(b) | MouseEventKind::Drag(b) => match b {
            CtButton::Left => MouseButton::Left,
            CtButton::Right => MouseButton::Right,
            CtButton::Middle => MouseButton::Middle,
        },
        MouseEventKind::ScrollUp => MouseButton::WheelUp,
        MouseEventKind::ScrollDown => MouseButton::WheelDown,
        MouseEventKind::ScrollLeft => MouseButton::WheelLeft,
        MouseEventKind::ScrollRight => MouseButton::WheelRight,
        MouseEventKind::Moved => MouseButton::None,
    };

    MouseMsg {
        x: event.column,
        y: event.row,
        shift: event.modifiers.contains(crossterm::event::KeyModifiers::SHIFT),
        alt: event.modifiers.contains(crossterm::event::KeyModifiers::ALT),
        ctrl: event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL),
        action,
        button,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_msg_display() {
        let mouse = MouseMsg {
            x: 10,
            y: 20,
            shift: false,
            alt: false,
            ctrl: false,
            action: MouseAction::Press,
            button: MouseButton::Left,
        };
        assert_eq!(mouse.to_string(), "left");

        let mouse = MouseMsg {
            x: 10,
            y: 20,
            shift: false,
            alt: false,
            ctrl: true,
            action: MouseAction::Press,
            button: MouseButton::Left,
        };
        assert_eq!(mouse.to_string(), "ctrl+left");
    }

    #[test]
    fn test_mouse_is_wheel() {
        let mouse = MouseMsg {
            button: MouseButton::WheelUp,
            ..Default::default()
        };
        assert!(mouse.is_wheel());

        let mouse = MouseMsg {
            button: MouseButton::Left,
            ..Default::default()
        };
        assert!(!mouse.is_wheel());
    }

    #[test]
    fn test_mouse_button_display() {
        assert_eq!(MouseButton::Left.to_string(), "left");
        assert_eq!(MouseButton::WheelUp.to_string(), "wheel up");
    }

    #[test]
    fn test_mouse_action_display() {
        assert_eq!(MouseAction::Press.to_string(), "press");
        assert_eq!(MouseAction::Release.to_string(), "release");
        assert_eq!(MouseAction::Motion.to_string(), "motion");
    }
}
