//! Configuration and paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
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
    /// Dock a floating window dropped against a screen edge (mouse tiling).
    pub edge_dock: bool,
    /// Per-command chord overrides: command → candidate chords, tried in order.
    /// A command named here replaces its default candidates entirely (an empty
    /// list unbinds it); commands not mentioned keep the defaults. Any command
    /// the daemon understands may be bound, not only the ones in the default
    /// plan, so `view 3` or `gap 12` can be put on a chord.
    pub bindings: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gap: 6.0,
            minimum_share: 0.05,
            auto_dock: false,
            // The overlay is a debugging aid, not decoration: lines in the gaps
            // and a viewport frame, off unless asked for (or `everything`).
            overlay: false,
            desktop_share: 0.0,
            terminal: std::env::var("TERMINAL").unwrap_or_else(|_| "xfce4-terminal".to_string()),
            hotkeys: true,
            edge_dock: true,
            bindings: HashMap::new(),
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

    /// The default binding plan with this config's per-command overrides
    /// applied. Every command the daemon understands can be bound, not only
    /// the ones in the default plan.
    pub fn binding_plan(&self) -> Vec<(String, Vec<String>)> {
        pergola_x11::binding_plan_with(&self.bindings)
    }
}

/// Write one command's candidate chords into config.json and return the path.
///
/// `Some(list)` replaces the command's candidates (an empty list unbinds it);
/// `None` removes the override so the command falls back to its default plan.
/// Every other key in the file is preserved, including keys this binary does
/// not know about. A file that does not parse as JSON is never rewritten: a
/// regenerated config would silently discard the user's typo and their edits.
pub fn write_bindings(command: &str, chords: Option<&[String]>) -> Result<PathBuf> {
    let path = config_path();
    write_bindings_at(&path, command, chords)?;
    Ok(path)
}

fn write_bindings_at(path: &Path, command: &str, chords: Option<&[String]>) -> Result<()> {
    let mut root: serde_json::Map<String, serde_json::Value> = match std::fs::read_to_string(path) {
        Ok(raw) if raw.trim().is_empty() => serde_json::Map::new(),
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(serde_json::Value::Object(map)) => map,
            _ => anyhow::bail!(
                "refusing to rewrite a malformed {}: fix the JSON first",
                path.display()
            ),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::Map::new(),
        Err(e) => {
            return Err(anyhow::Error::new(e).context(format!("cannot read {}", path.display())))
        }
    };
    let mut bindings = match root.remove("bindings") {
        Some(serde_json::Value::Object(map)) => map,
        Some(_) => anyhow::bail!(
            "refusing to rewrite {}: \"bindings\" is not an object",
            path.display()
        ),
        None => serde_json::Map::new(),
    };
    match chords {
        Some(list) => {
            bindings.insert(command.to_string(), serde_json::json!(list));
        }
        None => {
            bindings.remove(command);
        }
    }
    if !bindings.is_empty() {
        root.insert("bindings".to_string(), serde_json::Value::Object(bindings));
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| anyhow::Error::new(e).context(format!("cannot create {}", dir.display())))?;
    }
    let tmp = path.with_extension("json.tmp");
    let mut text = serde_json::to_string_pretty(&serde_json::Value::Object(root))?;
    text.push('\n');
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pergola-config-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("config.json")
    }

    #[test]
    fn bind_preserves_unknown_keys_and_other_settings() {
        let path = scratch("preserve");
        std::fs::write(&path, r#"{"gap":6.0,"future_option":"keep me"}"#).unwrap();
        write_bindings_at(&path, "zoom-out", Some(&["Super+Page_Up".to_string()])).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["gap"], 6.0);
        assert_eq!(value["future_option"], "keep me");
        assert_eq!(value["bindings"]["zoom-out"][0], "Super+Page_Up");
        // The daemon must still load it as a valid Config.
        let config: Config = serde_json::from_str(&raw).unwrap();
        assert_eq!(config.bindings["zoom-out"], vec!["Super+Page_Up"]);
    }

    #[test]
    fn unbind_writes_an_empty_list_and_removing_restores_the_default() {
        let path = scratch("unbind");
        write_bindings_at(&path, "overlay", Some(&[])).unwrap();
        let config: Config = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(config.bindings["overlay"].is_empty());
        write_bindings_at(&path, "overlay", None).unwrap();
        let config: Config = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(!config.bindings.contains_key("overlay"));
    }

    #[test]
    fn malformed_config_is_never_rewritten() {
        let path = scratch("malformed");
        std::fs::write(&path, r#"{"gap": 6.0, "bindings": {"back": ["Super+b"}"#).unwrap();
        let err = write_bindings_at(&path, "back", Some(&["Super+z".to_string()])).unwrap_err();
        assert!(err.to_string().contains("refusing to rewrite a malformed"), "{err}");
        // The user's broken file is still there, untouched.
        assert!(std::fs::read_to_string(&path).unwrap().contains("Super+b"));
    }

    #[test]
    fn creates_the_config_directory_when_missing() {
        let path = scratch("missing").join("nested").join("config.json");
        write_bindings_at(&path, "back", Some(&["Super+b".to_string()])).unwrap();
        assert!(path.exists());
    }
}

