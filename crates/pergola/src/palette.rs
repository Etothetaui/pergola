//! A human-facing action palette.
//!
//! The daemon takes its orders from a hotkey or the socket, so a person who has
//! neither in mind would need a terminal to tile a window. `pergola palette`
//! puts the action list behind whatever menu program the desktop already ships,
//! and hands the chosen line back to `pergola <action>`.

use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use anyhow::{anyhow, Context, Result};

/// One row of the palette: the words a human reads and the argv they stand for.
#[derive(Debug)]
pub struct Item {
    pub label: &'static str,
    pub argv: &'static [&'static str],
}

const TABLE: &[(&str, &[&str])] = &[
    ("Tile this window left", &["dock", "left"]),
    ("Tile this window right", &["dock", "right"]),
    ("Tile this window above", &["dock", "up"]),
    ("Tile this window below", &["dock", "down"]),
    ("Float this window", &["float"]),
    ("Arrange all windows", &["arrange"]),
    ("Float all windows", &["float-all"]),
    ("Zoom out one level", &["zoom-out"]),
    ("Frame this window", &["frame-window"]),
    ("Show the whole tree", &["everything"]),
    ("Split with a new terminal", &["split-terminal"]),
    ("Toggle the split overlay", &["overlay", "toggle"]),
];

pub fn items() -> Vec<Item> {
    TABLE
        .iter()
        .map(|&(label, argv)| Item { label, argv })
        .collect()
}

pub fn action_for(label: &str) -> Option<&'static [&'static str]> {
    let wanted = label.trim();
    items()
        .into_iter()
        .find(|item| item.label == wanted)
        .map(|item| item.argv)
}

/// Show the palette in a detached `pergola palette-run` process.
///
/// The caller returns immediately: a hotkey is handled inside the daemon's
/// event loop, and the CLI exits as soon as it has printed. The menu itself
/// therefore must live in a process of its own, not a thread of either.
pub fn open() -> Result<()> {
    if find_menu().is_none() {
        return Err(missing_menu());
    }
    let exe = env::current_exe().context("cannot locate the pergola executable")?;
    let mut child = Command::new(exe)
        .arg("palette-run")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| "cannot start the palette process")?;
    // The daemon lives for days: reap the helper so it cannot become a zombie.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Run the menu and hand the chosen action to `pergola <action>`. Blocking:
/// this is what `pergola palette-run` does in its own process.
pub fn run() -> Result<()> {
    let menu = find_menu().ok_or_else(missing_menu)?;
    run_menu(&menu)
}

/// Name the actual problem: a broken `PERGOLA_MENU` is not "no menu program".
fn missing_menu() -> anyhow::Error {
    let custom = env::var("PERGOLA_MENU").unwrap_or_default();
    let custom = custom.trim();
    if custom.is_empty() {
        anyhow!("no menu program found — install rofi (recommended) or set PERGOLA_MENU")
    } else {
        anyhow!("PERGOLA_MENU={custom} is not an executable menu — fix it or unset it")
    }
}

/// How a menu program wants its choices: on stdin, or as list columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Stdin,
    Dmenu,
    Zenity,
    Yad,
}

#[derive(Debug)]
struct Menu {
    path: PathBuf,
    kind: Kind,
}

fn find_menu() -> Option<Menu> {
    let custom = env::var("PERGOLA_MENU").unwrap_or_default();
    let custom = custom.trim();
    if !custom.is_empty() {
        // A typo must fail loudly at `pergola palette`, not silently inside the
        // detached helper where nobody reads stderr.
        let path = resolve(custom)?;
        let kind = kind_of(Path::new(custom));
        return Some(Menu { path, kind });
    }
    ["rofi", "zenity", "yad", "dmenu"]
        .into_iter()
        .find_map(|name| {
            let path = on_path(name)?;
            Some(Menu {
                path,
                kind: kind_of(Path::new(name)),
            })
        })
}

/// A custom `PERGOLA_MENU` speaks the protocol of whichever menu it names.
fn kind_of(command: &Path) -> Kind {
    let name = command.file_name().unwrap_or_default().to_string_lossy();
    if name.contains("zenity") {
        Kind::Zenity
    } else if name.contains("yad") {
        Kind::Yad
    } else if name.contains("dmenu") {
        Kind::Dmenu
    } else {
        Kind::Stdin
    }
}

/// A path as given (absolute or relative) or a name to look up on `PATH`.
fn resolve(command: &str) -> Option<PathBuf> {
    let direct = PathBuf::from(command);
    if is_executable(&direct) {
        return Some(direct);
    }
    on_path(command)
}

fn on_path(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn run_menu(menu: &Menu) -> Result<()> {
    let labels: Vec<&str> = items().iter().map(|item| item.label).collect();
    let output = match menu.kind {
        Kind::Stdin => run_stdin_menu(&menu.path, &["-dmenu", "-i", "-p", "pergola"], &labels)?,
        // dmenu has no `-dmenu`; its only flags are -i and -p.
        Kind::Dmenu => run_stdin_menu(&menu.path, &["-i", "-p", "pergola"], &labels)?,
        Kind::Zenity => run_list_menu(&menu.path, &["--list", "--title=Pergola"], &labels)?,
        Kind::Yad => run_list_menu(&menu.path, &["--list", "--no-headers"], &labels)?,
    };

    let label = chosen(&output, menu.kind);
    if label.is_empty() {
        return Ok(()); // cancelled
    }
    let Some(argv) = action_for(&label) else {
        eprintln!("pergola: palette: no action for {label:?}");
        return Ok(());
    };
    spawn_action(argv)
}

/// rofi and dmenu read the list from stdin and print the chosen line.
fn run_stdin_menu(path: &Path, args: &[&str], labels: &[&str]) -> Result<Output> {
    let mut child = Command::new(path)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| format!("cannot run {}", path.display()))?;
    if let Some(stdin) = child.stdin.as_mut() {
        let mut list = labels.join("\n");
        list.push('\n');
        stdin
            .write_all(list.as_bytes())
            .context("cannot feed the menu")?;
    }
    child.wait_with_output().context("cannot read the menu")
}

fn run_list_menu(path: &Path, args: &[&str], labels: &[&str]) -> Result<Output> {
    Command::new(path)
        .args(args)
        .arg("--column=Action")
        .args(labels)
        .output()
        .with_context(|| format!("cannot run {}", path.display()))
}

fn chosen(output: &Output, kind: Kind) -> String {
    let raw = String::from_utf8_lossy(&output.stdout);
    let line = raw.lines().next().unwrap_or("").trim();
    // yad answers `label|`, echoing the single column it was given.
    match kind {
        Kind::Yad => line.trim_end_matches(|c: char| c == '|' || c.is_whitespace()),
        _ => line,
    }
    .to_string()
}

fn spawn_action(argv: &[&str]) -> Result<()> {
    let exe = env::current_exe().context("cannot locate the pergola executable")?;
    let status = Command::new(exe)
        .args(argv)
        .status()
        .with_context(|| format!("cannot run `pergola {}`", argv.join(" ")))?;
    if !status.success() {
        eprintln!(
            "pergola: palette: `pergola {}` exited with {status}",
            argv.join(" ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn labels_are_unique() {
        let all = items();
        let mut seen = HashSet::new();
        for item in &all {
            assert!(seen.insert(item.label), "duplicate label {}", item.label);
        }
        assert_eq!(all.len(), 12);
    }

    #[test]
    fn every_action_is_a_known_verb() {
        let verbs = [
            "dock",
            "float",
            "arrange",
            "float-all",
            "zoom-out",
            "frame-window",
            "everything",
            "split-terminal",
            "overlay",
        ];
        for item in items() {
            let first = *item.argv.first().expect("empty argv");
            assert!(
                verbs.contains(&first),
                "{}: unknown verb {first}",
                item.label
            );
        }
    }

    #[test]
    fn menu_programs_map_to_their_own_protocol() {
        assert_eq!(kind_of(Path::new("rofi")), Kind::Stdin);
        assert_eq!(kind_of(Path::new("/usr/bin/dmenu")), Kind::Dmenu);
        assert_eq!(kind_of(Path::new("zenity")), Kind::Zenity);
        assert_eq!(kind_of(Path::new("/usr/bin/yad")), Kind::Yad);
    }

    #[test]
    fn a_broken_menu_path_is_not_a_menu() {
        assert_eq!(resolve("pergola-no-such-menu"), None);
        assert_eq!(resolve("/nonexistent/menu"), None);
    }

    #[test]
    fn action_for_round_trips() {
        for item in items() {
            assert_eq!(action_for(item.label), Some(item.argv));
            assert_eq!(action_for(&format!("  {}  ", item.label)), Some(item.argv));
        }
        assert_eq!(action_for("nope"), None);
        assert_eq!(action_for(""), None);
        assert_eq!(action_for("   "), None);
    }
}
