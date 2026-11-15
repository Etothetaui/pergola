//! Pergola core: a pure, testable fractal layout engine.
//!
//! Nothing in this crate touches X11, the filesystem or the clock, so the whole
//! model can be exercised in unit tests and in `pergola sim`.

pub mod camera;
pub mod command;
pub mod document;
pub mod engine;
pub mod geom;
pub mod node;

pub use camera::{Camera, CameraAction, CameraState};
pub use command::{Command, SettingsPane};
pub use document::{fractal_tree, FloatingWindow, LayoutDocument, LayoutFile, PergolaMeta, SeatRecord};
pub use engine::{GapHandle, LayoutEngine};
pub use geom::{Axis, Direction, IRect, Rect, Size};
pub use node::{normalize_weights, Desktop, LayoutError, LayoutNode, Panel, Split};

/// Crate version, as compiled in.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
