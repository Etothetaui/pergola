//! Configuration and paths.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
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
    /// Strength of the overlay's dim layer, 0 (off) to 0.9. Only used when a
    /// compositor can render `_NET_WM_WINDOW_OPACITY`.
    pub dim: f64,
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
            // 40% black keeps the tiles readable while the split lines stand
            // out; a config key because a bright or a dark desktop wants a
            // different strength.
            dim: 0.4,
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
    "dim",
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
    let mut root = read_object(path)?;
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
    write_object(path, &root)
}

/// The file as a JSON object. A malformed file is never rewritten: a
/// regenerated config would silently discard the user's typo and their edits.
fn read_object(path: &Path) -> Result<serde_json::Map<String, serde_json::Value>> {
    match std::fs::read_to_string(path) {
        Ok(raw) if raw.trim().is_empty() => Ok(serde_json::Map::new()),
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(serde_json::Value::Object(map)) => Ok(map),
            _ => anyhow::bail!(
                "refusing to rewrite a malformed {}: fix the JSON first",
                path.display()
            ),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::Map::new()),
        Err(e) => Err(anyhow::Error::new(e).context(format!("cannot read {}", path.display()))),
    }
}

/// Pretty-print `root` beside `path` and rename it into place, so a reader never
/// sees half a file.
fn write_object(path: &Path, root: &serde_json::Map<String, serde_json::Value>) -> Result<()> {
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir).map_err(|e| {
                anyhow::Error::new(e).context(format!("cannot create {}", dir.display()))
            })?;
        }
    }
    let tmp = path.with_extension("json.tmp");
    let mut text = serde_json::to_string_pretty(&serde_json::Value::Object(root.clone()))?;
    text.push('\n');
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Set one scalar key from the words a person typed (`gap`, `12`). The candidate
/// file is validated with the same checks as `config --check` and only renamed
/// into place when every one passes, so a bad value never reaches the file.
pub fn set_value_at(path: &Path, key: &str, text: &str) -> Result<()> {
    let value = parse_value(key, text)?;
    let before: std::collections::HashSet<String> =
        inspect_at(path).issues.into_iter().map(|i| i.message).collect();
    let mut root = read_object(path)?;
    root.insert(key.to_string(), value);
    let tmp = path.with_extension("json.tmp");
    write_object(&tmp, &root)?;
    // Only problems this change *introduces* block the write. A file that
    // already has a typo elsewhere is still editable; the typo keeps its line
    // and `config --check` still reports it.
    let introduced: Vec<Issue> = inspect_at(&tmp)
        .issues
        .into_iter()
        .filter(|issue| !before.contains(&issue.message))
        .collect();
    if !introduced.is_empty() {
        let _ = std::fs::remove_file(&tmp);
        let problems: Vec<String> = introduced.iter().map(|i| i.to_string()).collect();
        anyhow::bail!("{}", problems.join("; "));
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// The JSON value a typed setting turns into.
fn parse_value(key: &str, text: &str) -> Result<serde_json::Value> {
    let text = text.trim();
    Ok(match key {
        "gap" | "minimum_share" | "desktop_share" | "dim" => {
            let number: f64 = text
                .parse()
                .map_err(|_| anyhow!("{key} must be a number, got {text:?}"))?;
            serde_json::json!(number)
        }
        "overlay" | "hotkeys" | "edge_dock" => serde_json::json!(match text.to_ascii_lowercase().as_str() {
            "on" | "true" | "yes" | "1" => true,
            "off" | "false" | "no" | "0" => false,
            _ => anyhow::bail!("{key} must be on or off, got {text:?}"),
        }),
        "auto_dock" => serde_json::json!(match text.to_ascii_lowercase().as_str() {
            "off" | "false" | "none" | "never" => "off",
            "new" => "new",
            "all" | "true" | "on" | "everything" => "all",
            _ => anyhow::bail!("auto_dock must be off, new or all, got {text:?}"),
        }),
        "terminal" => {
            if text.is_empty() {
                anyhow::bail!("terminal cannot be empty");
            }
            serde_json::json!(text)
        }
        other => anyhow::bail!("{other} is not a setting I can change from the menu"),
    })
}

/// What one row of the settings palette would change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setting {
    /// A scalar key, changed through `pergola settings <key> <value>`.
    Key(&'static str),
    /// A command's chords, changed through `pergola bind <command> <chord>`.
    Chord(String),
}

/// One line of help for a settings row: what the key controls, the range the
/// validator enforces and the compiled default. It is part of the row, so a
/// person does not need the manual to know what `minimum_share = 0.05` means —
/// and a menu that is opened from a chord has no manual next to it.
pub fn help_text(key: &str) -> &'static str {
    match key {
        "gap" => "pixels between tiles; 0 to 400 (default 6)",
        "minimum_share" => "smallest share a tile can be squeezed to; >0 and <=0.5 (default 0.05)",
        "auto_dock" => "off | new | all: tile a window when it appears (default off)",
        "overlay" => "draw split lines in the gaps; on | off (default off)",
        "dim" => "overlay dim strength under a compositor; 0 (off) to 0.9 (default 0.4)",
        "desktop_share" => "screen share the desktop leaf keeps; 0 to <1 (default 0)",
        "terminal" => "program `split-terminal` runs (default $TERMINAL, else xfce4-terminal)",
        "hotkeys" => "grab the default chords at startup; on | off (default on)",
        "edge_dock" => "tile a window dropped on a screen edge; on | off (default on)",
        "bindings" => "per-command chord overrides; change them through the chord rows",
        _ => "a value this build does not explain",
    }
}

/// The rows the settings palette shows: every key with the value in force and
/// one line of help, then every command with the chords actually bound (the
/// `keys --check` view, so a chord the desktop owns is visible before it is
/// edited). The label is the prompt's identity: `run_settings` maps a pick back
/// to its `Setting` by exact match, so labels must stay unique.
pub fn settings_rows(config: &Config) -> Vec<(String, Setting)> {
    let mut rows: Vec<(String, Setting)> = Vec::new();
    for key in KNOWN_KEYS {
        if *key == "bindings" {
            continue;
        }
        rows.push((
            format!("{key} = {} — {}", value_text(config, key), help_text(key)),
            Setting::Key(key),
        ));
    }
    for (command, chords) in pergola_x11::binding_plan_with(&config.bindings) {
        let shown = if chords.is_empty() {
            "off".to_string()
        } else {
            chords.join(", ")
        };
        rows.push((
            format!("chord: {command} = {shown} — the chords that run it; a comma makes several"),
            Setting::Chord(command),
        ));
    }
    rows
}

/// The value one key has, as the palette and `settings --list` print it.
pub fn value_text(config: &Config, key: &str) -> String {
    match key {
        "gap" => format!("{}", config.gap),
        "minimum_share" => format!("{}", config.minimum_share),
        "auto_dock" => match config.auto_dock {
            AutoDock::Off => "off".to_string(),
            AutoDock::New => "new".to_string(),
            AutoDock::All => "all".to_string(),
        },
        "overlay" => config.overlay.to_string(),
        "dim" => format!("{}", config.dim),
        "desktop_share" => format!("{}", config.desktop_share),
        "terminal" => config.terminal.clone(),
        "hotkeys" => config.hotkeys.to_string(),
        "edge_dock" => config.edge_dock.to_string(),
        "bindings" => {
            if config.bindings.is_empty() {
                "{}".to_string()
            } else {
                format!("{} command(s)", config.bindings.len())
            }
        }
        other => format!("<{other}>"),
    }
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
    if !(0.0..=0.9).contains(&config.dim) {
        complain("dim", "dim must be at least 0 and at most 0.9");
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
    fn setting_a_key_writes_it_and_keeps_every_other_key() {
        let path = scratch("set");
        std::fs::write(&path, "{\"gap\": 6, \"gaap\": 12, \"overlay\": false}\n").unwrap();
        set_value_at(&path, "gap", "12").unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["gap"], serde_json::json!(12.0));
        // The typo is still there, at its line, for `--check` to report.
        assert_eq!(value["gaap"], serde_json::json!(12));
        assert_eq!(value["overlay"], serde_json::json!(false));
    }

    #[test]
    fn a_bad_value_is_rejected_and_the_file_is_untouched() {
        let path = scratch("set-bad");
        std::fs::write(&path, "{\"minimum_share\": 0.05}\n").unwrap();
        let err = set_value_at(&path, "minimum_share", "2").unwrap_err();
        assert!(err.to_string().contains("minimum_share"), "{err}");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{\"minimum_share\": 0.05}\n"
        );
        // The temp file is not left behind either.
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn a_terminal_that_is_not_there_is_rejected() {
        let path = scratch("set-term");
        std::fs::write(&path, "{}\n").unwrap();
        let err = set_value_at(&path, "terminal", "/nonexistent/term").unwrap_err();
        assert!(err.to_string().contains("not on PATH"), "{err}");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{}\n");
    }

    #[test]
    fn a_typo_elsewhere_does_not_block_an_edit() {
        let path = scratch("set-typo");
        std::fs::write(&path, "{\"gaap\": 1}\n").unwrap();
        set_value_at(&path, "gap", "9").unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["gap"], serde_json::json!(9.0));
        assert_eq!(value["gaap"], serde_json::json!(1));
    }

    #[test]
    fn a_key_that_is_not_a_setting_is_refused() {
        let path = scratch("set-nope");
        std::fs::write(&path, "{}\n").unwrap();
        let err = set_value_at(&path, "bindings", "x").unwrap_err();
        assert!(err.to_string().contains("not a setting"), "{err}");
    }

    #[test]
    fn every_known_key_has_help() {
        for key in KNOWN_KEYS {
            assert_ne!(
                help_text(key),
                "a value this build does not explain",
                "{key} has no help line"
            );
        }
    }

    #[test]
    fn settings_rows_cover_every_key_and_every_command() {
        let config = Config::default();
        let rows = settings_rows(&config);
        let labels: Vec<&str> = rows.iter().map(|(row, _)| row.as_str()).collect();
        for key in KNOWN_KEYS {
            if *key == "bindings" {
                continue; // edited through its commands' rows, not a scalar prompt
            }
            assert!(
                labels.iter().any(|row| row.starts_with(&format!("{key} = "))),
                "no row for {key}: {labels:#?}"
            );
        }
        let commands = pergola_x11::binding_plan_with(&config.bindings);
        assert!(commands.len() > 10);
        for (command, _) in commands {
            assert!(
                labels.iter().any(|row| row.starts_with(&format!("chord: {command} = "))),
                "no chord row for {command}"
            );
        }
        // Labels are unique, or the menu cannot map a pick back to a row.
        let mut seen = std::collections::HashSet::new();
        for label in &labels {
            assert!(seen.insert(*label), "duplicate row {label}");
        }
    }

    #[test]
    fn a_typed_setting_round_trips_through_parse_value() {
        assert_eq!(parse_value("gap", "12").unwrap(), serde_json::json!(12.0));
        assert_eq!(parse_value("overlay", "on").unwrap(), serde_json::json!(true));
        assert_eq!(parse_value("overlay", "off").unwrap(), serde_json::json!(false));
        assert_eq!(parse_value("auto_dock", "new").unwrap(), serde_json::json!("new"));
        assert_eq!(parse_value("auto_dock", "true").unwrap(), serde_json::json!("all"));
        assert_eq!(parse_value("terminal", "foot").unwrap(), serde_json::json!("foot"));
        assert!(parse_value("gap", "wide").is_err());
        assert!(parse_value("overlay", "maybe").is_err());
        assert!(parse_value("auto_dock", "sometimes").is_err());
        assert!(parse_value("terminal", "  ").is_err());
    }

    #[test]
    fn a_syntax_error_is_reported_without_losing_the_defaults() {
        let report = issues_of("{\n  \"gap\": 6,\n  \"overlay\": tru\n}\n", "syntax");
        assert_eq!(report.issues.len(), 1, "{:#?}", report.issues);
        assert!(report.issues[0].message.contains("not valid JSON"));
        assert_eq!(report.config.gap, Config::default().gap);
    }
}

