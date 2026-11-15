//! Pergola — a fractal window manager layer for X11 desktops.
//!
//! `pergola daemon` runs the layout engine; every other subcommand is a thin
//! client that forwards to it over a Unix socket.

mod config;
mod daemon;
mod ipc;
mod palette;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use pergola_core::{fractal_tree, LayoutEngine, Rect};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut socket = config::socket_path();
    let mut rest: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--socket" => {
                if let Some(path) = args.get(i + 1) {
                    socket = PathBuf::from(path);
                    i += 2;
                } else {
                    eprintln!("pergola: --socket needs a path");
                    return ExitCode::FAILURE;
                }
            }
            other => {
                rest.push(other.to_string());
                i += 1;
            }
        }
    }

    let Some(command) = rest.first().cloned() else {
        print_help();
        return ExitCode::SUCCESS;
    };

    let result: Result<()> = match command.as_str() {
        "daemon" => run_daemon(&socket),
        "doctor" => doctor(),
        "palette" => palette::open().map(|()| println!("palette opened — pick an action")),
        "palette-run" => palette::run(),
        "sim" => sim(&rest[1..]),
        "keys" => keys(&socket, &rest[1..]),
        "version" | "--version" | "-V" => {
            println!("pergola {}", pergola_core::VERSION);
            Ok(())
        }
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => forward(&socket, &rest),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("pergola: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run_daemon(socket: &PathBuf) -> Result<()> {
    let mut daemon = daemon::Daemon::new()?;
    daemon.run(socket)
}

fn forward(socket: &PathBuf, argv: &[String]) -> Result<()> {
    let response = ipc::call(socket, argv)?;
    if response.ok {
        if !response.message.is_empty() {
            println!("{}", response.message);
        }
        Ok(())
    } else {
        anyhow::bail!("{}", response.message)
    }
}

fn doctor() -> Result<()> {
    for note in pergola_x11::doctor()? {
        println!("{note}");
    }
    println!("pergola {}", pergola_core::VERSION);
    Ok(())
}

fn keys(socket: &PathBuf, args: &[String]) -> Result<()> {
    if !args.iter().any(|a| a == "--check" || a == "--probe") {
        // Ask the running daemon what it actually bound; fall back to the plan.
        if let Ok(response) = ipc::call(socket, &["keys".to_string()]) {
            if response.ok {
                println!("{}", response.message);
                return Ok(());
            }
        }
    }
    if let Some(pos) = args.iter().position(|a| a == "--probe") {
        let x = pergola_x11::X11::connect()?;
        let mut specs: Vec<pergola_x11::KeyBinding> = Vec::new();
        for raw in &args[pos + 1..] {
            let binding = pergola_x11::parse_binding(raw)
                .ok_or_else(|| anyhow::anyhow!("cannot parse binding '{raw}'"))?;
            specs.push(binding);
        }
        let failed = x.grab_keys(&specs)?;
        for binding in &specs {
            let state = if failed.iter().any(|f| f == &binding.label) {
                "TAKEN"
            } else {
                "free "
            };
            println!("{state} {}", binding.label);
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--check") {
        // A running daemon already owns its grabs. Probing from a fresh client
        // would collide with the daemon itself and report every live chord as
        // TAKEN, which is a lie about a working setup. Ask the daemon what it
        // actually bound first, and only probe when nothing is running.
        if let Ok(response) = ipc::call(socket, &["keys".to_string()]) {
            if response.ok {
                // The daemon prints `<label padded to 24> <command>`, and a
                // command may contain a space ("dock left"), so match the tail.
                // Print the label the daemon actually bound, not the plan's
                // primary chord: when the desktop owns the primary, the daemon
                // binds the fallback (`Super+Up`), and echoing the plan would
                // claim a chord the user cannot use.
                let bound_label = |command: &str| -> Option<String> {
                    let needle = format!(" {command}");
                    response.message.lines().find_map(|line| {
                        line.trim_end()
                            .strip_suffix(&needle)
                            .map(|label| label.trim_end().to_string())
                    })
                };
                for (command, specs) in pergola_x11::binding_plan() {
                    match bound_label(command) {
                        Some(label) => println!("bound {:<20} {}", label, command),
                        None => println!("NONE  {:<20} {}", command, specs.join(" / ")),
                    }
                }
                println!("\n(the daemon is running; these are its live grabs)");
                return Ok(());
            }
        }
        let x = pergola_x11::X11::connect()?;
        let (bound, failures) = x.install_bindings(&pergola_x11::binding_plan())?;
        for (binding, command) in &bound {
            println!("bound {:<20} {}", binding.label, command);
        }
        for failure in &failures {
            println!("NONE  {:<20} {}", failure.command, failure.tried.join(" / "));
        }
        return Ok(());
    }
    for (command, specs) in pergola_x11::binding_plan() {
        println!("{:<20} {}", specs[0], command);
    }
    Ok(())
}

/// Headless check: lay the fixture tree out in a 1600x900 frame and print it.
fn sim(args: &[String]) -> Result<()> {
    let engine = LayoutEngine::default();
    let tree = fractal_tree();
    tree.validate()?;
    let frame = Rect::new(0.0, 0.0, 1600.0, 900.0);
    let rects = engine.rects(&tree, frame);
    if args.iter().any(|a| a == "--json") {
        let out: Vec<serde_json::Value> = rects
            .iter()
            .map(|(id, r)| {
                serde_json::json!({"id": id, "x": r.x, "y": r.y, "w": r.w, "h": r.h})
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }
    println!(
        "pergola {} · fixture depth {} · {} nodes",
        pergola_core::VERSION,
        tree.nesting_depth(),
        rects.len()
    );
    for (id, r) in rects {
        println!(
            "{:<16} {:>6.1} {:>6.1} {:>7.1} {:>7.1}",
            id, r.x, r.y, r.w, r.h
        );
    }
    Ok(())
}

fn print_help() {
    println!(
        r#"pergola {version} — a fractal window manager layer for X11 desktops

USAGE
  pergola daemon                 run the layout daemon (one per display)
  pergola <command> [args]       talk to the running daemon

CAMERA
  everything                     show the whole tree (zoom all the way out)
  zoom-in | zoom-out             step one level in / out
  back | forward                 browser-style history
  frame-window                   frame the focused window (again to go back)
  frame-desktop                  frame the desktop layer
  toggle-desktop                 hide or show floating windows
  neighbour <left|right|up|down> move to the neighbouring space
  view <1-9> | save-view <1-9>   saved views

LAYOUT
  dock <left|right|up|down> [--window ID] [--target ID]
  float [--window ID]
  arrange                        tile every window into a balanced fractal tree
  float-all                      return every window to the desktop layer
  split-terminal                 split the focused window with a new terminal
  gap <pixels>                   change the gap between tiles

INSPECTION
  status | tree | list | keys | doctor | sim [--json]
  palette                        human menu of tile actions (rofi/zenity/dmenu)
  overlay [on|off|toggle]
  reload | save | stop

FILES
  config   {config}
  layout   {layout}
  socket   {socket}

The daemon never takes SubstructureRedirect, so it runs inside xfwm4, mutter,
kwin_x11, i3, openbox and any other EWMH-compliant X11 desktop."#,
        version = pergola_core::VERSION,
        config = config::config_path().display(),
        layout = config::layout_path().display(),
        socket = config::socket_path().display(),
    );
}
