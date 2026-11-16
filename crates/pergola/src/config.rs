//! Configuration and paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// What to do with a window the daemon has not seen before.
///
/// `off` leaves it floating (the historical default), `new` docks windows that
/// appear after the daemon started, and `all` also adopts the windows that were
/// already on screen when it started. The old boolean form still parses: `false`
/// is `off` and `true` is `all`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoDock {
    Off,
    New,
    All,
}

impl Default for AutoDock {
    fn default() -> Self {
        Self::Off
    }
}

impl AutoDock {
    /// Whether a window that appears now should be docked. `after_startup` is
    /// false only during the daemon's first reconcile, so `new` leaves the
    /// user's existing session alone and `all` adopts it.
    pub fn docks(self, after_startup: bool) -> bool {
        match self {
            AutoDock::Off => false,
            AutoDock::New => after_startup,
            AutoDock::All => true,
        }
    }
}

impl Serialize for AutoDock {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(match self {
            AutoDock::Off => "off",
            AutoDock::New => "new",
            AutoDock::All => "all",
        })
    }
}

impl<'de> Deserialize<'de> for AutoDock {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = AutoDock;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a boolean or one of \"off\", \"new\", \"all\"")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> std::result::Result<AutoDock, E> {
                Ok(if v { AutoDock::All } else { AutoDock::Off })
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> std::result::Result<AutoDock, E> {
                match v.to_ascii_lowercase().as_str() {
                    "off" | "false" | "none" | "never" => Ok(AutoDock::Off),
                    "new" => Ok(AutoDock::New),
                    "all" | "true" | "on" | "everything" => Ok(AutoDock::All),
                    other => Err(E::unknown_variant(other, &["off", "new", "all"])),
                }
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Gap between tiles, in pixels.
    pub gap: f64,
    /// Smallest share a tile may be squeezed to.
    pub minimum_share: f64,
    /// Dock new windows automatically instead of leaving them floating.
    pub auto_dock: AutoDock,
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
            auto_dock: AutoDock::Off,
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
    fn auto_dock_is_a_tri_state_and_still_reads_the_old_boolean() {
        // The table the backlog asks for: each mode against a window that was
        // already on screen at startup and one that appears afterwards.
        let cases = [
            (AutoDock::Off, false, false, "off never docks"),
            (AutoDock::Off, true, false, "off never docks, even later"),
            (AutoDock::New, false, false, "new leaves the startup session alone"),
            (AutoDock::New, true, true, "new docks a window opened later"),
            (AutoDock::All, false, true, "all adopts the startup session"),
            (AutoDock::All, true, true, "all docks a window opened later"),
        ];
        for (mode, after_startup, want, why) in cases {
            assert_eq!(mode.docks(after_startup), want, "{why}");
        }
        // Back-compat: the boolean form keeps meaning what it meant.
        let old: Config = serde_json::from_str(r#"{"auto_dock":true}"#).unwrap();
        assert_eq!(old.auto_dock, AutoDock::All);
        let off: Config = serde_json::from_str(r#"{"auto_dock":false}"#).unwrap();
        assert_eq!(off.auto_dock, AutoDock::Off);
        // The new string form round-trips.
        for (text, want) in [("off", AutoDock::Off), ("new", AutoDock::New), ("all", AutoDock::All)] {
            let c: Config = serde_json::from_str(&format!(r#"{{"auto_dock":"{text}"}}"#)).unwrap();
            assert_eq!(c.auto_dock, want);
        }
        let encoded = serde_json::to_string(&Config::default()).unwrap();
        assert!(encoded.contains(r#""auto_dock":"off""#), "{encoded}");
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

