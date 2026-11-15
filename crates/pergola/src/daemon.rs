//! The daemon: owns the tree, reconciles it with the live X11 client list, and
//! applies geometry through the running desktop's window manager.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use pergola_core::{
    Axis, Camera, CameraAction, Command, Direction, FloatingWindow, GapHandle, IRect, LayoutEngine,
    LayoutFile, LayoutNode, Rect, SeatRecord,
};
use pergola_x11::{binding_plan, command_for_key, describe, KeyBinding, WindowInfo, X11};
use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::ConfigureNotifyEvent;
use x11rb::protocol::Event;

use crate::config::{self, Config};
use crate::ipc::{Request, Response};

/// A pointer drag in progress on a split boundary.
struct GapDrag {
    first: String,
    second: String,
    axis: Axis,
    /// Split length along `axis` in screen pixels: divide the pointer delta by
    /// it to get a weight delta.
    extent: f64,
    /// Pointer position at ButtonPress, in root coordinates.
    start: (i32, i32),
    /// Tree when the drag began. Every motion re-derives from this snapshot, so
    /// a drag can never accumulate rounding drift.
    snapshot: LayoutNode,
}

/// A window that may have been dragged by the user, watched for the moment it
/// stops moving (edge docking: drop a window on a screen edge to tile it).
struct MoveWatch {
    /// Consecutive configure events that actually changed the geometry.
    moves: u32,
    at: Instant,
    rect: Rect,
}

pub struct Daemon {
    x: X11,
    engine: LayoutEngine,
    file: LayoutFile,
    camera: pergola_core::CameraState,
    config: Config,
    overlay_on: bool,
    bindings: Vec<(KeyBinding, &'static str)>,
    keymap: Vec<(u8, u32)>,
    seats: Vec<SeatRecord>,
    /// Panel that had focus before the current command, used as the default
    /// docking target when the focused window is the one being docked.
    last_focus: Option<String>,
    /// Desktop-space rectangle from the previous apply, used to notice that the
    /// desktop leaf moved (and floating windows must be remapped).
    last_desk: Option<Rect>,
    /// Force floating windows to their stored rect on the next apply.
    place_floating: bool,
    /// Whether the previous apply ran under a camera transform. Live rects seen
    /// while zoomed are screen-space, not world-space, so they must not be
    /// adopted back as the window's stored position.
    last_zoomed: bool,
    running: bool,
    layout_path: PathBuf,
    /// Split boundaries exposed to the pointer, as (grab window, handle).
    gap_bands: Vec<(u32, GapHandle)>,
    /// The gap drag in progress, if any.
    drag: Option<GapDrag>,
    /// The frame rectangle we last asked the WM to produce, per window. A
    /// configure that differs from this is the user moving the window, not us.
    applied: HashMap<u32, IRect>,
    /// Windows whose configure stream is being watched for a drop on an edge.
    moves: HashMap<u32, MoveWatch>,
    /// When our own apply last ran; edge detection ignores its event wake.
    last_apply: Instant,
}

impl Daemon {
    pub fn new() -> Result<Self> {
        let x = X11::connect()?;
        let config = Config::load();
        let layout_path = config::layout_path();
        let file = match std::fs::read_to_string(&layout_path) {
            Ok(raw) => LayoutFile::from_json(&raw).unwrap_or_else(|e| {
                eprintln!("pergola: ignoring malformed layout: {e}");
                LayoutFile::default()
            }),
            Err(_) => LayoutFile::default(),
        };
        let engine = LayoutEngine::new(config.gap, config.minimum_share)
            .with_desktop_share(config.desktop_share);
        let camera = file.camera();
        let seats = file
            .pergola
            .as_ref()
            .map(|p| p.seats.clone())
            .unwrap_or_default();
        // The config is the default; a toggle only lasts for the session, so a
        // persisted `overlay` value must not override a config edit.
        let overlay_on = config.overlay;
        let bindings: Vec<(KeyBinding, &'static str)> = Vec::new();
        let keymap = build_keymap(&x)?;
        let mut daemon = Self {
            x,
            engine,
            file,
            camera,
            config,
            overlay_on,
            bindings,
            keymap,
            seats,
            last_focus: None,
            last_desk: None,
            place_floating: true,
            last_zoomed: false,
            running: true,
            layout_path,
            gap_bands: Vec::new(),
            drag: None,
            applied: HashMap::new(),
            moves: HashMap::new(),
            last_apply: Instant::now(),
        };
        daemon.reconcile()?;
        daemon.normalize_camera();
        daemon.apply()?;
        Ok(daemon)
    }

    // MARK: reconciliation

    /// Match the tree against the live client list: drop panels whose window is
    /// gone, adopt new windows, refresh titles.
    fn reconcile(&mut self) -> Result<bool> {
        let windows = self.x.windows()?;
        let live: HashSet<u32> = windows.iter().map(|w| w.id).collect();

        let gone: Vec<SeatRecord> = self
            .seats
            .iter()
            .filter(|s| !live.contains(&s.window))
            .cloned()
            .collect();
        let mut changed = !gone.is_empty();
        for seat in gone {
            self.applied.remove(&seat.window);
            self.moves.remove(&seat.window);
            self.seats.retain(|s| s.seat != seat.seat);
            if self.file.doc.root.node(&seat.panel).is_some() {
                if let Ok(tree) = self.engine.removing(&seat.panel, &self.file.doc.root) {
                    self.replace_root(tree);
                }
            }
            self.file.doc.floating.retain(|f| f.panel.id != seat.panel);
        }

        for w in &windows {
            if let Some(record) = self.seats.iter_mut().find(|s| s.window == w.id) {
                record.title = w.title.clone();
                record.class = w.class.clone();
                continue;
            }
            let seat = format!("s-{:x}", w.id);
            let panel = format!("p-{:x}", w.id);
            self.seats.push(SeatRecord {
                seat: seat.clone(),
                panel: panel.clone(),
                window: w.id,
                class: w.class.clone(),
                title: w.title.clone(),
            });
            changed = true;
            if self.config.auto_dock {
                let target = self
                    .focused_panel()
                    .unwrap_or_else(|| self.file.doc.root.id().to_string());
                let node = LayoutNode::panel(panel.clone(), vec![seat.clone()]);
                match self
                    .engine
                    .inserting(node, &target, Direction::Right, &self.file.doc.root)
                {
                    Ok(tree) => self.replace_root(tree),
                    Err(_) => self.push_floating(w, panel, seat),
                }
            } else {
                self.push_floating(w, panel, seat);
            }
        }
        // Watch every managed window for the WM's synthetic ConfigureNotify:
        // that is the only way to see a drag the daemon did not initiate.
        for seat in &self.seats {
            let _ = self.x.watch_window(seat.window);
        }
        Ok(changed)
    }

    fn push_floating(&mut self, w: &WindowInfo, panel: String, seat: String) {
        let rect = w.frame_rect().to_rect();
        self.file.doc.floating.push(FloatingWindow {
            panel: pergola_core::Panel::new(panel, vec![seat]),
            rect,
            size: pergola_core::Size::new(w.rect.w, w.rect.h),
            z: None,
        });
    }

    /// Keep the camera pointing at a node that exists. An empty or stale
    /// `framed` means "the whole tree".
    fn normalize_camera(&mut self) {
        if self.camera.framed.is_empty()
            || self.file.doc.root.node(&self.camera.framed).is_none()
        {
            self.camera.framed = self.file.doc.root.id().to_string();
        }
    }

    /// Replace the tree, keeping "everything" framed on the new root.
    fn replace_root(&mut self, new_root: LayoutNode) {
        let old = self.file.doc.root.id().to_string();
        if self.camera.framed == old {
            self.camera.framed = new_root.id().to_string();
        }
        self.file.doc.root = new_root;
    }

    fn focused_panel(&self) -> Option<String> {
        let active = self.x.active_window()?;
        let seat = self.seats.iter().find(|s| s.window == active)?;
        Some(seat.panel.clone())
    }

    fn seat_for_window(&self, win: u32) -> Option<&SeatRecord> {
        self.seats.iter().find(|s| s.window == win)
    }

    // MARK: applying geometry

    fn apply(&mut self) -> Result<()> {
        let screen = self.x.workarea_rect()?;
        let world = screen.to_rect();
        let rects = self.engine.rects(&self.file.doc.root, world);
        let framed = if self.camera.framed != self.file.doc.root.id() {
            rects
                .iter()
                .find(|(id, _)| id == &self.camera.framed)
                .map(|(_, r)| *r)
        } else {
            None
        };
        let tf = |r: Rect| transform(r, framed, world);

        for (id, rect) in &rects {
            let Some(LayoutNode::Panel(panel)) = self.file.doc.root.node(id) else {
                continue;
            };
            let target = park_if_outside(tf(*rect).to_irect(), screen, framed.is_some());
            for view in &panel.views {
                if let Some(seat) = self.seats.iter().find(|s| &s.seat == view) {
                    let _ = self.x.move_resize(seat.window, target);
                    self.applied.insert(seat.window, target);
                }
            }
            if let Some(visible) = panel.visible() {
                if let Some(seat) = self.seats.iter().find(|s| s.seat == visible) {
                    let _ = self.x.raise(seat.window);
                }
            }
        }

        // The desktop leaf is a space, not the whole screen: floating windows
        // are mapped into whatever rectangle it currently occupies. Their stored
        // rects stay in full-screen ("world") coordinates, so a drag is undone
        // into world space and the layout never fights the user.
        let desk = rects
            .iter()
            .find(|(id, _)| id == "desktop")
            .map(|(_, r)| *r);
        let desk_changed = self
            .last_desk
            .map_or(true, |d| !same_rect(d, desk.unwrap_or(world)));
        let place = self.place_floating || desk_changed || self.last_zoomed;
        let mut floating = std::mem::take(&mut self.file.doc.floating);
        for f in floating.iter_mut() {
            let window = f
                .panel
                .views
                .first()
                .and_then(|view| self.seats.iter().find(|s| &s.seat == view))
                .map(|s| s.window);
            let Some(window) = window else { continue };
            if self.camera.desktop_hidden {
                let target = IRect::new(
                    screen.x,
                    screen.y + screen.h,
                    f.rect.w as i32,
                    f.rect.h as i32,
                );
                let _ = self.x.move_resize(window, target);
                self.applied.insert(window, target);
            } else if framed.is_some() {
                let target = tf(into_screen(f.rect, desk, world)).to_irect();
                let _ = self.x.move_resize(window, target);
                self.applied.insert(window, target);
            } else if place {
                let target = into_screen(f.rect, desk, world).to_irect();
                let _ = self.x.move_resize(window, target);
                self.applied.insert(window, target);
            } else if let Ok(Some(info)) = self.x.info(window) {
                // The desktop space did not move, so the window is where the
                // user put it: learn the new position instead of snapping back.
                f.rect = into_world(info.frame_rect().to_rect(), desk, world);
            }
        }
        self.file.doc.floating = floating;
        self.last_desk = Some(desk.unwrap_or(world));
        self.last_zoomed = framed.is_some();
        self.place_floating = false;

        if self.overlay_on {
            let framed_id = if self.camera.framed == self.file.doc.root.id() {
                None
            } else {
                Some(self.camera.framed.as_str())
            };
            let bars: Vec<(IRect, u32)> = overlay_bars(
                &self.file.doc.root,
                &rects,
                framed_id,
                self.engine.gap,
                world,
            )
            .into_iter()
            .map(|(bar, color)| {
                let r = tf(bar.to_rect());
                (
                    IRect::new(
                        r.x.round() as i32,
                        r.y.round() as i32,
                        r.w.round().max(1.0) as i32,
                        r.h.round().max(1.0) as i32,
                    ),
                    color,
                )
            })
            .collect();
            self.x.overlay_show(&bars)?;
        } else {
            self.x.overlay_hide()?;
        }
        // The pointer-facing boundaries follow every layout change — except
        // mid-drag, where replacing the windows under the pointer would abort
        // the drag.
        if self.drag.is_none() {
            self.refresh_gap_bands()?;
        }
        self.last_apply = Instant::now();
        self.x.conn.flush()?;
        Ok(())
    }

    /// Rebuild the invisible grab areas for every draggable split boundary, in
    /// screen coordinates (so they follow the camera).
    fn refresh_gap_bands(&mut self) -> Result<()> {
        let screen = self.x.workarea_rect()?;
        let world = screen.to_rect();
        let rects = self.engine.rects(&self.file.doc.root, world);
        let framed = if self.camera.framed != self.file.doc.root.id() {
            rects
                .iter()
                .find(|(id, _)| id == &self.camera.framed)
                .map(|(_, r)| *r)
        } else {
            None
        };
        let tf = |r: Rect| transform(r, framed, world);
        let mut handles = self.engine.gap_handles(&self.file.doc.root, world);
        let mut bands = Vec::new();
        for h in &mut handles {
            let shown = tf(h.band);
            // The camera transform is non-uniform, so the pixel→weight
            // denominator must use the axis the boundary moves on.
            let scale = axis_scale(h.axis, h.band, shown);
            h.band = shown;
            h.extent *= scale;
            bands.push((
                IRect::new(
                    shown.x.round() as i32,
                    shown.y.round() as i32,
                    shown.w.round().max(1.0) as i32,
                    shown.h.round().max(1.0) as i32,
                ),
                h.axis,
            ));
        }
        let ids = self.x.grab_bands_show(&bands)?;
        self.gap_bands = ids.into_iter().zip(handles).collect();
        Ok(())
    }

    fn save(&mut self) -> Result<()> {
        self.file.set_camera(&self.camera);
        let meta = pergola_core::PergolaMeta {
            seats: self.seats.clone(),
            gap: Some(self.engine.gap),
            auto_dock: Some(self.config.auto_dock),
            overlay: Some(self.overlay_on),
        };
        self.file.pergola = Some(meta);
        if let Some(parent) = self.layout_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.layout_path, self.file.to_json())
            .with_context(|| format!("writing {}", self.layout_path.display()))?;
        Ok(())
    }

    // MARK: commands

    pub fn handle(&mut self, argv: &[String]) -> Result<String> {
        // A command that arrives mid-drag (a hotkey, an IPC client) ends the
        // pointer gesture: its tree snapshot would otherwise be reinstated by
        // the next motion event.
        if self.drag.is_some() {
            let _ = self.end_gap_drag();
        }
        let result = self.dispatch(argv);
        if let Some(panel) = self.focused_panel() {
            self.last_focus = Some(panel);
        }
        result
    }

    fn dispatch(&mut self, argv: &[String]) -> Result<String> {
        let Some(name) = argv.first().map(String::as_str) else {
            return Err(anyhow!("empty command"));
        };
        let rest = &argv[1..];
        match name {
            "status" => self.status(),
            "tree" => Ok(self.file.to_json()),
            "list" => self.list(),
            "rects" => self.rects(rest),
            "dock" => self.dock(rest),
            "float" => self.float(rest),
            "arrange" => self.arrange(),
            "float-all" => self.float_all(),
            "gap" => self.set_gap(rest),
            "overlay" => self.overlay(rest),
            "keys" => self.keys(),
            "reload" => self.reload(),
            "save" => {
                self.save()?;
                Ok(format!("saved {}", self.layout_path.display()))
            }
            "stop" => {
                self.running = false;
                Ok("stopping".to_string())
            }
            "split-terminal" => self.split_terminal(),
            "palette" => {
                crate::palette::open()?;
                Ok("palette opened".to_string())
            }
            "settings" => Ok("settings are in ~/.config/pergola/config.json".to_string()),
            "url" => {
                let raw = rest.join(" ");
                let cmd = Command::parse(&raw).map_err(|e| anyhow!(e))?;
                self.camera_command(cmd)
            }
            other => {
                let cmd = parse_command(other, rest)?;
                self.camera_command(cmd)
            }
        }
    }

    fn camera_command(&mut self, cmd: Command) -> Result<String> {
        let screen = self.x.workarea_rect()?.to_rect();
        let focused = self.focused_panel();
        let action = cmd
            .camera_action(focused.as_deref(), screen)
            .ok_or_else(|| anyhow!("'{}' is not a camera command", cmd.name()))?;
        let camera = Camera::new(self.engine);
        self.camera = camera.apply(&action, &self.camera, &self.file.doc.root, screen);
        match action {
            CameraAction::Everything => self.overlay_on = true,
            CameraAction::ZoomIn { .. } | CameraAction::Frame { .. } => self.overlay_on = false,
            _ => {}
        }
        self.apply()?;
        self.save()?;
        Ok(format!(
            "{} → framed {} (depth {})",
            cmd.name(),
            self.camera.framed,
            self.file.doc.root.nesting_depth()
        ))
    }

    fn status(&self) -> Result<String> {
        let screen = self.x.workarea_rect()?;
        let panels = self
            .file
            .doc
            .root
            .flatten()
            .iter()
            .filter(|n| matches!(n, LayoutNode::Panel(_)))
            .count();
        Ok(format!(
            "pergola {} · {}x{} · {} panels, {} floating · depth {} · framed {} · gap {} · overlay {}",
            pergola_core::VERSION,
            screen.w,
            screen.h,
            panels,
            self.file.doc.floating.len(),
            self.file.doc.root.nesting_depth(),
            self.camera.framed,
            self.engine.gap,
            if self.overlay_on { "on" } else { "off" }
        ) + &match self.x.active_window() {
            Some(w) => format!(
                " · active 0x{w:x} ({})",
                self.seat_for_window(w).map(|s| s.panel.as_str()).unwrap_or("unmanaged")
            ),
            None => " · active none".to_string(),
        })
    }

    /// Current geometry: every panel's *frame* rectangle after the camera
    /// transform. Handy for scripts and for checking the daemon against itself.
    fn rects(&self, args: &[String]) -> Result<String> {
        let screen = self.x.workarea_rect()?.to_rect();
        let rects = self.engine.rects(&self.file.doc.root, screen);
        let framed = if self.camera.framed != self.file.doc.root.id() {
            rects
                .iter()
                .find(|(id, _)| id == &self.camera.framed)
                .map(|(_, r)| *r)
        } else {
            None
        };
        let rows: Vec<serde_json::Value> = rects
            .iter()
            .filter(|(id, _)| {
                self.file
                    .doc
                    .root
                    .node(id)
                    .map(|n| matches!(n, LayoutNode::Panel(_)))
                    .unwrap_or(false)
            })
            .map(|(id, r)| {
                let f = transform(*r, framed, screen).to_irect();
                serde_json::json!({"id": id, "x": f.x, "y": f.y, "w": f.w, "h": f.h})
            })
            .collect();
        if args.iter().any(|a| a == "--json") {
            return Ok(serde_json::to_string_pretty(&rows)?);
        }
        let mut out = String::new();
        for row in rows {
            out.push_str(&format!(
                "{} {} {} {} {}\n",
                row["id"].as_str().unwrap_or(""),
                row["x"],
                row["y"],
                row["w"],
                row["h"]
            ));
        }
        Ok(out.trim_end().to_string())
    }

    fn list(&self) -> Result<String> {
        let windows = self.x.windows()?;
        let mut out = String::new();
        for w in &windows {
            let seat = self.seat_for_window(w.id);
            let where_ = match seat {
                Some(s) if self.file.doc.root.node(&s.panel).is_some() => "docked",
                Some(_) => "floating",
                None => "unmanaged",
            };
            out.push_str(&format!("{:<9} {}\n", where_, describe(w)));
        }
        Ok(out.trim_end().to_string())
    }

    fn dock(&mut self, args: &[String]) -> Result<String> {
        let edge = args
            .first()
            .and_then(|s| Direction::parse(s))
            .ok_or_else(|| anyhow!("usage: pergola dock <left|right|up|down> [--window ID] [--target ID]"))?;
        let win = flag(args, "--window")
            .and_then(|v| parse_window(&v))
            .or_else(|| self.x.active_window())
            .ok_or_else(|| anyhow!("no active window to dock"))?;
        let target = flag(args, "--target");
        self.dock_window(win, edge, target)
    }

    /// Dock one window against `target` (or the last focused space). Split out
    /// from [`Self::dock`] so the pointer path — dropping a window on a screen
    /// edge — docks exactly the same way the CLI does.
    fn dock_window(
        &mut self,
        win: u32,
        edge: Direction,
        target: Option<String>,
    ) -> Result<String> {
        let seat = self
            .seat_for_window(win)
            .ok_or_else(|| anyhow!("window 0x{win:x} is not managed by pergola"))?
            .clone();
        let mut target = target
            .or_else(|| self.focused_panel())
            .unwrap_or_else(|| self.file.doc.root.id().to_string());
        if target == seat.panel {
            // The focused window is the one being docked: dock beside whatever
            // had focus before it, falling back to the whole tree.
            target = self
                .last_focus
                .clone()
                .filter(|p| p != &seat.panel && self.file.doc.root.node(p).is_some())
                .unwrap_or_else(|| self.file.doc.root.id().to_string());
        }
        if self.file.doc.root.node(&seat.panel).is_some() {
            let tree = self.engine.removing(&seat.panel, &self.file.doc.root)?;
            self.replace_root(tree);
        }
        self.file.doc.floating.retain(|f| f.panel.id != seat.panel);
        if self.file.doc.root.node(&target).is_none() {
            target = self.file.doc.root.id().to_string();
        }
        let node = LayoutNode::panel(seat.panel.clone(), vec![seat.seat.clone()]);
        let tree = self
            .engine
            .inserting(node, &target, edge, &self.file.doc.root)
            .map_err(|e| anyhow!("{e}"))?;
        self.replace_root(tree);
        self.normalize_camera();
        self.apply()?;
        self.save()?;
        Ok(format!(
            "docked {} {} of {} (depth {})",
            seat.panel,
            match edge {
                Direction::Left => "left",
                Direction::Right => "right",
                Direction::Up => "above",
                Direction::Down => "below",
            },
            target,
            self.file.doc.root.nesting_depth()
        ))
    }

    fn float(&mut self, args: &[String]) -> Result<String> {
        let win = flag(args, "--window")
            .and_then(|v| parse_window(&v))
            .or_else(|| self.x.active_window())
            .ok_or_else(|| anyhow!("no active window to float"))?;
        self.float_window(win, None)
    }

    /// Return one window to the desktop layer. `dropped` is the world-space
    /// rectangle it was let go at (mouse path); without it the window keeps the
    /// rectangle its tile had, so `pergola float` does not teleport it.
    fn float_window(&mut self, win: u32, dropped: Option<Rect>) -> Result<String> {
        let seat = self
            .seat_for_window(win)
            .ok_or_else(|| anyhow!("window 0x{win:x} is not managed by pergola"))?
            .clone();
        let rect = dropped
            .or_else(|| {
                self.engine
                    .rect_of(
                        &self.file.doc.root,
                        self.x.workarea_rect().ok()?.to_rect(),
                        &seat.panel,
                    )
            })
            .unwrap_or_else(|| Rect::new(80.0, 80.0, 800.0, 500.0));
        if self.file.doc.root.node(&seat.panel).is_some() {
            let tree = self.engine.removing(&seat.panel, &self.file.doc.root)?;
            self.replace_root(tree);
        }
        self.file.doc.floating.retain(|f| f.panel.id != seat.panel);
        self.file.doc.floating.push(FloatingWindow {
            panel: pergola_core::Panel::new(seat.panel.clone(), vec![seat.seat.clone()]),
            rect,
            size: pergola_core::Size::new(rect.w as i32, rect.h as i32),
            z: None,
        });
        self.place_floating = true;
        self.normalize_camera();
        self.apply()?;
        self.save()?;
        Ok(format!("floated {}", seat.panel))
    }

    fn arrange(&mut self) -> Result<String> {
        let mut panels: Vec<LayoutNode> = Vec::new();
        for node in self.file.doc.root.flatten() {
            if let LayoutNode::Panel(p) = node {
                panels.push(LayoutNode::Panel(p.clone()));
            }
        }
        for floating in &self.file.doc.floating {
            panels.push(LayoutNode::Panel(floating.panel.clone()));
        }
        if panels.is_empty() {
            return Ok("nothing to arrange".to_string());
        }
        let count = panels.len();
        let tree = self.engine.arrange(panels, self.config.desktop_share);
        self.replace_root(tree);
        self.file.doc.floating.clear();
        self.normalize_camera();
        self.apply()?;
        self.save()?;
        Ok(format!(
            "arranged {count} panels (depth {})",
            self.file.doc.root.nesting_depth()
        ))
    }

    fn float_all(&mut self) -> Result<String> {
        let screen = self.x.workarea_rect()?.to_rect();
        let rects = self.engine.rects(&self.file.doc.root, screen);
        let mut moved = 0;
        for node in self.file.doc.root.flatten() {
            if let LayoutNode::Panel(p) = node {
                let rect = rects
                    .iter()
                    .find(|(id, _)| id == &p.id)
                    .map(|(_, r)| *r)
                    .unwrap_or_else(|| Rect::new(80.0, 80.0, 800.0, 500.0));
                self.file.doc.floating.push(FloatingWindow {
                    panel: p.clone(),
                    rect,
                    size: pergola_core::Size::new(rect.w as i32, rect.h as i32),
                    z: None,
                });
                moved += 1;
            }
        }
        self.replace_root(LayoutNode::desktop());
        self.camera.framed = "desktop".to_string();
        self.place_floating = true;
        self.normalize_camera();
        self.apply()?;
        self.save()?;
        Ok(format!("floated {moved} panels"))
    }

    fn set_gap(&mut self, args: &[String]) -> Result<String> {
        let gap: f64 = args
            .first()
            .ok_or_else(|| anyhow!("usage: pergola gap <pixels>"))?
            .parse()
            .map_err(|_| anyhow!("gap must be a number"))?;
        self.engine = LayoutEngine::new(gap, self.engine.minimum_share)
            .with_desktop_share(self.engine.desktop_share);
        self.config.gap = gap;
        self.apply()?;
        self.save()?;
        Ok(format!("gap {gap}"))
    }

    fn overlay(&mut self, args: &[String]) -> Result<String> {
        match args.first().map(String::as_str) {
            Some("on") => self.overlay_on = true,
            Some("off") => self.overlay_on = false,
            _ => self.overlay_on = !self.overlay_on,
        }
        self.apply()?;
        self.save()?;
        Ok(format!(
            "overlay {}",
            if self.overlay_on { "on" } else { "off" }
        ) + &match self.x.active_window() {
            Some(w) => format!(
                " · active 0x{w:x} ({})",
                self.seat_for_window(w).map(|s| s.panel.as_str()).unwrap_or("unmanaged")
            ),
            None => " · active none".to_string(),
        })
    }

    fn keys(&self) -> Result<String> {
        let mut out = String::new();
        for (binding, command) in &self.bindings {
            out.push_str(&format!("{:<24} {}\n", binding.label, command));
        }
        Ok(out.trim_end().to_string())
    }

    fn reload(&mut self) -> Result<String> {
        self.config = Config::load();
        self.engine = LayoutEngine::new(self.config.gap, self.config.minimum_share)
            .with_desktop_share(self.config.desktop_share);
        self.apply()?;
        Ok("config reloaded".to_string())
    }

    fn split_terminal(&mut self) -> Result<String> {
        let before: HashSet<u32> = self.x.client_list()?.into_iter().collect();
        let mut command = std::process::Command::new(&self.config.terminal);
        if self.config.terminal.ends_with("xfce4-terminal") {
            command.arg("--disable-server");
        }
        command
            .spawn()
            .with_context(|| format!("cannot launch {}", self.config.terminal))?;
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut new_window = None;
        while Instant::now() < deadline {
            thread::sleep(Duration::from_millis(120));
            if let Ok(list) = self.x.client_list() {
                if let Some(id) = list.into_iter().find(|id| !before.contains(id)) {
                    new_window = Some(id);
                    break;
                }
            }
        }
        let win = new_window.ok_or_else(|| anyhow!("the terminal did not open a window"))?;
        self.reconcile()?;
        self.normalize_camera();
        let seat = self
            .seat_for_window(win)
            .ok_or_else(|| anyhow!("new terminal window was not adopted"))?
            .clone();
        let target = self
            .focused_panel()
            .unwrap_or_else(|| self.file.doc.root.id().to_string());
        if self.file.doc.root.node(&seat.panel).is_some() {
            let tree = self.engine.removing(&seat.panel, &self.file.doc.root)?;
            self.replace_root(tree);
        }
        self.file.doc.floating.retain(|f| f.panel.id != seat.panel);
        let node = LayoutNode::panel(seat.panel.clone(), vec![seat.seat.clone()]);
        let tree = self
            .engine
            .inserting(node, &target, Direction::Right, &self.file.doc.root)
            .map_err(|e| anyhow!("{e}"))?;
        self.replace_root(tree);
        self.apply()?;
        self.save()?;
        Ok(format!("split with {} (0x{win:x})", self.config.terminal))
    }

    // MARK: event loop

    pub fn run(&mut self, socket: &std::path::Path) -> Result<()> {
        let socket = socket.to_path_buf();
        if socket.exists() {
            // One daemon per display. A socket that still accepts a connection
            // belongs to a live daemon: refuse instead of unlinking it, which
            // would leave the first daemon running but unreachable.
            if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
                anyhow::bail!(
                    "a pergola daemon is already running on {} (one per display)",
                    socket.display()
                );
            }
            // No listener: a killed daemon left the file behind. Reclaim it.
            let _ = std::fs::remove_file(&socket);
        }
        if let Some(parent) = socket.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let listener = UnixListener::bind(&socket)
            .with_context(|| format!("binding {}", socket.display()))?;
        let (tx, rx) = mpsc::channel::<(Request, UnixStream)>();
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let Ok(clone) = stream.try_clone() else { continue };
                let mut reader = BufReader::new(clone);
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    if let Ok(request) = serde_json::from_str::<Request>(line.trim()) {
                        let _ = tx.send((request, stream));
                    }
                }
            }
        });

        if self.config.hotkeys {
            match self.x.install_bindings(&binding_plan()) {
                Ok((bound, failures)) => {
                    self.bindings = bound;
                    for failure in &failures {
                        eprintln!(
                            "pergola: no free key for '{}' (tried {})",
                            failure.command,
                            failure.tried.join(", ")
                        );
                    }
                }
                Err(e) => eprintln!("pergola: cannot grab hotkeys: {e}"),
            }
        }

        eprintln!(
            "pergola {} listening on {} ({} windows)",
            pergola_core::VERSION,
            socket.display(),
            self.seats.len()
        );

        let mut last_reconcile = Instant::now();
        while self.running {
            if last_reconcile.elapsed() >= Duration::from_secs(2) {
                last_reconcile = Instant::now();
                match self.reconcile() {
                    Ok(true) => {
                        if self.drag.is_some() {
                            let _ = self.end_gap_drag();
                        }
                        self.normalize_camera();
                        let _ = self.apply();
                        let _ = self.save();
                    }
                    Ok(false) => {}
                    Err(e) => eprintln!("pergola: reconcile: {e:#}"),
                }
            }
            while let Ok((request, mut stream)) = rx.try_recv() {
                let response = match self.handle(&request.argv) {
                    Ok(message) => Response::ok(message),
                    Err(e) => Response::err(format!("{e:#}")),
                };
                let _ = writeln!(stream, "{}", serde_json::to_string(&response)?);
            }
            let _ = self.settle_moves();
            // One blocking poll for the idle path, then drain whatever else the
            // server has queued (a window drag produces a burst of configures,
            // and the pointer drag stream must not fall behind).
            match self.x.next_event(50) {
                Ok(Some(event)) => {
                    if let Err(e) = self.on_x_event(event) {
                        eprintln!("pergola: event: {e:#}");
                    }
                    let mut drained = 0;
                    while drained < 256 {
                        match self.x.poll_event() {
                            Ok(Some(event)) => {
                                if let Err(e) = self.on_x_event(event) {
                                    eprintln!("pergola: event: {e:#}");
                                }
                                drained += 1;
                            }
                            _ => break,
                        }
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("pergola: X connection lost: {e}");
                    break;
                }
            }
        }

        let _ = self.x.overlay_hide();
        let _ = self.save();
        let _ = std::fs::remove_file(&socket);
        Ok(())
    }
}

// MARK: pointer and window-move events

impl Daemon {
    fn on_x_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::KeyPress(key) => {
                let state: u16 = key.state.into();
                if std::env::var_os("PERGOLA_DEBUG").is_some() {
                    eprintln!("pergola: KeyPress detail={} state=0x{state:x}", key.detail);
                }
                let mods = state & (4 | 8 | 1); // Control | Mod1 | Shift
                let keysym = self
                    .keymap
                    .iter()
                    .find(|(code, _)| *code == key.detail)
                    .map(|(_, sym)| *sym);
                if let Some(keysym) = keysym {
                    if let Some(command) = command_for_key(&self.bindings, mods, keysym) {
                        let argv: Vec<String> =
                            command.split_whitespace().map(str::to_string).collect();
                        if let Err(e) = self.handle(&argv) {
                            eprintln!("pergola: {command}: {e:#}");
                        }
                    }
                }
            }
            Event::ButtonPress(press) => {
                // Only the primary button drags a boundary, and a press
                // anywhere else means we never saw the previous release.
                if press.detail != 1 || self.band_for_window(press.event).is_none() {
                    self.end_gap_drag()?;
                    return Ok(());
                }
                // The implicit pointer grab keeps the motion stream on this
                // same window until the button is released.
                self.begin_gap_drag(press.event, press.root_x as i32, press.root_y as i32)?;
            }
            Event::MotionNotify(motion) => {
                self.update_gap_drag(motion.root_x as i32, motion.root_y as i32)?;
            }
            Event::ButtonRelease(_) => self.end_gap_drag()?,
            Event::ConfigureNotify(configure) => self.note_configure(&configure)?,
            _ => {}
        }
        Ok(())
    }

    fn band_for_window(&self, window: u32) -> Option<&GapHandle> {
        self.gap_bands
            .iter()
            .find(|(id, _)| *id == window)
            .map(|(_, handle)| handle)
    }

    /// Start dragging a split boundary. The tree is snapshotted so that every
    /// motion is absolute, not incremental.
    fn begin_gap_drag(&mut self, window: u32, x: i32, y: i32) -> Result<()> {
        let (first, second, axis, extent) = match self.band_for_window(window) {
            Some(handle) => (
                handle.first.clone(),
                handle.second.clone(),
                handle.axis,
                handle.extent.max(1.0),
            ),
            None => return Ok(()),
        };
        // A missed release leaves the previous gesture open; commit it before
        // starting another so its in-flight layout is not silently dropped.
        if self.drag.is_some() {
            self.end_gap_drag()?;
        }
        if std::env::var_os("PERGOLA_DEBUG").is_some() {
            eprintln!(
                "pergola: gap drag {first}|{second} axis {axis:?} extent {extent:.0}px"
            );
        }
        self.drag = Some(GapDrag {
            first,
            second,
            axis,
            extent,
            start: (x, y),
            snapshot: self.file.doc.root.clone(),
        });
        Ok(())
    }

    /// Move the dragged boundary: the pointer delta becomes a weight delta
    /// against the split's displayed length.
    fn update_gap_drag(&mut self, x: i32, y: i32) -> Result<()> {
        let Some(drag) = &self.drag else {
            return Ok(());
        };
        let delta_px = match drag.axis {
            Axis::X => (x - drag.start.0) as f64,
            Axis::Y => (y - drag.start.1) as f64,
        };
        let delta = delta_px / drag.extent;
        let snapshot = drag.snapshot.clone();
        let (first, second) = (drag.first.clone(), drag.second.clone());
        match self.engine.resizing_gap(&first, &second, delta, &snapshot) {
            Ok(tree) => {
                self.replace_root(tree);
                self.normalize_camera();
                self.apply()?;
            }
            Err(e) => {
                // Clamped at the minimum share: the layout is already as far as
                // it may go, so this is not an error the user needs to see.
                if std::env::var_os("PERGOLA_DEBUG").is_some() {
                    eprintln!("pergola: gap drag: {e}");
                }
            }
        }
        Ok(())
    }

    fn end_gap_drag(&mut self) -> Result<()> {
        if self.drag.take().is_none() {
            return Ok(());
        }
        // Rebuild the bands before saving: a failed write must not leave the
        // pointer without grab areas until the next apply.
        let bands = self.refresh_gap_bands();
        let saved = self.save();
        bands?;
        saved
    }

    /// Track a configure event that may be the user dragging a window. Our own
    /// moves are ignored (they match what we asked for, and echo right after an
    /// apply); a drag is a burst of changes followed by silence.
    fn note_configure(&mut self, event: &ConfigureNotifyEvent) -> Result<()> {
        if event.override_redirect || self.drag.is_some() {
            return Ok(());
        }
        if self.last_apply.elapsed() < Duration::from_millis(250) {
            return Ok(());
        }
        if self.seat_for_window(event.window).is_none() {
            return Ok(());
        }
        // The WM reports client coordinates; edge snapping is about the frame
        // the user sees, so add the decorations back.
        let (l, r, t, b) = self.x.frame_extents(event.window);
        let rect = Rect::new(
            (i32::from(event.x) - l) as f64,
            (i32::from(event.y) - t) as f64,
            (i32::from(event.width) + l + r) as f64,
            (i32::from(event.height) + t + b) as f64,
        );
        if let Some(applied) = self.applied.get(&event.window) {
            if same_rect(applied.to_rect(), rect) {
                return Ok(());
            }
        }
        let watch = self.moves.entry(event.window).or_insert(MoveWatch {
            moves: 0,
            at: Instant::now(),
            rect,
        });
        if !same_rect(watch.rect, rect) {
            watch.moves += 1;
            watch.rect = rect;
            watch.at = Instant::now();
        }
        Ok(())
    }

    /// A window that stopped moving after a real drag and sits against a screen
    /// edge is docked to that edge. This is the mouse path for tiling: drag a
    /// window to the left, right, top or bottom of the work area and let go.
    fn settle_moves(&mut self) -> Result<()> {
        if !self.config.edge_dock {
            self.moves.clear();
            return Ok(());
        }
        if self.drag.is_some() {
            return Ok(());
        }
        let now = Instant::now();
        let mut ready = Vec::new();
        for (window, watch) in &self.moves {
            if watch.moves >= 3 && now.duration_since(watch.at) >= Duration::from_millis(300) {
                ready.push((*window, watch.rect));
            }
        }
        if ready.is_empty() {
            return Ok(());
        }
        let screen = self.x.workarea_rect()?.to_rect();
        let world = screen;
        let desk = self
            .engine
            .rect_of(&self.file.doc.root, world, "desktop")
            .unwrap_or(world);
        // The drop rectangle is in screen space; a zoomed view must be undone
        // before it can be stored as a world position.
        let rects = self.engine.rects(&self.file.doc.root, world);
        let framed = if self.camera.framed != self.file.doc.root.id() {
            rects
                .iter()
                .find(|(id, _)| id == &self.camera.framed)
                .map(|(_, r)| *r)
        } else {
            None
        };
        for (window, rect) in ready {
            self.moves.remove(&window);
            let Some(seat) = self.seat_for_window(window).cloned() else {
                continue;
            };
            if self.file.doc.root.node(&seat.panel).is_some() {
                // A tile pulled off the tree: float it where it was dropped.
                if std::env::var_os("PERGOLA_DEBUG").is_some() {
                    eprintln!("pergola: drag out 0x{window:x} -> floating");
                }
                let dropped = into_world(untransform(rect, framed, world), Some(desk), world);
                if let Err(e) = self.float_window(window, Some(dropped)) {
                    eprintln!("pergola: drag out: {e:#}");
                }
                continue;
            }
            let Some(edge) = snap_edge(rect, screen, EDGE_SNAP_PX) else {
                continue;
            };
            if std::env::var_os("PERGOLA_DEBUG").is_some() {
                eprintln!("pergola: edge drop 0x{window:x} -> {edge:?}");
            }
            if let Err(e) = self.dock_window(window, edge, None) {
                eprintln!("pergola: edge dock: {e:#}");
            }
        }
        Ok(())
    }
}

/// How close to a screen edge a dragged window must be dropped to tile.
const EDGE_SNAP_PX: f64 = 14.0;

/// The screen edge a rectangle is sitting against, if any. Corners go to the
/// nearer edge; a window in the middle returns `None`.
fn snap_edge(rect: Rect, screen: Rect, slack: f64) -> Option<Direction> {
    let candidates = [
        (rect.x - screen.x, Direction::Left),
        (screen.max_x() - rect.max_x(), Direction::Right),
        (rect.y - screen.y, Direction::Up),
        (screen.max_y() - rect.max_y(), Direction::Down),
    ];
    candidates
        .into_iter()
        .filter(|(distance, _)| *distance <= slack)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, direction)| direction)
}

fn parse_command(name: &str, rest: &[String]) -> Result<Command> {
    match name {
        "neighbour" => {
            let dir = rest
                .first()
                .and_then(|s| Direction::parse(s))
                .ok_or_else(|| anyhow!("usage: pergola neighbour <left|right|up|down>"))?;
            Ok(Command::Neighbour(dir))
        }
        "view" | "save-view" => {
            let n: u8 = rest
                .first()
                .ok_or_else(|| anyhow!("usage: pergola {name} <1-9>"))?
                .parse()
                .map_err(|_| anyhow!("slot must be 1-9"))?;
            if !(1..=9).contains(&n) {
                return Err(anyhow!("slot must be 1-9"));
            }
            Ok(if name == "view" {
                Command::View(n)
            } else {
                Command::SaveView(n)
            })
        }
        other => Command::parse(other).map_err(|e| anyhow!(e)),
    }
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn parse_window(raw: &str) -> Option<u32> {
    let raw = raw.trim();
    if let Some(hex) = raw.strip_prefix("0x") {
        u32::from_str_radix(hex, 16).ok()
    } else {
        raw.parse().ok()
    }
}

/// Camera transform: when a node is framed, scale the world so that node fills
/// the screen. Everything else moves with it, which is what makes the zoom
/// continuous rather than a workspace switch.
fn transform(rect: Rect, framed: Option<Rect>, screen: Rect) -> Rect {
    let Some(f) = framed else { return rect };
    if f.w <= 0.0 || f.h <= 0.0 {
        return rect;
    }
    // Non-uniform: the framed node maps exactly onto the screen, so framing a
    // window behaves like maximising it, and everything else moves with it.
    let sx = screen.w / f.w;
    let sy = screen.h / f.h;
    // The framed node fills the *work area*, so a work area that does not start
    // at 0,0 (a top panel) lands where the window manager expects it.
    Rect::new(
        screen.x + (rect.x - f.x) * sx,
        screen.y + (rect.y - f.y) * sy,
        rect.w * sx,
        rect.h * sy,
    )
}

/// How many screen pixels one world pixel became, along the axis a boundary
/// moves on. The camera transform is non-uniform, so X and Y differ.
fn axis_scale(axis: Axis, world: Rect, shown: Rect) -> f64 {
    match axis {
        Axis::X if world.w > 0.0 => shown.w / world.w,
        Axis::Y if world.h > 0.0 => shown.h / world.h,
        _ => 1.0,
    }
}

/// The inverse of [`transform`]: a screen rectangle back into world space.
fn untransform(rect: Rect, framed: Option<Rect>, screen: Rect) -> Rect {
    let Some(f) = framed else { return rect };
    if f.w <= 0.0 || f.h <= 0.0 || screen.w <= 0.0 || screen.h <= 0.0 {
        return rect;
    }
    let sx = screen.w / f.w;
    let sy = screen.h / f.h;
    Rect::new(
        f.x + (rect.x - screen.x) / sx,
        f.y + (rect.y - screen.y) / sy,
        rect.w / sx,
        rect.h / sy,
    )
}

/// Ids on the path from the root to `target`, inclusive.
fn ancestors<'a>(root: &'a LayoutNode, target: &str) -> Vec<&'a str> {
    fn walk<'a>(node: &'a LayoutNode, target: &str, acc: &mut Vec<&'a str>) -> bool {
        acc.push(node.id());
        if node.id() == target {
            return true;
        }
        for child in node.children() {
            if walk(child, target, acc) {
                return true;
            }
        }
        acc.pop();
        false
    }
    let mut acc = Vec::new();
    if walk(root, target, &mut acc) {
        acc
    } else {
        Vec::new()
    }
}

/// Bars that visualise the tree: one 2 px line per split boundary, centred in
/// the gap, plus a 3 px frame around the viewport when a node is framed.
///
/// Lines live in the gaps, so the overlay never covers a window's title bar or
/// border — the previous design drew a border around every tile, which read as
/// a debug artefact on top of the decorations.
fn overlay_bars(
    root: &LayoutNode,
    rects: &[(String, Rect)],
    framed: Option<&str>,
    gap: f64,
    viewport: Rect,
) -> Vec<(IRect, u32)> {
    const LINE: u32 = 0x00505f6b;
    const ACCENT: u32 = 0x00e8b04b;
    let path = framed.map(|f| ancestors(root, f)).unwrap_or_default();
    let mut bars = Vec::new();
    for node in root.flatten() {
        let LayoutNode::Split(split) = node else { continue };
        let Some((_, rect)) = rects.iter().find(|(id, _)| id == &split.id) else {
            continue;
        };
        let accent = framed.is_some() && path.contains(&split.id.as_str());
        let color = if accent { ACCENT } else { LINE };
        for i in 0..split.children.len().saturating_sub(1) {
            // A zero-weight child (a collapsed desktop leaf) has no boundary to
            // draw: the line would sit on the screen edge for nothing.
            if split.weights.get(i).copied().unwrap_or(0.0) <= 0.0
                || split.weights.get(i + 1).copied().unwrap_or(0.0) <= 0.0
            {
                continue;
            }
            let Some((_, child)) = rects.iter().find(|(id, _)| id == split.children[i].id()) else {
                continue;
            };
            match split.axis {
                Axis::X => {
                    let x = child.max_x() + gap / 2.0 - 1.0;
                    bars.push((
                        IRect::new(x.round() as i32, rect.y.round() as i32, 2, rect.h.round() as i32),
                        color,
                    ));
                }
                Axis::Y => {
                    let y = child.max_y() + gap / 2.0 - 1.0;
                    bars.push((
                        IRect::new(rect.x.round() as i32, y.round() as i32, rect.w.round() as i32, 2),
                        color,
                    ));
                }
            }
        }
    }
    if framed.is_some() {
        let t = 3;
        let v = viewport.to_irect();
        bars.push((IRect::new(v.x, v.y, v.w, t), ACCENT));
        bars.push((IRect::new(v.x, v.y + v.h - t, v.w, t), ACCENT));
        bars.push((IRect::new(v.x, v.y, t, v.h), ACCENT));
        bars.push((IRect::new(v.x + v.w - t, v.y, t, v.h), ACCENT));
    }
    bars
}

/// A window that is entirely outside the framed view is parked *below* the work
/// area. Parking to the side does not work: xfwm4 (and most WMs) clamp a window
/// so a strip of its title bar stays reachable, which leaves a visible sliver at
/// the screen edge. Below the work area is not clamped.
fn park_if_outside(target: IRect, viewport: IRect, zoomed: bool) -> IRect {
    if !zoomed {
        return target;
    }
    let visible = target.x < viewport.x + viewport.w
        && target.x + target.w > viewport.x
        && target.y < viewport.y + viewport.h
        && target.y + target.h > viewport.y;
    if visible {
        target
    } else {
        IRect::new(viewport.x, viewport.y + viewport.h, target.w, target.h)
    }
}

fn same_rect(a: Rect, b: Rect) -> bool {
    (a.x - b.x).abs() < 0.5
        && (a.y - b.y).abs() < 0.5
        && (a.w - b.w).abs() < 0.5
        && (a.h - b.h).abs() < 0.5
}

/// Map a world rect (0..screen) into the rectangle the desktop leaf occupies.
fn into_screen(rect: Rect, desk: Option<Rect>, world: Rect) -> Rect {
    let Some(d) = desk else { return rect };
    if world.w <= 0.0 || world.h <= 0.0 || d.w <= 0.0 || d.h <= 0.0 {
        return rect;
    }
    if (d.x - world.x).abs() < 1.0
        && (d.y - world.y).abs() < 1.0
        && (d.w - world.w).abs() < 1.0
        && (d.h - world.h).abs() < 1.0
    {
        return rect;
    }
    let sx = d.w / world.w;
    let sy = d.h / world.h;
    Rect::new(
        d.x + (rect.x - world.x) * sx,
        d.y + (rect.y - world.y) * sy,
        rect.w * sx,
        rect.h * sy,
    )
}

/// The inverse of [`into_screen`]: a live screen rect back into world space.
fn into_world(rect: Rect, desk: Option<Rect>, world: Rect) -> Rect {
    let Some(d) = desk else { return rect };
    if world.w <= 0.0 || world.h <= 0.0 || d.w <= 0.0 || d.h <= 0.0 {
        return rect;
    }
    if (d.x - world.x).abs() < 1.0
        && (d.y - world.y).abs() < 1.0
        && (d.w - world.w).abs() < 1.0
        && (d.h - world.h).abs() < 1.0
    {
        return rect;
    }
    let sx = world.w / d.w;
    let sy = world.h / d.h;
    Rect::new(
        world.x + (rect.x - d.x) * sx,
        world.y + (rect.y - d.y) * sy,
        rect.w * sx,
        rect.h * sy,
    )
}

fn build_keymap(x: &X11) -> Result<Vec<(u8, u32)>> {
    use x11rb::protocol::xproto::ConnectionExt as _;
    let min = x.conn.setup().min_keycode;
    let max = x.conn.setup().max_keycode;
    let count = (max - min + 1) as u8;
    let mapping = x.conn.get_keyboard_mapping(min, count)?.reply()?;
    let per = mapping.keysyms_per_keycode as usize;
    let mut out = Vec::new();
    for (i, chunk) in mapping.keysyms.chunks(per).enumerate() {
        if let Some(sym) = chunk.first() {
            out.push((min + i as u8, *sym));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zoomed_boundary_uses_its_own_axis_scale() {
        // A split 1600x400 on a 1600x800 screen: x is 1:1, y doubles.
        let world = Rect::new(800.0, 0.0, 6.0, 400.0);
        let shown = Rect::new(800.0, 0.0, 6.0, 800.0);
        assert!((axis_scale(Axis::X, world, shown) - 1.0).abs() < 1e-9);
        assert!((axis_scale(Axis::Y, world, shown) - 2.0).abs() < 1e-9);
        // Degenerate bands fall back to 1 instead of dividing by zero.
        assert_eq!(axis_scale(Axis::X, Rect::new(0.0, 0.0, 0.0, 0.0), shown), 1.0);
    }

    #[test]
    fn untransform_undoes_the_camera() {
        let screen = Rect::new(0.0, 27.0, 1600.0, 873.0);
        let framed = Rect::new(400.0, 200.0, 800.0, 400.0);
        for rect in [
            Rect::new(400.0, 200.0, 800.0, 400.0),
            Rect::new(10.0, 900.0, 120.0, 60.0),
        ] {
            let shown = transform(rect, Some(framed), screen);
            let back = untransform(shown, Some(framed), screen);
            assert!((back.x - rect.x).abs() < 1e-6, "x {:?}", back);
            assert!((back.y - rect.y).abs() < 1e-6, "y {:?}", back);
            assert!((back.w - rect.w).abs() < 1e-6, "w {:?}", back);
            assert!((back.h - rect.h).abs() < 1e-6, "h {:?}", back);
        }
    }

    #[test]
    fn a_drop_near_an_edge_snaps_to_it() {
        let screen = Rect::new(0.0, 27.0, 1600.0, 873.0);
        let left = Rect::new(2.0, 300.0, 400.0, 300.0);
        let right = Rect::new(1198.0, 300.0, 400.0, 300.0);
        let top = Rect::new(500.0, 29.0, 400.0, 300.0);
        let bottom = Rect::new(500.0, 596.0, 400.0, 300.0);
        let middle = Rect::new(600.0, 300.0, 400.0, 300.0);
        assert_eq!(snap_edge(left, screen, EDGE_SNAP_PX), Some(Direction::Left));
        assert_eq!(snap_edge(right, screen, EDGE_SNAP_PX), Some(Direction::Right));
        assert_eq!(snap_edge(top, screen, EDGE_SNAP_PX), Some(Direction::Up));
        assert_eq!(snap_edge(bottom, screen, EDGE_SNAP_PX), Some(Direction::Down));
        assert_eq!(snap_edge(middle, screen, EDGE_SNAP_PX), None);
    }

    #[test]
    fn a_corner_drop_picks_the_nearer_edge() {
        let screen = Rect::new(0.0, 0.0, 1000.0, 800.0);
        // 3 px from the top, 9 px from the left: the top wins.
        let rect = Rect::new(9.0, 3.0, 200.0, 200.0);
        assert_eq!(snap_edge(rect, screen, EDGE_SNAP_PX), Some(Direction::Up));
    }

    #[test]
    fn transform_is_identity_without_a_frame() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(transform(r, None, Rect::new(0.0, 0.0, 100.0, 100.0)), r);
    }

    #[test]
    fn transform_fills_the_screen_with_the_framed_node() {
        let screen = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let framed = Rect::new(100.0, 50.0, 500.0, 250.0);
        let out = transform(framed, Some(framed), screen);
        assert!((out.x - 0.0).abs() < 1e-9);
        assert!((out.y - 0.0).abs() < 1e-9);
        assert!((out.w - 1000.0).abs() < 1e-9);
        assert!((out.h - 500.0).abs() < 1e-9);
    }

    #[test]
    fn transform_keeps_relative_positions() {
        let screen = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let framed = Rect::new(0.0, 0.0, 500.0, 250.0);
        let sibling = Rect::new(500.0, 0.0, 500.0, 250.0);
        let out = transform(sibling, Some(framed), screen);
        assert!((out.x - 1000.0).abs() < 1e-9);
        assert!((out.w - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn transform_fills_the_work_area_not_the_display() {
        // 1600x900 display with a 27 px top panel: the tile must land at y=27
        // and be 873 tall, or the bottom of every window sits off-screen.
        let work = Rect::new(0.0, 27.0, 1600.0, 873.0);
        let framed = Rect::new(0.0, 27.0, 800.0, 873.0);
        let out = transform(framed, Some(framed), work);
        assert!((out.y - 27.0).abs() < 1e-9);
        assert!((out.h - 873.0).abs() < 1e-9);
        let right = Rect::new(808.0, 27.0, 800.0, 873.0);
        let out = transform(right, Some(framed), work);
        assert!((out.x - 1616.0).abs() < 1e-9);
    }

    #[test]
    fn desktop_mapping_round_trips_in_an_offset_work_area() {
        let world = Rect::new(0.0, 27.0, 1600.0, 873.0);
        let desk = Rect::new(0.0, 27.0, 1600.0, 873.0);
        let rect = Rect::new(100.0, 200.0, 400.0, 300.0);
        assert_eq!(into_screen(rect, Some(desk), world), rect);
        assert_eq!(into_world(rect, Some(desk), world), rect);
    }

    #[test]
    fn transform_fills_both_axes() {
        let screen = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let framed = Rect::new(100.0, 100.0, 200.0, 100.0);
        let out = transform(framed, Some(framed), screen);
        assert!((out.x).abs() < 1e-9);
        assert!((out.y).abs() < 1e-9);
        assert!((out.w - 1000.0).abs() < 1e-9);
        assert!((out.h - 500.0).abs() < 1e-9);
    }

    #[test]
    fn desktop_mapping_round_trips() {
        let world = Rect::new(0.0, 0.0, 1600.0, 900.0);
        let desk = Rect::new(804.0, 0.0, 796.0, 900.0);
        let rect = Rect::new(100.0, 200.0, 400.0, 300.0);
        let screen = into_screen(rect, Some(desk), world);
        assert!((screen.x - (804.0 + 100.0 * 796.0 / 1600.0)).abs() < 1e-9);
        assert!((screen.w - 400.0 * 796.0 / 1600.0).abs() < 1e-9);
        let back = into_world(screen, Some(desk), world);
        assert!((back.x - rect.x).abs() < 1e-6);
        assert!((back.y - rect.y).abs() < 1e-6);
        assert!((back.w - rect.w).abs() < 1e-6);
        assert!((back.h - rect.h).abs() < 1e-6);
    }

    #[test]
    fn full_screen_desktop_is_identity() {
        let world = Rect::new(0.0, 0.0, 1600.0, 900.0);
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(into_screen(rect, Some(world), world), rect);
        assert_eq!(into_world(rect, Some(world), world), rect);
    }

    #[test]
    fn overlay_lines_sit_in_the_gaps() {
        let engine = LayoutEngine::new(8.0, 0.05);
        let tree = LayoutNode::split(
            "root",
            Axis::X,
            vec![
                LayoutNode::panel("p-a", vec!["s-a".into()]),
                LayoutNode::panel("p-b", vec!["s-b".into()]),
            ],
        );
        let world = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let rects = engine.rects(&tree, world);
        let bars = overlay_bars(&tree, &rects, None, 8.0, world);
        assert_eq!(bars.len(), 1, "one boundary for two children");
        let (bar, _) = bars[0];
        // a is 0..496, the gap is 496..504, so the line is centred at 500.
        assert_eq!(bar.x, 499);
        assert_eq!(bar.w, 2);
        assert_eq!(bar.h, 500);
        // No bar may overlap a tile: the line is inside the gap.
        assert!(bar.x >= 496 && bar.x + bar.w <= 504);
    }

    #[test]
    fn overlay_marks_the_framed_path_and_the_viewport() {
        let engine = LayoutEngine::new(8.0, 0.05);
        let inner = LayoutNode::split(
            "inner",
            Axis::Y,
            vec![
                LayoutNode::panel("p-b", vec!["s-b".into()]),
                LayoutNode::panel("p-c", vec!["s-c".into()]),
            ],
        );
        let tree = LayoutNode::split(
            "root",
            Axis::X,
            vec![LayoutNode::panel("p-a", vec!["s-a".into()]), inner],
        );
        let world = Rect::new(0.0, 0.0, 1000.0, 500.0);
        let rects = engine.rects(&tree, world);
        let bars = overlay_bars(&tree, &rects, Some("p-c"), 8.0, world);
        // root boundary + inner boundary + 4 viewport edges
        assert_eq!(bars.len(), 6);
        let accent = bars.iter().filter(|(_, c)| *c == 0x00e8b04b).count();
        assert_eq!(accent, 6, "both splits are on the path, plus the frame");
    }

    #[test]
    fn windows_outside_the_view_are_parked_below_it() {
        let view = IRect::new(0, 27, 1600, 873);
        // fully to the right of the framed node
        let right = IRect::new(1700, 27, 400, 400);
        assert_eq!(park_if_outside(right, view, true), IRect::new(0, 900, 400, 400));
        // fully above
        let above = IRect::new(0, -500, 400, 400);
        assert_eq!(park_if_outside(above, view, true), IRect::new(0, 900, 400, 400));
        // partially visible stays put: a zoom may legitimately clip a tile
        let partial = IRect::new(1500, 27, 400, 400);
        assert_eq!(park_if_outside(partial, view, true), partial);
        // not zoomed: never park
        assert_eq!(park_if_outside(right, view, false), right);
    }

    #[test]
    fn window_ids_parse_in_both_bases() {
        assert_eq!(parse_window("0x1a2b"), Some(0x1a2b));
        assert_eq!(parse_window("6699"), Some(6699));
        assert_eq!(parse_window("nope"), None);
    }
}
