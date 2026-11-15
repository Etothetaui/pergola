//! Configuration and paths.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Gap between tiles, in pixels.
    pub gap: f64,
    /// Smallest share a tile may be squeezed to.
    pub minimum_share: f64,
    /// Dock new windows automatically instead of leaving them floating.
    pub auto_dock: bool,
    /// Show the tree overlay when the camera moves.
    pub overlay: bool,
    /// Share of the screen given to the desktop leaf by `arrange`.
    pub desktop_share: f64,
    /// Terminal used by `split-terminal`.
    pub terminal: String,
    /// Grab the default hotkeys on the root window.
    pub hotkeys: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gap: 8.0,
            minimum_share: 0.05,
            auto_dock: false,
            overlay: true,
            desktop_share: 0.0,
            terminal: std::env::var("TERMINAL").unwrap_or_else(|_| "xfce4-terminal".to_string()),
            hotkeys: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|e| {
                eprintln!("pergola: ignoring malformed {}: {e}", path.display());
                Config::default()
            }),
            Err(_) => Config::default(),
        }
    }

}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pergola")
}

pub fn state_dir() -> PathBuf {
    dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("pergola")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn layout_path() -> PathBuf {
    state_dir().join("layout.json")
}

pub fn socket_path() -> PathBuf {
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime).join("pergola.sock")
    } else {
        state_dir().join("pergola.sock")
    }
}

