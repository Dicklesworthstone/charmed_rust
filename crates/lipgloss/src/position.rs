//! Position and alignment types.

/// Text alignment position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// Align to the top or left.
    #[default]
    Top,
    /// Align to the bottom or right.
    Bottom,
    /// Align to the center.
    Center,
    /// Alias for Top.
    Left,
    /// Alias for Bottom.
    Right,
}

impl Position {
    /// Convert position to a factor (0.0, 0.5, or 1.0).
    pub fn factor(&self) -> f64 {
        match self {
            Position::Top | Position::Left => 0.0,
            Position::Center => 0.5,
            Position::Bottom | Position::Right => 1.0,
        }
    }
}

/// CSS-like sides specification for padding, margin, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Copy> Sides<T> {
    /// Create sides with all values the same.
    pub const fn all(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create sides from individual values.
    pub const fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl<T: Copy + Default> Sides<T> {
    /// Create sides with zero/default values.
    pub fn zero() -> Self
    where
        T: Default,
    {
        Self::default()
    }
}

// From implementations for CSS-like shorthand

impl<T: Copy> From<T> for Sides<T> {
    /// Single value: all sides.
    fn from(all: T) -> Self {
        Self::all(all)
    }
}

impl<T: Copy> From<(T, T)> for Sides<T> {
    /// Two values: (vertical, horizontal).
    fn from((vertical, horizontal): (T, T)) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl<T: Copy> From<(T, T, T)> for Sides<T> {
    /// Three values: (top, horizontal, bottom).
    fn from((top, horizontal, bottom): (T, T, T)) -> Self {
        Self {
            top,
            right: horizontal,
            bottom,
            left: horizontal,
        }
    }
}

impl<T: Copy> From<(T, T, T, T)> for Sides<T> {
    /// Four values: (top, right, bottom, left) - clockwise.
    fn from((top, right, bottom, left): (T, T, T, T)) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl<T: Copy> From<[T; 1]> for Sides<T> {
    fn from([all]: [T; 1]) -> Self {
        Self::all(all)
    }
}

impl<T: Copy> From<[T; 2]> for Sides<T> {
    fn from([vertical, horizontal]: [T; 2]) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

impl<T: Copy> From<[T; 4]> for Sides<T> {
    fn from([top, right, bottom, left]: [T; 4]) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_factor() {
        assert_eq!(Position::Top.factor(), 0.0);
        assert_eq!(Position::Center.factor(), 0.5);
        assert_eq!(Position::Bottom.factor(), 1.0);
    }

    #[test]
    fn test_sides_from_single() {
        let s: Sides<u16> = 5.into();
        assert_eq!(s.top, 5);
        assert_eq!(s.right, 5);
        assert_eq!(s.bottom, 5);
        assert_eq!(s.left, 5);
    }

    #[test]
    fn test_sides_from_tuple2() {
        let s: Sides<u16> = (5, 10).into();
        assert_eq!(s.top, 5);
        assert_eq!(s.right, 10);
        assert_eq!(s.bottom, 5);
        assert_eq!(s.left, 10);
    }

    #[test]
    fn test_sides_from_tuple4() {
        let s: Sides<u16> = (1, 2, 3, 4).into();
        assert_eq!(s.top, 1);
        assert_eq!(s.right, 2);
        assert_eq!(s.bottom, 3);
        assert_eq!(s.left, 4);
    }
}
