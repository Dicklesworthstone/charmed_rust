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
            write!(f, " {}", self.action)?;
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
        shift: event
            .modifiers
            .contains(crossterm::event::KeyModifiers::SHIFT),
        alt: event
            .modifiers
            .contains(crossterm::event::KeyModifiers::ALT),
        ctrl: event
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL),
        action,
        button,
    }
}

/// Errors that can occur while parsing mouse escape sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseParseError {
    /// The sequence is not a supported mouse format.
    UnsupportedSequence,
    /// The sequence format is invalid.
    InvalidFormat,
    /// Numeric fields could not be parsed.
    InvalidNumber,
    /// Coordinates underflowed when converting to 0-indexed values.
    CoordinateUnderflow,
}

impl fmt::Display for MouseParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            MouseParseError::UnsupportedSequence => "unsupported mouse sequence",
            MouseParseError::InvalidFormat => "invalid mouse sequence format",
            MouseParseError::InvalidNumber => "invalid numeric value in mouse sequence",
            MouseParseError::CoordinateUnderflow => "mouse coordinates underflowed",
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for MouseParseError {}

#[derive(Debug, Clone, Copy)]
struct ParsedMouse {
    button: MouseButton,
    action: MouseAction,
    shift: bool,
    alt: bool,
    ctrl: bool,
}

fn is_wheel_button(button: MouseButton) -> bool {
    matches!(
        button,
        MouseButton::WheelUp
            | MouseButton::WheelDown
            | MouseButton::WheelLeft
            | MouseButton::WheelRight
    )
}

fn parse_mouse_button(encoded: u16, is_sgr: bool) -> ParsedMouse {
    let mut e = encoded;
    if !is_sgr {
        e = e.saturating_sub(32);
    }

    const BIT_SHIFT: u16 = 0b0000_0100;
    const BIT_ALT: u16 = 0b0000_1000;
    const BIT_CTRL: u16 = 0b0001_0000;
    const BIT_MOTION: u16 = 0b0010_0000;
    const BIT_WHEEL: u16 = 0b0100_0000;
    const BIT_ADD: u16 = 0b1000_0000;
    const BITS_MASK: u16 = 0b0000_0011;

    let mut action = MouseAction::Press;
    let button = if e & BIT_ADD != 0 {
        match e & BITS_MASK {
            0 => MouseButton::Backward,
            1 => MouseButton::Forward,
            2 => MouseButton::Button10,
            _ => MouseButton::Button11,
        }
    } else if e & BIT_WHEEL != 0 {
        match e & BITS_MASK {
            0 => MouseButton::WheelUp,
            1 => MouseButton::WheelDown,
            2 => MouseButton::WheelLeft,
            _ => MouseButton::WheelRight,
        }
    } else {
        match e & BITS_MASK {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            _ => {
                action = MouseAction::Release;
                MouseButton::None
            }
        }
    };

    if e & BIT_MOTION != 0 && !is_wheel_button(button) {
        action = MouseAction::Motion;
    }

    ParsedMouse {
        button,
        action,
        shift: e & BIT_SHIFT != 0,
        alt: e & BIT_ALT != 0,
        ctrl: e & BIT_CTRL != 0,
    }
}

fn parse_x10_mouse_event(buf: &[u8]) -> Result<MouseMsg, MouseParseError> {
    if buf.len() < 6 {
        return Err(MouseParseError::InvalidFormat);
    }

    let parsed = parse_mouse_button(buf[3] as u16, false);

    let x = i32::from(buf[4]) - 32 - 1;
    let y = i32::from(buf[5]) - 32 - 1;
    if x < 0 || y < 0 {
        return Err(MouseParseError::CoordinateUnderflow);
    }

    Ok(MouseMsg {
        x: x as u16,
        y: y as u16,
        shift: parsed.shift,
        alt: parsed.alt,
        ctrl: parsed.ctrl,
        action: parsed.action,
        button: parsed.button,
    })
}

fn parse_sgr_mouse_event(buf: &[u8]) -> Result<MouseMsg, MouseParseError> {
    if !buf.starts_with(b"\x1b[<") {
        return Err(MouseParseError::InvalidFormat);
    }

    let mut nums = [0u16; 3];
    let mut idx = 0usize;
    let mut current: u16 = 0;
    let mut has_digit = false;
    let mut release = false;

    for &b in &buf[3..] {
        match b {
            b'0'..=b'9' => {
                current = current
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(u16::from(b - b'0')))
                    .ok_or(MouseParseError::InvalidNumber)?;
                has_digit = true;
            }
            b';' => {
                if !has_digit || idx >= nums.len() {
                    return Err(MouseParseError::InvalidFormat);
                }
                nums[idx] = current;
                idx += 1;
                current = 0;
                has_digit = false;
            }
            b'M' | b'm' => {
                if !has_digit || idx != 2 {
                    return Err(MouseParseError::InvalidFormat);
                }
                nums[idx] = current;
                release = b == b'm';
                break;
            }
            _ => return Err(MouseParseError::InvalidFormat),
        }
    }

    let mut parsed = parse_mouse_button(nums[0], true);
    if release && parsed.action != MouseAction::Motion && !is_wheel_button(parsed.button) {
        parsed.action = MouseAction::Release;
    }

    if nums[1] == 0 || nums[2] == 0 {
        return Err(MouseParseError::CoordinateUnderflow);
    }

    Ok(MouseMsg {
        x: nums[1] - 1,
        y: nums[2] - 1,
        shift: parsed.shift,
        alt: parsed.alt,
        ctrl: parsed.ctrl,
        action: parsed.action,
        button: parsed.button,
    })
}

/// Parse an ANSI mouse escape sequence into a [`MouseMsg`].
pub fn parse_mouse_event_sequence(buf: &[u8]) -> Result<MouseMsg, MouseParseError> {
    if buf.starts_with(b"\x1b[<") {
        parse_sgr_mouse_event(buf)
    } else if buf.starts_with(b"\x1b[M") {
        parse_x10_mouse_event(buf)
    } else {
        Err(MouseParseError::UnsupportedSequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_from_crossterm_mouse_drag_maps_to_motion_with_button() {
        use crossterm::event::{KeyModifiers, MouseButton as CtButton, MouseEvent, MouseEventKind};

        let event = MouseEvent {
            kind: MouseEventKind::Drag(CtButton::Left),
            column: 12,
            row: 34,
            modifiers: KeyModifiers::empty(),
        };

        let msg = from_crossterm_mouse(event);
        assert_eq!(msg.x, 12);
        assert_eq!(msg.y, 34);
        assert_eq!(msg.action, MouseAction::Motion);
        assert_eq!(msg.button, MouseButton::Left);
        assert!(!msg.shift);
        assert!(!msg.alt);
        assert!(!msg.ctrl);
    }

    #[test]
    fn test_from_crossterm_mouse_moved_maps_to_motion_without_button() {
        use crossterm::event::{KeyModifiers, MouseEvent, MouseEventKind};

        let event = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 1,
            row: 2,
            modifiers: KeyModifiers::empty(),
        };

        let msg = from_crossterm_mouse(event);
        assert_eq!(msg.action, MouseAction::Motion);
        assert_eq!(msg.button, MouseButton::None);
    }

    fn sgr_sequence_bytes(encoded: u16, x: u16, y: u16, release: bool) -> Vec<u8> {
        let suffix = if release { 'm' } else { 'M' };
        format!("\x1b[<{};{};{}{}", encoded, x, y, suffix).into_bytes()
    }

    fn expected_sgr_mouse(encoded: u16, x: u16, y: u16, release: bool) -> MouseMsg {
        let mut parsed = parse_mouse_button(encoded, true);
        if release && parsed.action != MouseAction::Motion && !is_wheel_button(parsed.button) {
            parsed.action = MouseAction::Release;
        }
        MouseMsg {
            x: x - 1,
            y: y - 1,
            shift: parsed.shift,
            alt: parsed.alt,
            ctrl: parsed.ctrl,
            action: parsed.action,
            button: parsed.button,
        }
    }

    fn x10_sequence_bytes(encoded: u8, x: u16, y: u16) -> [u8; 6] {
        let x_byte = (x + 33) as u8;
        let y_byte = (y + 33) as u8;
        [0x1b, b'[', b'M', encoded, x_byte, y_byte]
    }

    fn expected_x10_mouse(encoded: u8, x: u16, y: u16) -> MouseMsg {
        let parsed = parse_mouse_button(u16::from(encoded), false);
        MouseMsg {
            x,
            y,
            shift: parsed.shift,
            alt: parsed.alt,
            ctrl: parsed.ctrl,
            action: parsed.action,
            button: parsed.button,
        }
    }

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
        assert_eq!(mouse.to_string(), "left press");

        let mouse = MouseMsg {
            x: 10,
            y: 20,
            shift: false,
            alt: false,
            ctrl: true,
            action: MouseAction::Press,
            button: MouseButton::Left,
        };
        assert_eq!(mouse.to_string(), "ctrl+left press");
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

    proptest! {
        #[test]
        fn prop_parse_sgr_mouse_roundtrip(
            encoded in 0u16..=255,
            x in 1u16..=2000,
            y in 1u16..=2000,
            release in any::<bool>(),
        ) {
            let buf = sgr_sequence_bytes(encoded, x, y, release);
            let msg = parse_mouse_event_sequence(&buf).unwrap();
            let expected = expected_sgr_mouse(encoded, x, y, release);
            prop_assert_eq!(msg, expected);
        }

        #[test]
        fn prop_parse_x10_mouse_roundtrip(
            encoded in 32u8..=255,
            x in 0u16..=222,
            y in 0u16..=222,
        ) {
            let buf = x10_sequence_bytes(encoded, x, y);
            let msg = parse_mouse_event_sequence(&buf).unwrap();
            let expected = expected_x10_mouse(encoded, x, y);
            prop_assert_eq!(msg, expected);
        }
    }
}
