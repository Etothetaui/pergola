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

/// Every key `pergola config` knows about, in the order it prints them.
pub const KNOWN_KEYS: &[&str] = &[
    "gap",
    "minimum_share",
    "auto_dock",
    "overlay",
    "desktop_share",
    "terminal",
    "hotkeys",
    "edge_dock",
    "bindings",
];

/// One complaint about `config.json`, tied to the line that caused it when the
/// problem comes from the file (a missing terminal is not "." a line's fault).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.line > 0 {
            write!(f, "line {}: {}", self.line, self.message)
        } else {
            f.write_str(&self.message)
        }
    }
}

/// What `pergola config` shows: the resolved values, where each one came from,
/// and everything `--check` would reject.
pub struct Report {
    pub path: PathBuf,
    /// Whether the file existed at all. Missing is not a problem: every key has a
    /// default.
    pub found: bool,
    pub config: Config,
    /// 1-based line of each key that appears in the file. Absent keys are simply
    /// not in the map.
    pub lines: HashMap<&'static str, usize>,
    pub issues: Vec<Issue>,
}

impl Report {
    /// The 1-based line a key appears on, or 0 when it does not (a default).
    pub fn line_of(&self, key: &str) -> usize {
        self.lines.get(key).copied().unwrap_or(0)
    }

    /// `default` or `config.json:<line>`, the source of one key.
    pub fn source_of(&self, key: &str) -> String {
        let line = self.line_of(key);
        if line == 0 {
            "default".to_string()
        } else {
            format!("config.json:{line}")
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

    /// Resolve and validate `config.json` without applying it, for
    /// `pergola config` and `pergola config --check`.
    pub fn inspect() -> Report {
        inspect_at(&config_path())
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

/// Parse and validate `path` the way the daemon would, collecting the problems
/// instead of printing them. A syntax error yields the defaults plus one issue,
/// exactly what `Config::load` falls back to.
fn inspect_at(path: &Path) -> Report {
    let mut report = Report {
        path: path.to_path_buf(),
        found: false,
        config: Config::default(),
        lines: HashMap::new(),
        issues: Vec::new(),
    };
    let raw = match std::fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return report,
        Err(e) => {
            report.issues.push(Issue {
                line: 0,
                message: format!("cannot read {}: {e}", path.display()),
            });
            return report;
        }
    };
    report.found = true;
    if raw.trim().is_empty() {
        return report;
    }
    let value: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(e) => {
            report.issues.push(Issue {
                line: e.line(),
                message: format!("not valid JSON: {e}"),
            });
            return report;
        }
    };
    let Some(object) = value.as_object() else {
        report.issues.push(Issue {
            line: 1,
            message: "the whole file must be a JSON object".to_string(),
        });
        return report;
    };
    let key_lines = top_level_key_lines(&raw);
    for key in KNOWN_KEYS {
        if object.contains_key(*key) {
            if let Some(line) = key_lines.get(*key) {
                report.lines.insert(key, *line);
            }
        }
    }
    for key in object.keys() {
        if !KNOWN_KEYS.contains(&key.as_str()) {
            report.issues.push(Issue {
                line: key_lines.get(key).copied().unwrap_or(0),
                message: format!(
                    "unknown key \"{key}\" (known: {})",
                    KNOWN_KEYS.join(", ")
                ),
            });
        }
    }
    report.config = match serde_json::from_str::<Config>(&raw) {
        Ok(config) => config,
        Err(e) => {
            report.issues.push(Issue {
                line: e.line(),
                message: e.to_string(),
            });
            Config::default()
        }
    };
    check_ranges(&report.config, &report.lines, &mut report.issues);
    check_bindings(&report.config, &report.lines, &mut report.issues);
    report
}

fn check_ranges(config: &Config, lines: &HashMap<&'static str, usize>, issues: &mut Vec<Issue>) {
    let mut complain = |key: &'static str, message: &str| {
        issues.push(Issue {
            line: lines.get(key).copied().unwrap_or(0),
            message: message.to_string(),
        });
    };
    if !(0.0..=400.0).contains(&config.gap) {
        complain("gap", "gap must be between 0 and 400 pixels");
    }
    if !(config.minimum_share > 0.0 && config.minimum_share <= 0.5) {
        complain(
            "minimum_share",
            "minimum_share must be greater than 0 and at most 0.5",
        );
    }
    if !(0.0..1.0).contains(&config.desktop_share) {
        complain("desktop_share", "desktop_share must be at least 0 and less than 1");
    }
    if !terminal_exists(&config.terminal) {
        complain(
            "terminal",
            &format!(
                "terminal \"{}\" is not on PATH and is not an executable file",
                config.terminal
            ),
        );
    }
}

fn check_bindings(config: &Config, lines: &HashMap<&'static str, usize>, issues: &mut Vec<Issue>) {
    let line = lines.get("bindings").copied().unwrap_or(0);
    for (command, chords) in &config.bindings {
        if command.trim().is_empty() {
            issues.push(Issue {
                line,
                message: "bindings has an empty command name".to_string(),
            });
        }
        for chord in chords {
            if pergola_x11::parse_binding(chord).is_none() {
                issues.push(Issue {
                    line,
                    message: format!(
                        "bindings: \"{chord}\" for \"{command}\" is not a chord I understand"
                    ),
                });
            }
        }
    }
}

fn terminal_exists(term: &str) -> bool {
    if term.trim().is_empty() {
        return false;
    }
    let path = Path::new(term);
    if path.is_absolute() || term.contains('/') {
        return path.is_file();
    }
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(term).is_file()))
        .unwrap_or(false)
}

/// The 1-based line each *top-level* key of the object starts on, whether the
/// file is pretty-printed or one long line. Strings and the braces of nested
/// objects are tracked, so `{"bindings": {"gap": …}}` does not report the
/// binding's line for the top-level `gap` key.
fn top_level_key_lines(raw: &str) -> HashMap<String, usize> {
    let mut lines: HashMap<String, usize> = HashMap::new();
    let mut depth = 0i32;
    let mut line = 1usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut reading_key = false;
    let mut pending_key: Option<(String, usize)> = None;
    let mut expect_key = false;
    let mut key = String::new();
    let mut key_line = 1usize;
    for ch in raw.chars() {
        if ch == '\n' {
            line += 1;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
                if reading_key {
                    reading_key = false;
                    pending_key = Some((key.clone(), key_line));
                }
            } else if reading_key {
                key.push(ch);
            }
            continue;
        }
        match ch {
            '"' => {
                in_string = true;
                if depth == 1 && expect_key {
                    reading_key = true;
                    key.clear();
                    key_line = line;
                }
            }
            '{' | '[' => {
                depth += 1;
                expect_key = ch == '{' && depth == 1;
            }
            '}' | ']' => {
                depth -= 1;
                expect_key = false;
            }
            ':' => {
                if let Some((finished, at_line)) = pending_key.take() {
                    lines.entry(finished).or_insert(at_line);
                }
                expect_key = false;
            }
            ',' => {
                expect_key = depth == 1;
            }
            _ => {}
        }
    }
    lines
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

    fn issues_of(raw: &str, name: &str) -> Report {
        let path = scratch(name);
        std::fs::write(&path, raw).unwrap();
        inspect_at(&path)
    }

    #[test]
    fn a_missing_file_is_defaults_with_no_complaint() {
        let path = scratch("absent");
        let report = inspect_at(&path);
        assert!(!report.found);
        assert!(report.issues.is_empty(), "{:#?}", report.issues);
        assert_eq!(report.config.gap, Config::default().gap);
        assert_eq!(report.line_of("gap"), 0);
        assert_eq!(report.source_of("gap"), "default");
    }

    #[test]
    fn a_good_file_resolves_every_key_and_names_its_lines() {
        let report = issues_of(
            "{\n  \"gap\": 9,\n  \"minimum_share\": 0.1,\n  \"auto_dock\": \"new\",\n  \"overlay\": true,\n  \"desktop_share\": 0.2,\n  \"terminal\": \"/bin/sh\",\n  \"hotkeys\": false,\n  \"edge_dock\": false,\n  \"bindings\": {\"view 3\": [\"Super+9\"]}\n}\n",
            "good",
        );
        assert!(report.issues.is_empty(), "{:#?}", report.issues);
        assert_eq!(report.config.gap, 9.0);
        assert_eq!(report.config.auto_dock, AutoDock::New);
        assert!(report.config.overlay);
        assert!(!report.config.hotkeys);
        assert!(!report.config.edge_dock);
        assert_eq!(report.config.terminal, "/bin/sh");
        assert_eq!(report.config.bindings["view 3"], vec!["Super+9"]);
        assert_eq!(report.line_of("gap"), 2);
        assert_eq!(report.source_of("gap"), "config.json:2");
    }

    #[test]
    fn an_unknown_key_is_reported_on_its_line() {
        let report = issues_of("{\n  \"gap\": 6,\n  \"gaap\": 12\n}\n", "unknown");
        assert_eq!(report.issues.len(), 1, "{:#?}", report.issues);
        assert_eq!(report.issues[0].line, 3);
        assert!(report.issues[0].message.contains("unknown key \"gaap\""));
        // A typo does not stop the rest of the file from being used.
        assert_eq!(report.config.gap, 6.0);
    }

    #[test]
    fn a_binding_named_after_a_key_does_not_shadow_the_key_line() {
        let report = issues_of(
            "{\n  \"gap\": 6,\n  \"bindings\": {\"gap\": [\"Super+g\"]}\n}\n",
            "shadow",
        );
        assert_eq!(report.line_of("gap"), 2);
        assert!(report.issues.is_empty(), "{:#?}", report.issues);
    }

    #[test]
    fn a_one_line_file_still_names_its_source() {
        // A compact config is one line, so every key is on line 1 — but they must
        // not be reported as defaults, which would be a lie about where the
        // value came from.
        let report = issues_of(
            "{\"bindings\": {\"gap\": [\"Super+g\"]}, \"gap\": 9}",
            "compact",
        );
        assert_eq!(report.config.gap, 9.0);
        assert_eq!(report.line_of("gap"), 1);
        assert_eq!(report.source_of("gap"), "config.json:1");
        assert!(report.issues.is_empty(), "{:#?}", report.issues);
    }

    #[test]
    fn out_of_range_values_are_rejected() {
        let report = issues_of(
            "{\n  \"gap\": -4,\n  \"minimum_share\": 0,\n  \"desktop_share\": 1.0\n}\n",
            "ranges",
        );
        let messages: Vec<&str> = report.issues.iter().map(|i| i.message.as_str()).collect();
        assert!(messages.iter().any(|m| m.contains("gap must be between 0 and 400")), "{messages:#?}");
        assert!(messages.iter().any(|m| m.contains("minimum_share")), "{messages:#?}");
        assert!(messages.iter().any(|m| m.contains("desktop_share")), "{messages:#?}");
        assert_eq!(report.issues[0].line, 2);
    }

    #[test]
    fn a_terminal_that_does_not_exist_is_rejected() {
        let report = issues_of("{\"terminal\": \"/nonexistent/term\"}\n", "terminal");
        assert_eq!(report.issues.len(), 1, "{:#?}", report.issues);
        assert!(report.issues[0].message.contains("not on PATH"));
        // An absolute path that does exist is fine.
        let report = issues_of("{\"terminal\": \"/bin/sh\"}\n", "terminal-ok");
        assert!(report.issues.is_empty(), "{:#?}", report.issues);
    }

    #[test]
    fn a_chord_that_does_not_parse_is_rejected() {
        let report = issues_of(
            "{\"bindings\": {\"view 2\": [\"Super+bracketleft\", \"Super+;;\"]}}\n",
            "chords",
        );
        assert_eq!(report.issues.len(), 1, "{:#?}", report.issues);
        assert!(report.issues[0].message.contains("is not a chord I understand"));
    }

    #[test]
    fn a_syntax_error_is_reported_without_losing_the_defaults() {
        let report = issues_of("{\n  \"gap\": 6,\n  \"overlay\": tru\n}\n", "syntax");
        assert_eq!(report.issues.len(), 1, "{:#?}", report.issues);
        assert!(report.issues[0].message.contains("not valid JSON"));
        assert_eq!(report.config.gap, Config::default().gap);
    }
}

