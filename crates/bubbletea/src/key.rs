//! Keyboard input handling.
//!
//! This module provides types for representing keyboard events, including
//! special keys, control combinations, and regular character input.

use std::fmt;

/// Keyboard key event message.
///
/// KeyMsg is sent to the program's update function when a key is pressed.
///
/// # Example
///
/// ```rust
/// use bubbletea::{KeyMsg, KeyType};
///
/// fn handle_key(key: KeyMsg) {
///     match key.key_type {
///         KeyType::Enter => println!("Enter pressed"),
///         KeyType::Runes => println!("Typed: {:?}", key.runes),
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMsg {
    /// The type of key pressed.
    pub key_type: KeyType,
    /// For KeyType::Runes, the characters typed.
    pub runes: Vec<char>,
    /// Whether Alt was held.
    pub alt: bool,
    /// Whether this came from a paste operation.
    pub paste: bool,
}

impl KeyMsg {
    /// Create a new key message from a key type.
    pub fn from_type(key_type: KeyType) -> Self {
        Self {
            key_type,
            runes: Vec::new(),
            alt: false,
            paste: false,
        }
    }

    /// Create a new key message from a character.
    pub fn from_char(c: char) -> Self {
        Self {
            key_type: KeyType::Runes,
            runes: vec![c],
            alt: false,
            paste: false,
        }
    }

    /// Create a new key message from multiple characters (e.g., from IME).
    pub fn from_runes(runes: Vec<char>) -> Self {
        Self {
            key_type: KeyType::Runes,
            runes,
            alt: false,
            paste: false,
        }
    }

    /// Set the alt modifier.
    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    /// Set the paste flag.
    pub fn with_paste(mut self) -> Self {
        self.paste = true;
        self
    }
}

impl fmt::Display for KeyMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.alt {
            write!(f, "alt+")?;
        }
        if self.key_type == KeyType::Runes {
            if self.paste {
                write!(f, "[")?;
            }
            for c in &self.runes {
                write!(f, "{}", c)?;
            }
            if self.paste {
                write!(f, "]")?;
            }
        } else {
            write!(f, "{}", self.key_type)?;
        }
        Ok(())
    }
}

/// Key type enumeration.
///
/// Represents different types of keys that can be pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i16)]
pub enum KeyType {
    // Control keys (ASCII values)
    /// Null character (Ctrl+@).
    Null = 0,
    /// Ctrl+A.
    CtrlA = 1,
    /// Ctrl+B.
    CtrlB = 2,
    /// Break/Interrupt (Ctrl+C).
    CtrlC = 3,
    /// Ctrl+D (EOF).
    CtrlD = 4,
    /// Ctrl+E.
    CtrlE = 5,
    /// Ctrl+F.
    CtrlF = 6,
    /// Ctrl+G (Bell).
    CtrlG = 7,
    /// Ctrl+H (Backspace on some systems).
    CtrlH = 8,
    /// Tab (Ctrl+I).
    Tab = 9,
    /// Ctrl+J (Line feed).
    CtrlJ = 10,
    /// Ctrl+K.
    CtrlK = 11,
    /// Ctrl+L.
    CtrlL = 12,
    /// Enter (Ctrl+M, Carriage return).
    Enter = 13,
    /// Ctrl+N.
    CtrlN = 14,
    /// Ctrl+O.
    CtrlO = 15,
    /// Ctrl+P.
    CtrlP = 16,
    /// Ctrl+Q.
    CtrlQ = 17,
    /// Ctrl+R.
    CtrlR = 18,
    /// Ctrl+S.
    CtrlS = 19,
    /// Ctrl+T.
    CtrlT = 20,
    /// Ctrl+U.
    CtrlU = 21,
    /// Ctrl+V.
    CtrlV = 22,
    /// Ctrl+W.
    CtrlW = 23,
    /// Ctrl+X.
    CtrlX = 24,
    /// Ctrl+Y.
    CtrlY = 25,
    /// Ctrl+Z.
    CtrlZ = 26,
    /// Escape (Ctrl+[).
    Esc = 27,
    /// Ctrl+\.
    CtrlBackslash = 28,
    /// Ctrl+].
    CtrlCloseBracket = 29,
    /// Ctrl+^.
    CtrlCaret = 30,
    /// Ctrl+_.
    CtrlUnderscore = 31,
    /// Delete (127).
    Backspace = 127,

    // Special keys (negative values to avoid collision)
    /// Regular character(s) input.
    Runes = -1,
    /// Up arrow.
    Up = -2,
    /// Down arrow.
    Down = -3,
    /// Right arrow.
    Right = -4,
    /// Left arrow.
    Left = -5,
    /// Shift+Tab.
    ShiftTab = -6,
    /// Home key.
    Home = -7,
    /// End key.
    End = -8,
    /// Page Up.
    PgUp = -9,
    /// Page Down.
    PgDown = -10,
    /// Ctrl+Page Up.
    CtrlPgUp = -11,
    /// Ctrl+Page Down.
    CtrlPgDown = -12,
    /// Delete key.
    Delete = -13,
    /// Insert key.
    Insert = -14,
    /// Space key.
    Space = -15,
    /// Ctrl+Up.
    CtrlUp = -16,
    /// Ctrl+Down.
    CtrlDown = -17,
    /// Ctrl+Right.
    CtrlRight = -18,
    /// Ctrl+Left.
    CtrlLeft = -19,
    /// Ctrl+Home.
    CtrlHome = -20,
    /// Ctrl+End.
    CtrlEnd = -21,
    /// Shift+Up.
    ShiftUp = -22,
    /// Shift+Down.
    ShiftDown = -23,
    /// Shift+Right.
    ShiftRight = -24,
    /// Shift+Left.
    ShiftLeft = -25,
    /// Shift+Home.
    ShiftHome = -26,
    /// Shift+End.
    ShiftEnd = -27,
    /// Ctrl+Shift+Up.
    CtrlShiftUp = -28,
    /// Ctrl+Shift+Down.
    CtrlShiftDown = -29,
    /// Ctrl+Shift+Left.
    CtrlShiftLeft = -30,
    /// Ctrl+Shift+Right.
    CtrlShiftRight = -31,
    /// Ctrl+Shift+Home.
    CtrlShiftHome = -32,
    /// Ctrl+Shift+End.
    CtrlShiftEnd = -33,
    /// F1.
    F1 = -34,
    /// F2.
    F2 = -35,
    /// F3.
    F3 = -36,
    /// F4.
    F4 = -37,
    /// F5.
    F5 = -38,
    /// F6.
    F6 = -39,
    /// F7.
    F7 = -40,
    /// F8.
    F8 = -41,
    /// F9.
    F9 = -42,
    /// F10.
    F10 = -43,
    /// F11.
    F11 = -44,
    /// F12.
    F12 = -45,
    /// F13.
    F13 = -46,
    /// F14.
    F14 = -47,
    /// F15.
    F15 = -48,
    /// F16.
    F16 = -49,
    /// F17.
    F17 = -50,
    /// F18.
    F18 = -51,
    /// F19.
    F19 = -52,
    /// F20.
    F20 = -53,
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            KeyType::Null => "ctrl+@",
            KeyType::CtrlA => "ctrl+a",
            KeyType::CtrlB => "ctrl+b",
            KeyType::CtrlC => "ctrl+c",
            KeyType::CtrlD => "ctrl+d",
            KeyType::CtrlE => "ctrl+e",
            KeyType::CtrlF => "ctrl+f",
            KeyType::CtrlG => "ctrl+g",
            KeyType::CtrlH => "ctrl+h",
            KeyType::Tab => "tab",
            KeyType::CtrlJ => "ctrl+j",
            KeyType::CtrlK => "ctrl+k",
            KeyType::CtrlL => "ctrl+l",
            KeyType::Enter => "enter",
            KeyType::CtrlN => "ctrl+n",
            KeyType::CtrlO => "ctrl+o",
            KeyType::CtrlP => "ctrl+p",
            KeyType::CtrlQ => "ctrl+q",
            KeyType::CtrlR => "ctrl+r",
            KeyType::CtrlS => "ctrl+s",
            KeyType::CtrlT => "ctrl+t",
            KeyType::CtrlU => "ctrl+u",
            KeyType::CtrlV => "ctrl+v",
            KeyType::CtrlW => "ctrl+w",
            KeyType::CtrlX => "ctrl+x",
            KeyType::CtrlY => "ctrl+y",
            KeyType::CtrlZ => "ctrl+z",
            KeyType::Esc => "esc",
            KeyType::CtrlBackslash => "ctrl+\\",
            KeyType::CtrlCloseBracket => "ctrl+]",
            KeyType::CtrlCaret => "ctrl+^",
            KeyType::CtrlUnderscore => "ctrl+_",
            KeyType::Backspace => "backspace",
            KeyType::Runes => "runes",
            KeyType::Up => "up",
            KeyType::Down => "down",
            KeyType::Right => "right",
            KeyType::Left => "left",
            KeyType::ShiftTab => "shift+tab",
            KeyType::Home => "home",
            KeyType::End => "end",
            KeyType::PgUp => "pgup",
            KeyType::PgDown => "pgdown",
            KeyType::CtrlPgUp => "ctrl+pgup",
            KeyType::CtrlPgDown => "ctrl+pgdown",
            KeyType::Delete => "delete",
            KeyType::Insert => "insert",
            KeyType::Space => " ",
            KeyType::CtrlUp => "ctrl+up",
            KeyType::CtrlDown => "ctrl+down",
            KeyType::CtrlRight => "ctrl+right",
            KeyType::CtrlLeft => "ctrl+left",
            KeyType::CtrlHome => "ctrl+home",
            KeyType::CtrlEnd => "ctrl+end",
            KeyType::ShiftUp => "shift+up",
            KeyType::ShiftDown => "shift+down",
            KeyType::ShiftRight => "shift+right",
            KeyType::ShiftLeft => "shift+left",
            KeyType::ShiftHome => "shift+home",
            KeyType::ShiftEnd => "shift+end",
            KeyType::CtrlShiftUp => "ctrl+shift+up",
            KeyType::CtrlShiftDown => "ctrl+shift+down",
            KeyType::CtrlShiftLeft => "ctrl+shift+left",
            KeyType::CtrlShiftRight => "ctrl+shift+right",
            KeyType::CtrlShiftHome => "ctrl+shift+home",
            KeyType::CtrlShiftEnd => "ctrl+shift+end",
            KeyType::F1 => "f1",
            KeyType::F2 => "f2",
            KeyType::F3 => "f3",
            KeyType::F4 => "f4",
            KeyType::F5 => "f5",
            KeyType::F6 => "f6",
            KeyType::F7 => "f7",
            KeyType::F8 => "f8",
            KeyType::F9 => "f9",
            KeyType::F10 => "f10",
            KeyType::F11 => "f11",
            KeyType::F12 => "f12",
            KeyType::F13 => "f13",
            KeyType::F14 => "f14",
            KeyType::F15 => "f15",
            KeyType::F16 => "f16",
            KeyType::F17 => "f17",
            KeyType::F18 => "f18",
            KeyType::F19 => "f19",
            KeyType::F20 => "f20",
        };
        write!(f, "{}", name)
    }
}

impl KeyType {
    /// Check if this key type represents a control character.
    pub fn is_ctrl(&self) -> bool {
        let val = *self as i16;
        (0..=31).contains(&val) || val == 127
    }

    /// Check if this is a function key (F1-F20).
    pub fn is_function_key(&self) -> bool {
        matches!(
            self,
            KeyType::F1
                | KeyType::F2
                | KeyType::F3
                | KeyType::F4
                | KeyType::F5
                | KeyType::F6
                | KeyType::F7
                | KeyType::F8
                | KeyType::F9
                | KeyType::F10
                | KeyType::F11
                | KeyType::F12
                | KeyType::F13
                | KeyType::F14
                | KeyType::F15
                | KeyType::F16
                | KeyType::F17
                | KeyType::F18
                | KeyType::F19
                | KeyType::F20
        )
    }

    /// Check if this is a cursor movement key.
    pub fn is_cursor(&self) -> bool {
        matches!(
            self,
            KeyType::Up
                | KeyType::Down
                | KeyType::Left
                | KeyType::Right
                | KeyType::Home
                | KeyType::End
                | KeyType::PgUp
                | KeyType::PgDown
                | KeyType::CtrlUp
                | KeyType::CtrlDown
                | KeyType::CtrlLeft
                | KeyType::CtrlRight
                | KeyType::CtrlHome
                | KeyType::CtrlEnd
                | KeyType::ShiftUp
                | KeyType::ShiftDown
                | KeyType::ShiftLeft
                | KeyType::ShiftRight
                | KeyType::ShiftHome
                | KeyType::ShiftEnd
                | KeyType::CtrlShiftUp
                | KeyType::CtrlShiftDown
                | KeyType::CtrlShiftLeft
                | KeyType::CtrlShiftRight
                | KeyType::CtrlShiftHome
                | KeyType::CtrlShiftEnd
                | KeyType::CtrlPgUp
                | KeyType::CtrlPgDown
        )
    }
}

/// Convert a crossterm KeyCode to our KeyType.
pub fn from_crossterm_key(code: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> KeyMsg {
    use crossterm::event::{KeyCode, KeyModifiers};

    let ctrl = modifiers.contains(KeyModifiers::CONTROL);
    let shift = modifiers.contains(KeyModifiers::SHIFT);
    let alt = modifiers.contains(KeyModifiers::ALT);

    let (key_type, runes) = match code {
        KeyCode::Char(c) if ctrl => {
            let kt = match c.to_ascii_lowercase() {
                '@' => KeyType::Null,
                'a' => KeyType::CtrlA,
                'b' => KeyType::CtrlB,
                'c' => KeyType::CtrlC,
                'd' => KeyType::CtrlD,
                'e' => KeyType::CtrlE,
                'f' => KeyType::CtrlF,
                'g' => KeyType::CtrlG,
                'h' => KeyType::CtrlH,
                'i' => KeyType::Tab,
                'j' => KeyType::CtrlJ,
                'k' => KeyType::CtrlK,
                'l' => KeyType::CtrlL,
                'm' => KeyType::Enter,
                'n' => KeyType::CtrlN,
                'o' => KeyType::CtrlO,
                'p' => KeyType::CtrlP,
                'q' => KeyType::CtrlQ,
                'r' => KeyType::CtrlR,
                's' => KeyType::CtrlS,
                't' => KeyType::CtrlT,
                'u' => KeyType::CtrlU,
                'v' => KeyType::CtrlV,
                'w' => KeyType::CtrlW,
                'x' => KeyType::CtrlX,
                'y' => KeyType::CtrlY,
                'z' => KeyType::CtrlZ,
                '\\' => KeyType::CtrlBackslash,
                ']' => KeyType::CtrlCloseBracket,
                '^' => KeyType::CtrlCaret,
                '_' => KeyType::CtrlUnderscore,
                _ => return KeyMsg {
                    key_type: KeyType::Runes,
                    runes: vec![c],
                    alt,
                    paste: false,
                },
            };
            (kt, Vec::new())
        }
        KeyCode::Char(' ') => (KeyType::Space, Vec::new()),
        KeyCode::Char(c) => (KeyType::Runes, vec![c]),
        KeyCode::Enter => (KeyType::Enter, Vec::new()),
        KeyCode::Backspace => (KeyType::Backspace, Vec::new()),
        KeyCode::Tab if shift => (KeyType::ShiftTab, Vec::new()),
        KeyCode::Tab => (KeyType::Tab, Vec::new()),
        KeyCode::Esc => (KeyType::Esc, Vec::new()),
        KeyCode::Delete => (KeyType::Delete, Vec::new()),
        KeyCode::Insert => (KeyType::Insert, Vec::new()),
        KeyCode::Up if ctrl && shift => (KeyType::CtrlShiftUp, Vec::new()),
        KeyCode::Up if ctrl => (KeyType::CtrlUp, Vec::new()),
        KeyCode::Up if shift => (KeyType::ShiftUp, Vec::new()),
        KeyCode::Up => (KeyType::Up, Vec::new()),
        KeyCode::Down if ctrl && shift => (KeyType::CtrlShiftDown, Vec::new()),
        KeyCode::Down if ctrl => (KeyType::CtrlDown, Vec::new()),
        KeyCode::Down if shift => (KeyType::ShiftDown, Vec::new()),
        KeyCode::Down => (KeyType::Down, Vec::new()),
        KeyCode::Left if ctrl && shift => (KeyType::CtrlShiftLeft, Vec::new()),
        KeyCode::Left if ctrl => (KeyType::CtrlLeft, Vec::new()),
        KeyCode::Left if shift => (KeyType::ShiftLeft, Vec::new()),
        KeyCode::Left => (KeyType::Left, Vec::new()),
        KeyCode::Right if ctrl && shift => (KeyType::CtrlShiftRight, Vec::new()),
        KeyCode::Right if ctrl => (KeyType::CtrlRight, Vec::new()),
        KeyCode::Right if shift => (KeyType::ShiftRight, Vec::new()),
        KeyCode::Right => (KeyType::Right, Vec::new()),
        KeyCode::Home if ctrl && shift => (KeyType::CtrlShiftHome, Vec::new()),
        KeyCode::Home if ctrl => (KeyType::CtrlHome, Vec::new()),
        KeyCode::Home if shift => (KeyType::ShiftHome, Vec::new()),
        KeyCode::Home => (KeyType::Home, Vec::new()),
        KeyCode::End if ctrl && shift => (KeyType::CtrlShiftEnd, Vec::new()),
        KeyCode::End if ctrl => (KeyType::CtrlEnd, Vec::new()),
        KeyCode::End if shift => (KeyType::ShiftEnd, Vec::new()),
        KeyCode::End => (KeyType::End, Vec::new()),
        KeyCode::PageUp if ctrl => (KeyType::CtrlPgUp, Vec::new()),
        KeyCode::PageUp => (KeyType::PgUp, Vec::new()),
        KeyCode::PageDown if ctrl => (KeyType::CtrlPgDown, Vec::new()),
        KeyCode::PageDown => (KeyType::PgDown, Vec::new()),
        KeyCode::F(1) => (KeyType::F1, Vec::new()),
        KeyCode::F(2) => (KeyType::F2, Vec::new()),
        KeyCode::F(3) => (KeyType::F3, Vec::new()),
        KeyCode::F(4) => (KeyType::F4, Vec::new()),
        KeyCode::F(5) => (KeyType::F5, Vec::new()),
        KeyCode::F(6) => (KeyType::F6, Vec::new()),
        KeyCode::F(7) => (KeyType::F7, Vec::new()),
        KeyCode::F(8) => (KeyType::F8, Vec::new()),
        KeyCode::F(9) => (KeyType::F9, Vec::new()),
        KeyCode::F(10) => (KeyType::F10, Vec::new()),
        KeyCode::F(11) => (KeyType::F11, Vec::new()),
        KeyCode::F(12) => (KeyType::F12, Vec::new()),
        KeyCode::F(13) => (KeyType::F13, Vec::new()),
        KeyCode::F(14) => (KeyType::F14, Vec::new()),
        KeyCode::F(15) => (KeyType::F15, Vec::new()),
        KeyCode::F(16) => (KeyType::F16, Vec::new()),
        KeyCode::F(17) => (KeyType::F17, Vec::new()),
        KeyCode::F(18) => (KeyType::F18, Vec::new()),
        KeyCode::F(19) => (KeyType::F19, Vec::new()),
        KeyCode::F(20) => (KeyType::F20, Vec::new()),
        _ => (KeyType::Runes, Vec::new()),
    };

    KeyMsg {
        key_type,
        runes,
        alt,
        paste: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_msg_display() {
        let key = KeyMsg::from_type(KeyType::Enter);
        assert_eq!(key.to_string(), "enter");

        let key = KeyMsg::from_char('a');
        assert_eq!(key.to_string(), "a");

        let key = KeyMsg::from_char('a').with_alt();
        assert_eq!(key.to_string(), "alt+a");

        let key = KeyMsg::from_runes(vec!['h', 'e', 'l', 'l', 'o']).with_paste();
        assert_eq!(key.to_string(), "[hello]");
    }

    #[test]
    fn test_key_type_display() {
        assert_eq!(KeyType::Enter.to_string(), "enter");
        assert_eq!(KeyType::CtrlC.to_string(), "ctrl+c");
        assert_eq!(KeyType::F1.to_string(), "f1");
    }

    #[test]
    fn test_key_type_is_ctrl() {
        assert!(KeyType::CtrlC.is_ctrl());
        assert!(KeyType::Enter.is_ctrl());
        assert!(!KeyType::Up.is_ctrl());
    }

    #[test]
    fn test_key_type_is_function_key() {
        assert!(KeyType::F1.is_function_key());
        assert!(KeyType::F12.is_function_key());
        assert!(!KeyType::Enter.is_function_key());
    }

    #[test]
    fn test_key_type_is_cursor() {
        assert!(KeyType::Up.is_cursor());
        assert!(KeyType::CtrlLeft.is_cursor());
        assert!(!KeyType::Enter.is_cursor());
    }
}
