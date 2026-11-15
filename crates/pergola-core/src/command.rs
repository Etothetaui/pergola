//! The command vocabulary, and the `pergola://` URL form.
//!
//! The names mirror the public TrellisWM guide (see `docs/RESEARCH.md`) so the
//! port can be checked against the evidence set; the scheme is Pergola's own.

use crate::camera::CameraAction;
use crate::geom::{Direction, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPane {
    General,
    Windows,
    Tabs,
    Appearance,
    Gestures,
    Apps,
    Shortcuts,
    License,
}

impl SettingsPane {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "general" => Self::General,
            "windows" => Self::Windows,
            "tabs" => Self::Tabs,
            "appearance" => Self::Appearance,
            "gestures" => Self::Gestures,
            "apps" => Self::Apps,
            "shortcuts" => Self::Shortcuts,
            "license" => Self::License,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Windows => "windows",
            Self::Tabs => "tabs",
            Self::Appearance => "appearance",
            Self::Gestures => "gestures",
            Self::Apps => "apps",
            Self::Shortcuts => "shortcuts",
            Self::License => "license",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Everything,
    ZoomOut,
    ZoomIn,
    Back,
    Forward,
    Neighbour(Direction),
    FrameWindow,
    FrameDesktop,
    ToggleDesktop,
    View(u8),
    SaveView(u8),
    /// Split the focused window with a new terminal (the Linux reading of
    /// "split with a new Finder window").
    SplitTerminal,
    Arrange,
    FloatAll,
    Settings(SettingsPane),
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Everything => "everything",
            Self::ZoomOut => "zoom-out",
            Self::ZoomIn => "zoom-in",
            Self::Back => "back",
            Self::Forward => "forward",
            Self::Neighbour(_) => "neighbour",
            Self::FrameWindow => "frame-window",
            Self::FrameDesktop => "frame-desktop",
            Self::ToggleDesktop => "toggle-desktop",
            Self::View(_) => "view",
            Self::SaveView(_) => "save-view",
            Self::SplitTerminal => "split-terminal",
            Self::Arrange => "arrange",
            Self::FloatAll => "float-all",
            Self::Settings(_) => "settings",
        }
    }

    /// Parse `pergola://zoom-in`, `pergola://view?n=2`, or a bare command word.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let rest = raw
            .strip_prefix("pergola://")
            .or_else(|| raw.strip_prefix("pergola:"))
            .unwrap_or(raw);
        let (name, query) = match rest.split_once('?') {
            Some((n, q)) => (n, q),
            None => (rest, ""),
        };
        let mut params: Vec<(String, String)> = Vec::new();
        for pair in query.split('&').filter(|p| !p.is_empty()) {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            params.push((k.to_ascii_lowercase(), v.to_ascii_lowercase()));
        }
        let get = |key: &str| {
            params
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        let slot = |key: &str| -> Result<u8, String> {
            let raw = get(key).ok_or_else(|| format!("{name} requires {key}=1..9"))?;
            let n: u8 = raw.parse().map_err(|_| format!("{key} must be 1..9"))?;
            if !(1..=9).contains(&n) {
                return Err(format!("{key} must be 1..9"));
            }
            Ok(n)
        };
        Ok(match name.to_ascii_lowercase().as_str() {
            "everything" => Self::Everything,
            "zoom-out" => Self::ZoomOut,
            "zoom-in" => Self::ZoomIn,
            "back" => Self::Back,
            "forward" => Self::Forward,
            "neighbour" => {
                let dir = get("dir")
                    .and_then(Direction::parse)
                    .ok_or("neighbour requires dir=left|right|up|down")?;
                Self::Neighbour(dir)
            }
            "frame-window" => Self::FrameWindow,
            "frame-desktop" => Self::FrameDesktop,
            "toggle-desktop" => Self::ToggleDesktop,
            "view" => Self::View(slot("n")?),
            "save-view" => Self::SaveView(slot("n")?),
            "split-terminal" | "split-finder" => Self::SplitTerminal,
            "arrange" => Self::Arrange,
            "float-all" => Self::FloatAll,
            "settings" => Self::Settings(
                get("pane")
                    .and_then(SettingsPane::parse)
                    .unwrap_or(SettingsPane::General),
            ),
            other => return Err(format!("unknown command '{other}'")),
        })
    }

    pub fn to_url(&self) -> String {
        match self {
            Self::Neighbour(d) => format!("pergola://neighbour?dir={}", dir_name(*d)),
            Self::View(n) => format!("pergola://view?n={n}"),
            Self::SaveView(n) => format!("pergola://save-view?n={n}"),
            Self::Settings(p) => format!("pergola://settings?pane={}", p.as_str()),
            other => format!("pergola://{}", other.name()),
        }
    }

    /// Commands that are pure camera moves. The rest need the window system.
    pub fn camera_action(&self, focused_node: Option<&str>, frame: Rect) -> Option<CameraAction> {
        Some(match self {
            Self::Everything => CameraAction::Everything,
            Self::ZoomOut => CameraAction::ZoomOut,
            Self::ZoomIn => CameraAction::ZoomIn { toward_seat: None },
            Self::Back => CameraAction::Back,
            Self::Forward => CameraAction::Forward,
            Self::FrameWindow => CameraAction::Frame {
                node_id: focused_node?.to_string(),
            },
            Self::FrameDesktop => CameraAction::FrameDesktop,
            Self::ToggleDesktop => CameraAction::ToggleDesktop,
            Self::View(n) => CameraAction::GoToView(*n),
            Self::SaveView(n) => CameraAction::SaveView(*n),
            Self::Neighbour(d) => CameraAction::Neighbour {
                direction: *d,
                frame,
            },
            _ => return None,
        })
    }
}

fn dir_name(d: Direction) -> &'static str {
    match d {
        Direction::Left => "left",
        Direction::Right => "right",
        Direction::Up => "up",
        Direction::Down => "down",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_urls_and_bare_words() {
        assert_eq!(Command::parse("pergola://everything").unwrap(), Command::Everything);
        assert_eq!(Command::parse("zoom-in").unwrap(), Command::ZoomIn);
        assert_eq!(
            Command::parse("pergola://neighbour?dir=left").unwrap(),
            Command::Neighbour(Direction::Left)
        );
        assert_eq!(Command::parse("pergola://view?n=4").unwrap(), Command::View(4));
        assert_eq!(
            Command::parse("pergola://settings?pane=tabs").unwrap(),
            Command::Settings(SettingsPane::Tabs)
        );
        assert!(Command::parse("pergola://view?n=0").is_err());
        assert!(Command::parse("pergola://nope").is_err());
    }

    #[test]
    fn urls_round_trip() {
        for c in [
            Command::Everything,
            Command::Neighbour(Direction::Up),
            Command::View(9),
            Command::SaveView(2),
            Command::Settings(SettingsPane::Shortcuts),
            Command::SplitTerminal,
        ] {
            assert_eq!(Command::parse(&c.to_url()).unwrap(), c);
        }
    }

    #[test]
    fn camera_actions_only_for_camera_commands() {
        let f = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(Command::Arrange.camera_action(None, f).is_none());
        assert!(Command::FrameWindow.camera_action(None, f).is_none());
        assert!(Command::FrameWindow.camera_action(Some("p-a"), f).is_some());
    }
}
