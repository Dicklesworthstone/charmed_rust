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
    /// Shift+Enter.
    ShiftEnter = -54,
    /// Ctrl+Enter.
    CtrlEnter = -55,
    /// Ctrl+Shift+Enter.
    CtrlShiftEnter = -56,
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
            KeyType::ShiftEnter => "shift+enter",
            KeyType::CtrlEnter => "ctrl+enter",
            KeyType::CtrlShiftEnter => "ctrl+shift+enter",
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
pub fn from_crossterm_key(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> KeyMsg {
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
                _ => {
                    return KeyMsg {
                        key_type: KeyType::Runes,
                        runes: vec![c],
                        alt,
                        paste: false,
                    };
                }
            };
            (kt, Vec::new())
        }
        KeyCode::Char(' ') => (KeyType::Space, Vec::new()),
        KeyCode::Char(c) => (KeyType::Runes, vec![c]),
        KeyCode::Enter if ctrl && shift => (KeyType::CtrlShiftEnter, Vec::new()),
        KeyCode::Enter if ctrl => (KeyType::CtrlEnter, Vec::new()),
        KeyCode::Enter if shift => (KeyType::ShiftEnter, Vec::new()),
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

/// Parse a raw ANSI escape sequence into a KeyMsg.
///
/// This function parses terminal escape sequences (like arrow keys, function keys,
/// etc.) into their corresponding KeyMsg values. It matches the behavior of the
/// Go bubbletea library's sequence parsing.
///
/// # Arguments
///
/// * `input` - A byte slice containing an ANSI escape sequence
///
/// # Returns
///
/// Returns `Some(KeyMsg)` if the sequence was recognized, `None` otherwise.
///
/// # Example
///
/// ```rust
/// use bubbletea::{parse_sequence, KeyType};
///
/// // Parse arrow up sequence
/// let key = parse_sequence(b"\x1b[A").unwrap();
/// assert_eq!(key.key_type, KeyType::Up);
/// assert!(!key.alt);
/// ```
pub fn parse_sequence(input: &[u8]) -> Option<KeyMsg> {
    // Convert to string for easier matching
    let seq = std::str::from_utf8(input).ok()?;

    // Try to match known sequences (longest match first approach like Go)
    SEQUENCES.get(seq).cloned()
}

/// Parse a raw ANSI escape sequence from the start of the input.
///
/// This function attempts to find the longest known ANSI escape sequence
/// that matches the beginning of the input slice.
///
/// # Arguments
///
/// * `input` - A byte slice
///
/// # Returns
///
/// Returns `Some((KeyMsg, usize))` where usize is the number of bytes consumed,
/// or `None` if no sequence matched the start of the input.
///
/// # Example
///
/// ```rust
/// use bubbletea::{parse_sequence_prefix, KeyType};
///
/// let input = b"\x1b[A\x1b[B";
/// let (key, len) = parse_sequence_prefix(input).unwrap();
/// assert_eq!(key.key_type, KeyType::Up);
/// assert_eq!(len, 3);
/// ```
pub fn parse_sequence_prefix(input: &[u8]) -> Option<(KeyMsg, usize)> {
    let s = std::str::from_utf8(input).ok()?;

    // We need to find the longest sequence that 's' starts with.
    // Iterating the whole map is O(N), where N is ~100. This is fast enough.
    let mut best_match: Option<(KeyMsg, usize)> = None;

    for (seq, key) in SEQUENCES.iter() {
        if s.starts_with(seq) {
            let len = seq.len();
            match best_match {
                None => best_match = Some((key.clone(), len)),
                Some((_, best_len)) => {
                    if len > best_len {
                        best_match = Some((key.clone(), len));
                    }
                }
            }
        }
    }

    best_match
}

/// Check if the input bytes form a prefix of any known ANSI sequence.
///
/// # Arguments
///
/// * `input` - A byte slice
///
/// # Returns
///
/// Returns `true` if the input is a prefix of a known sequence, `false` otherwise.
pub fn is_sequence_prefix(input: &[u8]) -> bool {
    let s = match std::str::from_utf8(input) {
        Ok(s) => s,
        Err(_) => return false,
    };

    SEQUENCES.keys().any(|seq| seq.starts_with(s))
}

use std::collections::HashMap;
use std::sync::LazyLock;

/// Mapping of ANSI escape sequences to KeyMsg values.
/// This matches the Go bubbletea library's `sequences` map.
static SEQUENCES: LazyLock<HashMap<&'static str, KeyMsg>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Arrow keys
    m.insert("\x1b[A", KeyMsg::from_type(KeyType::Up));
    m.insert("\x1b[B", KeyMsg::from_type(KeyType::Down));
    m.insert("\x1b[C", KeyMsg::from_type(KeyType::Right));
    m.insert("\x1b[D", KeyMsg::from_type(KeyType::Left));

    // Shift + Arrow keys
    m.insert("\x1b[1;2A", KeyMsg::from_type(KeyType::ShiftUp));
    m.insert("\x1b[1;2B", KeyMsg::from_type(KeyType::ShiftDown));
    m.insert("\x1b[1;2C", KeyMsg::from_type(KeyType::ShiftRight));
    m.insert("\x1b[1;2D", KeyMsg::from_type(KeyType::ShiftLeft));
    // DECCKM variants
    m.insert("\x1b[OA", KeyMsg::from_type(KeyType::ShiftUp));
    m.insert("\x1b[OB", KeyMsg::from_type(KeyType::ShiftDown));
    m.insert("\x1b[OC", KeyMsg::from_type(KeyType::ShiftRight));
    m.insert("\x1b[OD", KeyMsg::from_type(KeyType::ShiftLeft));
    // urxvt variants
    m.insert("\x1b[a", KeyMsg::from_type(KeyType::ShiftUp));
    m.insert("\x1b[b", KeyMsg::from_type(KeyType::ShiftDown));
    m.insert("\x1b[c", KeyMsg::from_type(KeyType::ShiftRight));
    m.insert("\x1b[d", KeyMsg::from_type(KeyType::ShiftLeft));

    // Alt + Arrow keys
    m.insert("\x1b[1;3A", KeyMsg::from_type(KeyType::Up).with_alt());
    m.insert("\x1b[1;3B", KeyMsg::from_type(KeyType::Down).with_alt());
    m.insert("\x1b[1;3C", KeyMsg::from_type(KeyType::Right).with_alt());
    m.insert("\x1b[1;3D", KeyMsg::from_type(KeyType::Left).with_alt());

    // Alt + Shift + Arrow keys
    m.insert("\x1b[1;4A", KeyMsg::from_type(KeyType::ShiftUp).with_alt());
    m.insert(
        "\x1b[1;4B",
        KeyMsg::from_type(KeyType::ShiftDown).with_alt(),
    );
    m.insert(
        "\x1b[1;4C",
        KeyMsg::from_type(KeyType::ShiftRight).with_alt(),
    );
    m.insert(
        "\x1b[1;4D",
        KeyMsg::from_type(KeyType::ShiftLeft).with_alt(),
    );

    // Ctrl + Arrow keys
    m.insert("\x1b[1;5A", KeyMsg::from_type(KeyType::CtrlUp));
    m.insert("\x1b[1;5B", KeyMsg::from_type(KeyType::CtrlDown));
    m.insert("\x1b[1;5C", KeyMsg::from_type(KeyType::CtrlRight));
    m.insert("\x1b[1;5D", KeyMsg::from_type(KeyType::CtrlLeft));
    // urxvt Ctrl+Arrow variants (with Alt)
    m.insert("\x1b[Oa", KeyMsg::from_type(KeyType::CtrlUp).with_alt());
    m.insert("\x1b[Ob", KeyMsg::from_type(KeyType::CtrlDown).with_alt());
    m.insert("\x1b[Oc", KeyMsg::from_type(KeyType::CtrlRight).with_alt());
    m.insert("\x1b[Od", KeyMsg::from_type(KeyType::CtrlLeft).with_alt());

    // Ctrl + Shift + Arrow keys
    m.insert("\x1b[1;6A", KeyMsg::from_type(KeyType::CtrlShiftUp));
    m.insert("\x1b[1;6B", KeyMsg::from_type(KeyType::CtrlShiftDown));
    m.insert("\x1b[1;6C", KeyMsg::from_type(KeyType::CtrlShiftRight));
    m.insert("\x1b[1;6D", KeyMsg::from_type(KeyType::CtrlShiftLeft));

    // Ctrl + Alt + Arrow keys
    m.insert("\x1b[1;7A", KeyMsg::from_type(KeyType::CtrlUp).with_alt());
    m.insert("\x1b[1;7B", KeyMsg::from_type(KeyType::CtrlDown).with_alt());
    m.insert(
        "\x1b[1;7C",
        KeyMsg::from_type(KeyType::CtrlRight).with_alt(),
    );
    m.insert("\x1b[1;7D", KeyMsg::from_type(KeyType::CtrlLeft).with_alt());

    // Ctrl + Shift + Alt + Arrow keys
    m.insert(
        "\x1b[1;8A",
        KeyMsg::from_type(KeyType::CtrlShiftUp).with_alt(),
    );
    m.insert(
        "\x1b[1;8B",
        KeyMsg::from_type(KeyType::CtrlShiftDown).with_alt(),
    );
    m.insert(
        "\x1b[1;8C",
        KeyMsg::from_type(KeyType::CtrlShiftRight).with_alt(),
    );
    m.insert(
        "\x1b[1;8D",
        KeyMsg::from_type(KeyType::CtrlShiftLeft).with_alt(),
    );

    // Shift+Tab
    m.insert("\x1b[Z", KeyMsg::from_type(KeyType::ShiftTab));

    // Insert
    m.insert("\x1b[2~", KeyMsg::from_type(KeyType::Insert));
    m.insert("\x1b[3;2~", KeyMsg::from_type(KeyType::Insert).with_alt());

    // Delete
    m.insert("\x1b[3~", KeyMsg::from_type(KeyType::Delete));
    m.insert("\x1b[3;3~", KeyMsg::from_type(KeyType::Delete).with_alt());

    // Page Up
    m.insert("\x1b[5~", KeyMsg::from_type(KeyType::PgUp));
    m.insert("\x1b[5;3~", KeyMsg::from_type(KeyType::PgUp).with_alt());
    m.insert("\x1b[5;5~", KeyMsg::from_type(KeyType::CtrlPgUp));
    m.insert("\x1b[5^", KeyMsg::from_type(KeyType::CtrlPgUp)); // urxvt
    m.insert("\x1b[5;7~", KeyMsg::from_type(KeyType::CtrlPgUp).with_alt());

    // Page Down
    m.insert("\x1b[6~", KeyMsg::from_type(KeyType::PgDown));
    m.insert("\x1b[6;3~", KeyMsg::from_type(KeyType::PgDown).with_alt());
    m.insert("\x1b[6;5~", KeyMsg::from_type(KeyType::CtrlPgDown));
    m.insert("\x1b[6^", KeyMsg::from_type(KeyType::CtrlPgDown)); // urxvt
    m.insert(
        "\x1b[6;7~",
        KeyMsg::from_type(KeyType::CtrlPgDown).with_alt(),
    );

    // Home
    m.insert("\x1b[1~", KeyMsg::from_type(KeyType::Home));
    m.insert("\x1b[H", KeyMsg::from_type(KeyType::Home)); // xterm, lxterm
    m.insert("\x1b[1;3H", KeyMsg::from_type(KeyType::Home).with_alt());
    m.insert("\x1b[1;5H", KeyMsg::from_type(KeyType::CtrlHome));
    m.insert("\x1b[1;7H", KeyMsg::from_type(KeyType::CtrlHome).with_alt());
    m.insert("\x1b[1;2H", KeyMsg::from_type(KeyType::ShiftHome));
    m.insert(
        "\x1b[1;4H",
        KeyMsg::from_type(KeyType::ShiftHome).with_alt(),
    );
    m.insert("\x1b[1;6H", KeyMsg::from_type(KeyType::CtrlShiftHome));
    m.insert(
        "\x1b[1;8H",
        KeyMsg::from_type(KeyType::CtrlShiftHome).with_alt(),
    );
    m.insert("\x1b[7~", KeyMsg::from_type(KeyType::Home)); // urxvt
    m.insert("\x1b[7^", KeyMsg::from_type(KeyType::CtrlHome)); // urxvt
    m.insert("\x1b[7$", KeyMsg::from_type(KeyType::ShiftHome)); // urxvt
    m.insert("\x1b[7@", KeyMsg::from_type(KeyType::CtrlShiftHome)); // urxvt

    // End
    m.insert("\x1b[4~", KeyMsg::from_type(KeyType::End));
    m.insert("\x1b[F", KeyMsg::from_type(KeyType::End)); // xterm, lxterm
    m.insert("\x1b[1;3F", KeyMsg::from_type(KeyType::End).with_alt());
    m.insert("\x1b[1;5F", KeyMsg::from_type(KeyType::CtrlEnd));
    m.insert("\x1b[1;7F", KeyMsg::from_type(KeyType::CtrlEnd).with_alt());
    m.insert("\x1b[1;2F", KeyMsg::from_type(KeyType::ShiftEnd));
    m.insert("\x1b[1;4F", KeyMsg::from_type(KeyType::ShiftEnd).with_alt());
    m.insert("\x1b[1;6F", KeyMsg::from_type(KeyType::CtrlShiftEnd));
    m.insert(
        "\x1b[1;8F",
        KeyMsg::from_type(KeyType::CtrlShiftEnd).with_alt(),
    );
    m.insert("\x1b[8~", KeyMsg::from_type(KeyType::End)); // urxvt
    m.insert("\x1b[8^", KeyMsg::from_type(KeyType::CtrlEnd)); // urxvt
    m.insert("\x1b[8$", KeyMsg::from_type(KeyType::ShiftEnd)); // urxvt
    m.insert("\x1b[8@", KeyMsg::from_type(KeyType::CtrlShiftEnd)); // urxvt

    // Function keys - Linux console
    m.insert("\x1b[[A", KeyMsg::from_type(KeyType::F1));
    m.insert("\x1b[[B", KeyMsg::from_type(KeyType::F2));
    m.insert("\x1b[[C", KeyMsg::from_type(KeyType::F3));
    m.insert("\x1b[[D", KeyMsg::from_type(KeyType::F4));
    m.insert("\x1b[[E", KeyMsg::from_type(KeyType::F5));

    // Function keys - VT100/xterm F1-F4
    m.insert("\x1bOP", KeyMsg::from_type(KeyType::F1));
    m.insert("\x1bOQ", KeyMsg::from_type(KeyType::F2));
    m.insert("\x1bOR", KeyMsg::from_type(KeyType::F3));
    m.insert("\x1bOS", KeyMsg::from_type(KeyType::F4));

    // Function keys - VT100/xterm F1-F4 with Alt
    m.insert("\x1b[1;3P", KeyMsg::from_type(KeyType::F1).with_alt());
    m.insert("\x1b[1;3Q", KeyMsg::from_type(KeyType::F2).with_alt());
    m.insert("\x1b[1;3R", KeyMsg::from_type(KeyType::F3).with_alt());
    m.insert("\x1b[1;3S", KeyMsg::from_type(KeyType::F4).with_alt());

    // Function keys - urxvt F1-F4
    m.insert("\x1b[11~", KeyMsg::from_type(KeyType::F1));
    m.insert("\x1b[12~", KeyMsg::from_type(KeyType::F2));
    m.insert("\x1b[13~", KeyMsg::from_type(KeyType::F3));
    m.insert("\x1b[14~", KeyMsg::from_type(KeyType::F4));

    // Function keys F5-F12
    m.insert("\x1b[15~", KeyMsg::from_type(KeyType::F5));
    m.insert("\x1b[15;3~", KeyMsg::from_type(KeyType::F5).with_alt());
    m.insert("\x1b[17~", KeyMsg::from_type(KeyType::F6));
    m.insert("\x1b[17;3~", KeyMsg::from_type(KeyType::F6).with_alt());
    m.insert("\x1b[18~", KeyMsg::from_type(KeyType::F7));
    m.insert("\x1b[18;3~", KeyMsg::from_type(KeyType::F7).with_alt());
    m.insert("\x1b[19~", KeyMsg::from_type(KeyType::F8));
    m.insert("\x1b[19;3~", KeyMsg::from_type(KeyType::F8).with_alt());
    m.insert("\x1b[20~", KeyMsg::from_type(KeyType::F9));
    m.insert("\x1b[20;3~", KeyMsg::from_type(KeyType::F9).with_alt());
    m.insert("\x1b[21~", KeyMsg::from_type(KeyType::F10));
    m.insert("\x1b[21;3~", KeyMsg::from_type(KeyType::F10).with_alt());
    m.insert("\x1b[23~", KeyMsg::from_type(KeyType::F11));
    m.insert("\x1b[23;3~", KeyMsg::from_type(KeyType::F11).with_alt());
    m.insert("\x1b[24~", KeyMsg::from_type(KeyType::F12));
    m.insert("\x1b[24;3~", KeyMsg::from_type(KeyType::F12).with_alt());

    // Function keys F13-F16
    m.insert("\x1b[1;2P", KeyMsg::from_type(KeyType::F13));
    m.insert("\x1b[1;2Q", KeyMsg::from_type(KeyType::F14));
    m.insert("\x1b[25~", KeyMsg::from_type(KeyType::F13));
    m.insert("\x1b[26~", KeyMsg::from_type(KeyType::F14));
    m.insert("\x1b[25;3~", KeyMsg::from_type(KeyType::F13).with_alt());
    m.insert("\x1b[26;3~", KeyMsg::from_type(KeyType::F14).with_alt());
    m.insert("\x1b[1;2R", KeyMsg::from_type(KeyType::F15));
    m.insert("\x1b[1;2S", KeyMsg::from_type(KeyType::F16));
    m.insert("\x1b[28~", KeyMsg::from_type(KeyType::F15));
    m.insert("\x1b[29~", KeyMsg::from_type(KeyType::F16));
    m.insert("\x1b[28;3~", KeyMsg::from_type(KeyType::F15).with_alt());
    m.insert("\x1b[29;3~", KeyMsg::from_type(KeyType::F16).with_alt());

    // Function keys F17-F20
    m.insert("\x1b[15;2~", KeyMsg::from_type(KeyType::F17));
    m.insert("\x1b[17;2~", KeyMsg::from_type(KeyType::F18));
    m.insert("\x1b[18;2~", KeyMsg::from_type(KeyType::F19));
    m.insert("\x1b[19;2~", KeyMsg::from_type(KeyType::F20));
    m.insert("\x1b[31~", KeyMsg::from_type(KeyType::F17));
    m.insert("\x1b[32~", KeyMsg::from_type(KeyType::F18));
    m.insert("\x1b[33~", KeyMsg::from_type(KeyType::F19));
    m.insert("\x1b[34~", KeyMsg::from_type(KeyType::F20));

    // PowerShell sequences
    m.insert("\x1bOA", KeyMsg::from_type(KeyType::Up));
    m.insert("\x1bOB", KeyMsg::from_type(KeyType::Down));
    m.insert("\x1bOC", KeyMsg::from_type(KeyType::Right));
    m.insert("\x1bOD", KeyMsg::from_type(KeyType::Left));

    m
});

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn sequence_strategy() -> impl Strategy<Value = &'static str> {
        let sequences: Vec<&'static str> = SEQUENCES.keys().copied().collect();
        prop::sample::select(sequences)
    }

    #[test]
    fn test_parse_sequence_arrows() {
        assert_eq!(
            parse_sequence(b"\x1b[A"),
            Some(KeyMsg::from_type(KeyType::Up))
        );
        assert_eq!(
            parse_sequence(b"\x1b[B"),
            Some(KeyMsg::from_type(KeyType::Down))
        );
        assert_eq!(
            parse_sequence(b"\x1b[C"),
            Some(KeyMsg::from_type(KeyType::Right))
        );
        assert_eq!(
            parse_sequence(b"\x1b[D"),
            Some(KeyMsg::from_type(KeyType::Left))
        );
    }

    #[test]
    fn test_parse_sequence_alt_arrows() {
        assert_eq!(
            parse_sequence(b"\x1b[1;3A"),
            Some(KeyMsg::from_type(KeyType::Up).with_alt())
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;3B"),
            Some(KeyMsg::from_type(KeyType::Down).with_alt())
        );
    }

    #[test]
    fn test_parse_sequence_ctrl_arrows() {
        assert_eq!(
            parse_sequence(b"\x1b[1;5A"),
            Some(KeyMsg::from_type(KeyType::CtrlUp))
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;5B"),
            Some(KeyMsg::from_type(KeyType::CtrlDown))
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;5C"),
            Some(KeyMsg::from_type(KeyType::CtrlRight))
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;5D"),
            Some(KeyMsg::from_type(KeyType::CtrlLeft))
        );
    }

    #[test]
    fn test_parse_sequence_ctrl_alt_arrows() {
        // Ctrl+Alt+Arrow keys (modifier 7)
        assert_eq!(
            parse_sequence(b"\x1b[1;7A"),
            Some(KeyMsg::from_type(KeyType::CtrlUp).with_alt())
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;7B"),
            Some(KeyMsg::from_type(KeyType::CtrlDown).with_alt())
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;7C"),
            Some(KeyMsg::from_type(KeyType::CtrlRight).with_alt())
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;7D"),
            Some(KeyMsg::from_type(KeyType::CtrlLeft).with_alt())
        );

        // Verify alt flag is properly set
        let key = parse_sequence(b"\x1b[1;7A").unwrap();
        assert!(key.alt);
        assert_eq!(key.key_type, KeyType::CtrlUp);
    }

    #[test]
    fn test_parse_sequence_ctrl_shift_alt_arrows() {
        // Ctrl+Shift+Alt+Arrow keys (modifier 8)
        assert_eq!(
            parse_sequence(b"\x1b[1;8A"),
            Some(KeyMsg::from_type(KeyType::CtrlShiftUp).with_alt())
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;8B"),
            Some(KeyMsg::from_type(KeyType::CtrlShiftDown).with_alt())
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;8C"),
            Some(KeyMsg::from_type(KeyType::CtrlShiftRight).with_alt())
        );
        assert_eq!(
            parse_sequence(b"\x1b[1;8D"),
            Some(KeyMsg::from_type(KeyType::CtrlShiftLeft).with_alt())
        );
    }

    #[test]
    fn test_parse_sequence_function_keys() {
        assert_eq!(
            parse_sequence(b"\x1bOP"),
            Some(KeyMsg::from_type(KeyType::F1))
        );
        assert_eq!(
            parse_sequence(b"\x1b[15~"),
            Some(KeyMsg::from_type(KeyType::F5))
        );
        assert_eq!(
            parse_sequence(b"\x1b[24~"),
            Some(KeyMsg::from_type(KeyType::F12))
        );
    }

    #[test]
    fn test_parse_sequence_special_keys() {
        assert_eq!(
            parse_sequence(b"\x1b[Z"),
            Some(KeyMsg::from_type(KeyType::ShiftTab))
        );
        assert_eq!(
            parse_sequence(b"\x1b[2~"),
            Some(KeyMsg::from_type(KeyType::Insert))
        );
        assert_eq!(
            parse_sequence(b"\x1b[3~"),
            Some(KeyMsg::from_type(KeyType::Delete))
        );
        assert_eq!(
            parse_sequence(b"\x1b[5~"),
            Some(KeyMsg::from_type(KeyType::PgUp))
        );
        assert_eq!(
            parse_sequence(b"\x1b[6~"),
            Some(KeyMsg::from_type(KeyType::PgDown))
        );
    }

    #[test]
    fn test_parse_sequence_home_end() {
        assert_eq!(
            parse_sequence(b"\x1b[H"),
            Some(KeyMsg::from_type(KeyType::Home))
        );
        assert_eq!(
            parse_sequence(b"\x1b[1~"),
            Some(KeyMsg::from_type(KeyType::Home))
        );
        assert_eq!(
            parse_sequence(b"\x1b[F"),
            Some(KeyMsg::from_type(KeyType::End))
        );
        assert_eq!(
            parse_sequence(b"\x1b[4~"),
            Some(KeyMsg::from_type(KeyType::End))
        );
    }

    #[test]
    fn test_parse_sequence_unknown() {
        assert_eq!(parse_sequence(b"unknown"), None);
        assert_eq!(parse_sequence(b"\x1b[999~"), None);
    }

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

    proptest! {
        #[test]
        fn prop_parse_sequence_matches_table(seq in sequence_strategy()) {
            let expected = SEQUENCES.get(seq).cloned();
            prop_assert_eq!(parse_sequence(seq.as_bytes()), expected);
        }
    }
}
