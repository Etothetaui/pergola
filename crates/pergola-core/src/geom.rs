//! Geometry primitives. Top-left origin, matching X11 and the layout documents
//! Pergola reads and writes.

use serde::{Deserialize, Serialize};

/// A rectangle in floating-point pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn mid_x(&self) -> f64 {
        self.x + self.w / 2.0
    }

    pub fn mid_y(&self) -> f64 {
        self.y + self.h / 2.0
    }

    pub fn max_x(&self) -> f64 {
        self.x + self.w
    }

    pub fn max_y(&self) -> f64 {
        self.y + self.h
    }

    /// Round to whole pixels, never smaller than 1x1.
    pub fn to_irect(self) -> IRect {
        IRect {
            x: self.x.round() as i32,
            y: self.y.round() as i32,
            w: (self.w.round() as i32).max(1),
            h: (self.h.round() as i32).max(1),
        }
    }
}

/// A rectangle in whole pixels, as X11 wants it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl IRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub fn to_rect(self) -> Rect {
        Rect::new(self.x as f64, self.y as f64, self.w as f64, self.h as f64)
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

impl Size {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }
}

/// Split axis. `X` lays children out left to right, `Y` top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    X,
    Y,
}

/// A docking edge, and the direction a neighbour search walks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn axis(self) -> Axis {
        match self {
            Direction::Left | Direction::Right => Axis::X,
            Direction::Up | Direction::Down => Axis::Y,
        }
    }

    /// True for the edges that come before the target on its axis.
    pub fn is_negative(self) -> bool {
        matches!(self, Direction::Left | Direction::Up)
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "left" | "l" | "west" => Some(Direction::Left),
            "right" | "r" | "east" => Some(Direction::Right),
            "up" | "u" | "top" | "north" => Some(Direction::Up),
            "down" | "d" | "bottom" | "south" => Some(Direction::Down),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_never_produces_empty_rects() {
        let r = Rect::new(10.4, 10.6, 0.2, 0.2).to_irect();
        assert_eq!(r, IRect::new(10, 11, 1, 1));
    }

    #[test]
    fn direction_axis_and_sign() {
        assert_eq!(Direction::Left.axis(), Axis::X);
        assert!(Direction::Up.is_negative());
        assert!(!Direction::Right.is_negative());
        assert_eq!(Direction::parse("south"), Some(Direction::Down));
    }
}
