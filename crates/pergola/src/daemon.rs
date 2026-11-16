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
use pergola_core::engine::replace_node;
use pergola_core::{
    Axis, Camera, CameraAction, Command, Direction, FloatingWindow, GapHandle, IRect, LayoutEngine,
    LayoutFile, LayoutNode, Rect, SeatRecord,
};
use pergola_x11::{
    command_for_key, describe, event_mods, snap_to_hints, KeyBinding, WindowInfo, X11,
};
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
    /// Configure events that actually changed the geometry since the window
    /// left the position we last applied.
    moves: u32,
    at: Instant,
    rect: Rect,
    /// Where the window was when it was first seen moving: a drag is judged by
    /// the distance travelled, not only by how many events it produced.
    start: Rect,
}

pub struct Daemon {
    x: X11,
    engine: LayoutEngine,
    file: LayoutFile,
    camera: pergola_core::CameraState,
    config: Config,
    overlay_on: bool,
    bindings: Vec<(KeyBinding, String)>,
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
    /// The last `KeyRelease` seen: keycode and server timestamp. The server
    /// auto-repeats a held key by sending a `KeyRelease` and a `KeyPress` with
    /// the *same* timestamp, so that pair is a repeat and the press is dropped.
    last_release: Option<(u8, u32)>,
    /// False until the daemon's first reconcile finishes. `auto_dock: new`
    /// docks windows that appear after this, `all` also adopts the ones that
    /// were already on screen at startup.
    started: bool,
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
        let bindings: Vec<(KeyBinding, String)> = Vec::new();
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
            last_release: None,
            started: false,
        };
        daemon.reconcile()?;
        daemon.started = true;
        daemon.normalize_camera();
        daemon.apply()?;
        Ok(daemon)
    }

    // MARK: reconciliation

    /// Match the tree against the live client list: drop panels whose window is
    /// gone, adopt new windows, refresh titles.
    fn reconcile(&mut self) -> Result<bool> {
        let windows = self.x.windows()?;
        // `_NET_CLIENT_LIST` still contains a minimised window (and windows on
        // the host WM's other desktops) even though it is unmapped. A seat is
        // gone only when its window leaves the client list: dropping it because
        // the window was merely minimised collapses the tile and loses the
        // layout — i3#718, awesome#92.
        let clients: HashSet<u32> = self.x.client_list()?.into_iter().collect();

        let gone: Vec<SeatRecord> = self
            .seats
            .iter()
            .filter(|s| !clients.contains(&s.window))
            .cloned()
            .collect();
        let mut changed = !gone.is_empty();
        for seat in gone {
            self.applied.remove(&seat.window);
            self.moves.remove(&seat.window);
            self.seats.retain(|s| s.seat != seat.seat);
            // A tab group loses one view, not the whole panel: closing a tab
            // must leave its siblings where they are. A panel whose last view
            // goes collapses through the same path.
            self.detach_seat(&seat)?;
            self.file.doc.floating.retain(|f| f.panel.id != seat.panel);
        }

        for w in &windows {
            if w.hidden {
                // Minimised: keep the seat and its title, but do not treat the
                // geometry as the user's.
                if let Some(record) = self.seats.iter_mut().find(|s| s.window == w.id) {
                    record.title = w.title.clone();
                    record.class = w.class.clone();
                }
                continue;
            }
            // A dialog or popup (`WM_TRANSIENT_FOR`) belongs to its parent and
            // the WM places it. Adopting it would create a seat for a window
            // that `apply` then refuses to move, leaving an empty panel in the
            // tree and a tile with nothing in it — awesome#22, i3#787.
            if w.transient {
                continue;
            }
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
            if self.config.auto_dock.docks(self.started) {
                // The WM may have already focused the new window, so
                // `focused_panel` can name a panel that is not in the tree yet
                // (its own). Dock beside a panel that is actually there, or the
                // whole tree when nothing docked has focus.
                let target = self
                    .focused_panel()
                    .filter(|p| self.file.doc.root.node(p).is_some())
                    .or_else(|| {
                        self.last_focus
                            .clone()
                            .filter(|p| self.file.doc.root.node(p).is_some())
                    })
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

    /// Take one seat out of the tree wherever it lives: out of a tab group
    /// (only that view — its siblings stay), out of a plain panel (which then
    /// leaves the tree like any node), or off the desktop layer. The seat
    /// record stays, so a caller can put the window back somewhere else.
    fn detach_seat(&mut self, seat: &SeatRecord) -> Result<()> {
        self.file
            .doc
            .floating
            .retain(|f| !f.panel.views.iter().any(|v| v == &seat.seat));
        // `removing_view` handles both cases: a group loses one view, and an
        // empty panel collapses through the same rules as `removing`.
        if let Ok(tree) = self.engine.removing_view(&seat.seat, &self.file.doc.root) {
            self.replace_root(tree);
        }
        Ok(())
    }

    /// A panel id the seat can own outright. A seat leaving a tab group needs
    /// a fresh id because the group keeps its own; a seat that owned its panel
    /// can keep the id. The seat record is updated so `focused_panel` and
    /// `list` stay truthful afterwards.
    fn own_panel_id(&mut self, seat: &SeatRecord) -> String {
        let taken = |id: &str, root: &LayoutNode| -> bool { root.node(id).is_some() };
        let mut id = seat.panel.clone();
        if taken(&id, &self.file.doc.root)
            || self.file.doc.floating.iter().any(|f| f.panel.id == id)
        {
            let base = format!("p-{:x}", seat.window);
            id = base.clone();
            let mut n = 1;
            while taken(&id, &self.file.doc.root)
                || self.file.doc.floating.iter().any(|f| f.panel.id == id)
            {
                id = format!("{base}-{n}");
                n += 1;
            }
        }
        if let Some(record) = self.seats.iter_mut().find(|s| s.seat == seat.seat) {
            record.panel = id.clone();
        }
        id
    }

    // MARK: applying geometry

    fn apply(&mut self) -> Result<()> {
        let screen = self.x.workarea_rect()?;
        // No usable output geometry yet (server still starting, monitor asleep,
        // RandR mid-change): every rect computed from a 0x0 world divides by
        // zero and collapses the whole tree. Leave the last geometry alone and
        // let the next reconcile try again.
        if screen.w <= 0 || screen.h <= 0 {
            return Ok(());
        }
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
                let Some(seat) = self.seats.iter().find(|s| &s.seat == view) else {
                    continue;
                };
                let Ok(Some(info)) = self.x.info(seat.window) else {
                    continue;
                };
                // A fullscreen window owns its geometry: the WM has taken it
                // over and moving it would fight the user's F11. A transient
                // window (dialog, popup) belongs to its parent, and tiling it
                // lands it wherever the parent happens to be — awesome#22.
                // Leave both alone until the state clears.
                if info.fullscreen || info.transient {
                    continue;
                }
                // A maximized window ignores `_NET_MOVERESIZE_WINDOW`: the
                // tile would keep the full-screen geometry and the tree would
                // lie. Drop the state, then move.
                if info.maximized {
                    let _ = self.x.unmaximize(seat.window);
                }
                // Snap the client area to its own resize grid, so a terminal
                // lands flush against the tile instead of being clamped by the
                // WM to an arbitrary shortfall.
                let target = snap_to_hints(target, self.x.size_hints(seat.window), info.extents);
                let _ = self.x.move_resize(seat.window, target);
                self.applied.insert(seat.window, target);
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

    /// Copy the live seat records into the document's metadata. `tree` reads
    /// the document, not the daemon's own `seats`, so a title or class change
    /// handled by `PropertyNotify` must reach the metadata or the read-only
    /// `tree` view keeps showing the name the window had when it was adopted.
    fn sync_meta(&mut self) {
        let meta = pergola_core::PergolaMeta {
            seats: self.seats.clone(),
            gap: Some(self.engine.gap),
            auto_dock: Some(self.config.auto_dock != crate::config::AutoDock::Off),
            overlay: Some(self.overlay_on),
        };
        self.file.pergola = Some(meta);
    }

    fn save(&mut self) -> Result<()> {
        self.file.set_camera(&self.camera);
        self.sync_meta();
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
            "tree" => {
                self.sync_meta();
                Ok(self.file.to_json())
            }
            "list" => self.list(),
            "rects" => self.rects(rest),
            "dock" => self.dock(rest),
            "float" => self.float(rest),
            "arrange" => self.arrange(),
            "float-all" => self.float_all(),
            "gap" => self.set_gap(rest),
            "ratio" => self.set_ratio(rest),
            "overlay" => self.overlay(rest),
            "tab" => self.tab(rest),
            "tab-select" => self.tab_select(rest),
            "tab-next" => self.tab_cycle(true),
            "tab-prev" => self.tab_cycle(false),
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
        // Views beyond the first of each panel are tabs: mapped at the same
        // rectangle, only one of them raised.
        let tabs: usize = self
            .file
            .doc
            .root
            .flatten()
            .iter()
            .filter_map(|n| match n {
                LayoutNode::Panel(p) => Some(p.views.len().saturating_sub(1)),
                _ => None,
            })
            .sum();
        let tab_note = if tabs > 0 {
            format!(" · {tabs} tabs")
        } else {
            String::new()
        };
        Ok(format!(
            "pergola {} · {}x{} · {} panels, {} floating{tab_note} · depth {} · framed {} · gap {} · overlay {}",
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
                Some(s) if self.file.doc.root.node(&s.panel).is_some() => {
                    // A view of a tab group that is not the selected one is
                    // mapped at the tile but covered: report it as a tab, not
                    // as a tile someone is looking at.
                    let hidden = match self.file.doc.root.node(&s.panel) {
                        Some(LayoutNode::Panel(p)) => p.visible() != Some(s.seat.as_str()),
                        _ => false,
                    };
                    if hidden { "tab" } else { "docked" }
                }
                Some(_) => "floating",
                None => "unmanaged",
            };
            out.push_str(&format!("{:<9} {}\n", where_, describe(w)));
        }
        // Minimised windows are unmapped, so `windows()` does not list them,
        // but their seats are still in the tree. Say so instead of letting the
        // user think the tile vanished.
        for seat in &self.seats {
            if !windows.iter().any(|w| w.id == seat.window) {
                out.push_str(&format!("{:<9} 0x{:08x}  {}\n", "hidden", seat.window, seat.title));
            }
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
        self.detach_seat(&seat)?;
        let panel_id = self.own_panel_id(&seat);
        if self.file.doc.root.node(&target).is_none() {
            target = self.file.doc.root.id().to_string();
        }
        let node = LayoutNode::panel(panel_id.clone(), vec![seat.seat.clone()]);
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
            panel_id,
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
            // Already floating: keep the rectangle the user put it in. Falling
            // back to the default 80x80x800x500 here teleports the window to
            // the top-left corner, where a WM's own "drag to the edge" gesture
            // then fires (measured 2026-11-15: `float` on an already-floating
            // xterm under openbox jumped it to the top edge and openbox
            // maximised it).
            .or_else(|| {
                self.file
                    .doc
                    .floating
                    .iter()
                    .find(|f| f.panel.id == seat.panel)
                    .map(|f| f.rect)
            })
            .or_else(|| {
                self.engine
                    .rect_of(
                        &self.file.doc.root,
                        self.x.workarea_rect().ok()?.to_rect(),
                        &seat.panel,
                    )
            })
            .unwrap_or_else(|| Rect::new(80.0, 80.0, 800.0, 500.0));
        self.detach_seat(&seat)?;
        let panel_id = self.own_panel_id(&seat);
        self.file.doc.floating.push(FloatingWindow {
            panel: pergola_core::Panel::new(panel_id.clone(), vec![seat.seat.clone()]),
            rect,
            size: pergola_core::Size::new(rect.w as i32, rect.h as i32),
            z: None,
        });
        self.place_floating = true;
        self.normalize_camera();
        self.apply()?;
        self.save()?;
        Ok(format!("floated {panel_id}"))
    }

    /// Move a window into a panel as a tab. Without `--target` the window joins
    /// the panel that had focus before it — so `pergola tab` on the active
    /// window folds it into the neighbouring panel instead of doing nothing.
    /// A panel that gains a second view becomes a tab group; only its selected
    /// view is raised.
    fn tab(&mut self, args: &[String]) -> Result<String> {
        let win = flag(args, "--window")
            .and_then(|v| parse_window(&v))
            .or_else(|| self.x.active_window())
            .ok_or_else(|| anyhow!("no active window to tab"))?;
        let seat = self
            .seat_for_window(win)
            .ok_or_else(|| anyhow!("window 0x{win:x} is not managed by pergola"))?
            .clone();
        // A seat already in a panel is a tab move; a floating seat is folded in
        // from the desktop layer. `panel_containing` is None for the latter and
        // `detach_seat` removes it from the floating layer, so the same code
        // folds both into the target panel.
        let source = self.file.doc.root.panel_containing(&seat.seat).cloned();
        let source_id = source.as_ref().map(|p| p.id.clone());
        let target = match flag(args, "--target") {
            Some(t) => t,
            None => match self.focused_panel() {
                Some(p) if Some(&p) != source_id.as_ref() && self.file.doc.root.node(&p).is_some() => {
                    p
                }
                _ => self
                    .last_focus
                    .clone()
                    .filter(|p| {
                        Some(p) != source_id.as_ref() && self.file.doc.root.node(p).is_some()
                    })
                    .or_else(|| {
                        // A floating window has no panel of its own and may have
                        // no last-focused tile; fall back to the first leaf so the
                        // command folds it somewhere sensible instead of failing.
                        self.file
                            .doc
                            .root
                            .leaves()
                            .into_iter()
                            .find(|n| !n.is_desktop())
                            .map(|n| n.id().to_string())
                    })
                    .unwrap_or_else(|| self.file.doc.root.id().to_string()),
            },
        };
        let target_id = if self.file.doc.root.node(&target).is_some() {
            target
        } else {
            self.file
                .doc
                .root
                .panel_containing(&target)
                .map(|p| p.id.clone())
                .ok_or_else(|| anyhow!("no panel '{target}' to tab into"))?
        };
        if Some(&target_id) == source_id.as_ref() {
            // Already in the target panel: show it rather than rebuilding.
            let Some(mut panel) = source else {
                return Err(anyhow!("no panel '{target_id}' to tab into"));
            };
            panel.select(&seat.seat);
            let tree = replace_node(&self.file.doc.root, &target_id, &LayoutNode::Panel(panel));
            self.replace_root(tree);
            self.apply()?;
            self.save()?;
            return Ok(format!("showing {} in {}", seat.seat, target_id));
        }
        // Take the seat out of its old panel. A tab group loses one view; a
        // plain panel goes away like any other node.
        self.detach_seat(&seat)?;
        let Some(LayoutNode::Panel(mut target_panel)) =
            self.file.doc.root.node(&target_id).cloned()
        else {
            return Err(anyhow!("no panel '{target_id}' to tab into"));
        };
        if !target_panel.views.iter().any(|v| v == &seat.seat) {
            target_panel.views.push(seat.seat.clone());
        }
        target_panel.select(&seat.seat);
        target_panel.tabbed = Some(true);
        let views = target_panel.views.len();
        let tree = replace_node(&self.file.doc.root, &target_id, &LayoutNode::Panel(target_panel));
        self.replace_root(tree);
        // The seat now belongs to the target panel; without this `focused_panel`
        // and `list` keep pointing at the panel it just left.
        if let Some(record) = self.seats.iter_mut().find(|s| s.seat == seat.seat) {
            record.panel = target_id.clone();
        }
        self.normalize_camera();
        self.apply()?;
        // The tabbed window is the one being shown, so give it the keyboard.
        self.focus_seat(&seat.seat);
        self.save()?;
        Ok(format!(
            "tabbed {} into {target_id} ({views} views)",
            seat.seat
        ))
    }

    /// Show one view of a tab group. Takes a seat id or a window id.
    fn tab_select(&mut self, args: &[String]) -> Result<String> {
        let raw = args
            .first()
            .ok_or_else(|| anyhow!("usage: pergola tab-select <seat|window>"))?;
        let seat = match self.seats.iter().find(|s| &s.seat == raw) {
            Some(s) => s.seat.clone(),
            None => {
                let win = parse_window(raw)
                    .ok_or_else(|| anyhow!("'{raw}' is not a seat or window id"))?;
                self.seat_for_window(win)
                    .ok_or_else(|| anyhow!("window 0x{win:x} is not managed by pergola"))?
                    .seat
                    .clone()
            }
        };
        let panel = self
            .file
            .doc
            .root
            .panel_containing(&seat)
            .cloned()
            .ok_or_else(|| anyhow!("{seat} is not in the layout"))?;
        if !panel.is_tab_group() {
            return Ok(format!("{} is not a tab group", panel.id));
        }
        let panel_id = panel.id.clone();
        let mut panel = panel;
        if !panel.select(&seat) {
            return Err(anyhow!("{seat} is not a view of {panel_id}"));
        }
        let tree = replace_node(&self.file.doc.root, &panel_id, &LayoutNode::Panel(panel));
        self.replace_root(tree);
        self.apply()?;
        self.focus_seat(&seat);
        self.save()?;
        Ok(format!("showing {seat} in {panel_id}"))
    }

    /// Cycle the visible view of the focused tab group.
    fn tab_cycle(&mut self, forward: bool) -> Result<String> {
        // The group to cycle is the one the user is looking at. That is the
        // panel of the active window when it is a tab group; when the WM has
        // parked focus elsewhere (a new window, a dialog) fall back to the
        // group focused last. Both are resolved from the tree, never from the
        // seat record, which can lag a move.
        let mut panel_id = self
            .focused_panel()
            .filter(|id| self.is_tab_group(id))
            .or_else(|| {
                self.last_focus
                    .clone()
                    .filter(|id| self.is_tab_group(id))
            });
        if panel_id.is_none() {
            // Last resort: any tab group at all. A user pressing tab-next with
            // one group on screen means that group.
            panel_id = self
                .file
                .doc
                .root
                .flatten()
                .into_iter()
                .find_map(|n| match n {
                    LayoutNode::Panel(p) if p.is_tab_group() => Some(p.id.clone()),
                    _ => None,
                });
        }
        let Some(panel_id) = panel_id else {
            return Ok("no tab group to cycle".to_string());
        };
        let Some(LayoutNode::Panel(mut panel)) = self.file.doc.root.node(&panel_id).cloned() else {
            return Ok(format!("{panel_id} is not a panel"));
        };
        let Some(seat) = panel.cycle(forward) else {
            return Ok(format!("{panel_id} has no views"));
        };
        let tree = replace_node(&self.file.doc.root, &panel_id, &LayoutNode::Panel(panel));
        self.replace_root(tree);
        self.apply()?;
        self.focus_seat(&seat);
        self.save()?;
        Ok(format!("showing {seat} in {panel_id}"))
    }

    /// Whether a panel id names a tab group in the tree.
    fn is_tab_group(&self, id: &str) -> bool {
        matches!(
            self.file.doc.root.node(id),
            Some(LayoutNode::Panel(p)) if p.is_tab_group()
        )
    }

    /// Give the keyboard to a seat's window. Switching tabs without this leaves
    /// the user typing into the hidden view, which is the bug the whole feature
    /// exists to avoid.
    fn focus_seat(&self, seat: &str) {
        if let Some(record) = self.seats.iter().find(|s| s.seat == seat) {
            if let Err(e) = self.x.activate(record.window) {
                eprintln!("pergola: could not focus 0x{:x}: {e}", record.window);
            }
        }
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

    /// Rebalance the split that contains the focused panel: `ratio 0.3` gives
    /// the focused window 30 % of its split. The siblings keep their relative
    /// proportions, so a two-child split moves the boundary to exactly 30 % —
    /// the same result as dragging the gap by the same amount.
    fn set_ratio(&mut self, args: &[String]) -> Result<String> {
        let ratio: f64 = args
            .first()
            .ok_or_else(|| anyhow!("usage: pergola ratio <0.0-1.0>"))?
            .parse()
            .map_err(|_| anyhow!("ratio must be a number"))?;
        if !(0.0..=1.0).contains(&ratio) {
            return Err(anyhow!("ratio must be between 0.0 and 1.0"));
        }
        let focused = self
            .focused_panel()
            .ok_or_else(|| anyhow!("no focused window to rebalance"))?;
        let tree = self
            .engine
            .set_ratio(&focused, ratio, &self.file.doc.root)
            .map_err(|e| anyhow!("{e}"))?;
        self.replace_root(tree);
        self.normalize_camera();
        self.apply()?;
        self.save()?;
        // Report the resulting weights so the change is observable without a UI.
        let weights = self
            .file
            .doc
            .root
            .parent(&focused)
            .map(|s| s.weights.clone())
            .unwrap_or_default();
        let shown: Vec<String> = weights.iter().map(|w| format!("{w:.2}")).collect();
        Ok(format!(
            "ratio {ratio:.2} · split weights [{}]",
            shown.join(", ")
        ))
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
        // Re-grab so a `bindings` edit takes effect without a restart. The
        // notes come back in the reply so the caller sees a conflict where it
        // ran the command, not only in the journal.
        let notes = self.reinstall_bindings()?;
        self.apply()?;
        let mut message = "config reloaded".to_string();
        for note in notes {
            message.push_str("\n");
            message.push_str(&note);
        }
        Ok(message)
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
        self.detach_seat(&seat)?;
        let panel_id = self.own_panel_id(&seat);
        let node = LayoutNode::panel(panel_id, vec![seat.seat.clone()]);
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

        match self.reinstall_bindings() {
            Ok(notes) => {
                for note in notes {
                    eprintln!("pergola: {note}");
                }
            }
            Err(e) => eprintln!("pergola: cannot grab hotkeys: {e}"),
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
                    eprintln!(
                        "pergola: KeyPress detail={} state=0x{state:x} time={} last_release={:?}",
                        key.detail, key.time, self.last_release
                    );
                }
                // Mask to the modifiers a binding can name: Caps Lock and
                // Num Lock are keyboard state, not chords. Matching the raw
                // state is the classic "shortcuts die with numlock on" bug.
                let mods = event_mods(state);
                // A held key auto-repeats: the server sends the same keycode
                // again and again, so a bound chord would fire ~11 times a
                // second (measured on xfwm4+Xvfb: 11 presses in a 1 s hold).
                // Every command here is a discrete action — "back" held for a
                // second would race through eleven history entries — so the
                // first press acts and its repeats are swallowed.
                //
                // A repeat is exactly "a KeyPress whose timestamp equals the
                // preceding KeyRelease's" — the server sends the pair with one
                // timestamp. A deliberate second press always follows a release
                // with a later timestamp, so it fires. The latch is consumed so
                // a stale pair cannot swallow a later press.
                if is_auto_repeat(self.last_release, key.detail, key.time) {
                    self.last_release = None;
                    return Ok(());
                }
                let keysym = self
                    .keymap
                    .iter()
                    .find(|(code, _)| *code == key.detail)
                    .map(|(_, sym)| *sym);
                if let Some(keysym) = keysym {
                    // Owned, because `handle` needs `&mut self` while the
                    // binding table is borrowed.
                    let command = command_for_key(&self.bindings, mods, keysym).map(str::to_string);
                    if let Some(command) = command {
                        let argv: Vec<String> =
                            command.split_whitespace().map(str::to_string).collect();
                        if let Err(e) = self.handle(&argv) {
                            eprintln!("pergola: {command}: {e:#}");
                        }
                    }
                }
            }
            Event::KeyRelease(release) => {
                // Remember when and which key came up. The next KeyPress with
                // the same keycode *and* the same timestamp is the server
                // repeating, not a new press; a real press carries a later
                // timestamp. Modifier state is deliberately not compared: the
                // server sends the release of a grabbed key after the modifiers
                // have gone up when the user lets go of them first (measured
                // under xfwm4: `Ctrl+Alt+Shift+Left` arrives as shift-up
                // state=0xd, ctrl-up 0xc, alt-up 0x8, then Left-up state=0x0).
                if std::env::var_os("PERGOLA_DEBUG").is_some() {
                    eprintln!(
                        "pergola: KeyRelease detail={} state=0x{:x} time={}",
                        release.detail,
                        u16::from(release.state),
                        release.time
                    );
                }
                self.last_release = Some((release.detail, release.time));
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
            // A keyboard-map change (layout switch, xmodmap, another client's
            // remap) invalidates both the keycode table and the grabs. Rebuild
            // and re-grab, or every binding silently points at the old layout.
            Event::MappingNotify(_) => {
                for note in self.reinstall_bindings()? {
                    eprintln!("pergola: {note}");
                }
            }
            // A title or class change is a property change, not a configure:
            // without this the tree keeps the name the window had when it was
            // adopted until the next 2 s reconcile — herbstluftwm#35, bspwm#50.
            Event::PropertyNotify(property) => {
                if self.x.is_title_atom(property.atom) {
                    if let Ok(Some(info)) = self.x.info(property.window) {
                        if let Some(seat) = self.seats.iter_mut().find(|s| s.window == property.window) {
                            seat.title = info.title;
                            seat.class = info.class;
                        }
                    }
                }
            }
            // A window we own is gone. Remove its seat now instead of waiting
            // up to 2 s for the timer reconcile: a WM may hand the same XID to
            // a new window inside that window (measured on openbox: a killed
            // xterm's id came back ~0.4 s later), and the lingering seat would
            // make the daemon mistake the new window for the dead one —
            // updating its title and never adopting it.
            Event::DestroyNotify(destroy) => {
                if self.seats.iter().any(|s| s.window == destroy.window) {
                    if self.reconcile()? {
                        if self.drag.is_some() {
                            let _ = self.end_gap_drag();
                        }
                        self.normalize_camera();
                        let _ = self.apply();
                        let _ = self.save();
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Rebuild the keycode table and re-grab every binding after a keyboard
    /// map change. Ungrabbing first drops chords that still point at the old
    /// map; regrabbing the same client is idempotent.
    /// Re-grab every binding and return the notes worth telling the user:
    /// commands with no free key, and commands that fell back to a secondary
    /// chord because the desktop owns the primary one. `reload` returns these
    /// to the caller so a conflict is visible where the command ran, not only
    /// in the journal.
    fn reinstall_bindings(&mut self) -> Result<Vec<String>> {
        if !self.config.hotkeys {
            // Hotkeys were switched off by a reload: drop the grabs so the
            // desktop gets its chords back instead of the daemon holding them.
            let _ = self.x.ungrab_all_keys();
            self.bindings.clear();
            return Ok(vec!["hotkeys are off: no chords are grabbed".to_string()]);
        }
        self.keymap = build_keymap(&self.x)?;
        let _ = self.x.ungrab_all_keys();
        let plan = self.config.binding_plan();
        let mut notes = Vec::new();
        match self.x.install_bindings(&plan) {
            Ok((bound, failures)) => {
                self.bindings = bound;
                for failure in &failures {
                    notes.push(format!(
                        "no free key for '{}' (tried {})",
                        failure.command,
                        failure.tried.join(", ")
                    ));
                }
                for (binding, command) in &self.bindings {
                    if let Some((_, specs)) = plan.iter().find(|(c, _)| c == command) {
                        if let Some(primary) = specs.first() {
                            if binding.label != *primary {
                                notes.push(format!(
                                    "'{}' bound to {} ({} is owned by the desktop)",
                                    command, binding.label, primary
                                ));
                            }
                        }
                    }
                }
            }
            Err(e) => notes.push(format!("cannot grab hotkeys: {e}")),
        }
        Ok(notes)
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
        // Our own apply echoes back as configure events; a window after it is
        // not a user drag. (Removing this and relying only on the applied rect
        // was measured 2026-11-15: intermediate echoes of a two-step apply read
        // as drags and every window docked itself. The openbox cost of keeping
        // it is filed as pi-agenda bug 'edge drag under openbox'.)
        //
        // 100 ms, not 250 ms: measured 2026-11-15, the stale ConfigureNotify a
        // WM delivers after our move arrives within ~0.5 ms of the apply (a
        // pre-move echo the WM had already queued). 100 ms still catches that
        // burst while shrinking the window in which a real drag can be
        // swallowed 2.5x; both WMs passed 43/43 three runs each at 100 ms.
        if self.last_apply.elapsed() < Duration::from_millis(100) {
            return Ok(());
        }
        if self.seat_for_window(event.window).is_none() {
            return Ok(());
        }
        // A fullscreen window's geometry is the WM's (F11), not a drag the
        // user wants docked; and a transient window's place belongs to its
        // parent. Treating either as a drag docks the wrong window to the
        // nearest edge — awesome#22, i3#594.
        if let Ok(Some(info)) = self.x.info(event.window) {
            if info.fullscreen || info.transient {
                return Ok(());
            }
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
        if std::env::var_os("PERGOLA_DEBUG").is_some() {
            eprintln!(
                "pergola: configure 0x{:x} rect={:?} applied={:?} seat={}",
                event.window,
                rect,
                self.applied.get(&event.window).map(|r| r.to_rect()),
                self.seat_for_window(event.window).is_some(),
            );
        }
        if let Some(applied) = self.applied.get(&event.window) {
            if same_rect(applied.to_rect(), rect) {
                return Ok(());
            }
        }
        // The baseline is where we last put the window ourselves. A WM that
        // reports the drop position only (openbox sends one ICCCM-synthetic
        // ConfigureNotify at the end of a drag, xfwm4 sends a stream) still
        // shows a real travel from that baseline; a window the WM merely
        // clamped, or our own echo, does not move. Measured 2026-11-15: with a
        // strict "three configure events" rule, edge docking never fired under
        // openbox and the e2e suite's one failing assertion was this path.
        let baseline = self.applied.get(&event.window).map(|r| r.to_rect());
        let moved_from_baseline = baseline.is_some_and(|b| !same_rect(b, rect));
        let watch = self.moves.entry(event.window).or_insert(MoveWatch {
            moves: u32::from(moved_from_baseline),
            at: Instant::now(),
            rect,
            start: baseline.unwrap_or(rect),
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
            if std::env::var_os("PERGOLA_DEBUG").is_some() {
                eprintln!(
                    "pergola: move-watch 0x{:x} moves={} rect={:?} start={:?} age={:?} drag={}",
                    window,
                    watch.moves,
                    watch.rect,
                    watch.start,
                    now.duration_since(watch.at),
                    is_drag(watch),
                );
            }
            if now.duration_since(watch.at) >= Duration::from_millis(300) && is_drag(watch) {
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

/// A settled window counts as dragged when it produced a burst of configure
/// events, or when one event moved it far enough that no WM placement could
/// explain it (a single-report WM: openbox).
fn is_drag(watch: &MoveWatch) -> bool {
    if watch.moves >= 3 {
        return true;
    }
    if watch.moves == 0 {
        return false;
    }
    let travel = (watch.rect.x - watch.start.x)
        .abs()
        .max((watch.rect.y - watch.start.y).abs());
    travel >= DRAG_TRAVEL_PX
}

/// How far a single-event move must travel to be a drag rather than placement.
const DRAG_TRAVEL_PX: f64 = 40.0;

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

/// Is this KeyPress the server repeating a held key?
///
/// The server auto-repeats a held key: on this host 11 presses arrive in a
/// one-second hold. Every command here is a discrete action, so the first press
/// acts and its repeats are dropped. The server marks a repeat exactly: it
/// sends a `KeyRelease` and a `KeyPress` carrying the *same* timestamp. So a
/// press is a repeat when the immediately preceding release has the same
/// keycode and the same timestamp. A deliberate press always follows a release
/// with a later timestamp and therefore fires; so does a different key.
pub fn is_auto_repeat(last_release: Option<(u8, u32)>, code: u8, time: u32) -> bool {
    last_release == Some((code, time))
}

fn build_keymap(x: &X11) -> Result<Vec<(u8, u32)>> {
    use x11rb::protocol::xproto::ConnectionExt as _;
    let min = x.conn.setup().min_keycode;
    let max = x.conn.setup().max_keycode;
    let count = max - min + 1;
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
    fn a_held_key_is_one_press_not_eleven() {
        // The server repeats a held key ~11 times a second, each repeat as a
        // KeyRelease+KeyPress pair sharing one timestamp. A press whose
        // timestamp equals the preceding release's is that repeat.
        assert!(is_auto_repeat(Some((113, 92756959)), 113, 92756959));
        // The real press arrives with no preceding release, so it fires.
        assert!(!is_auto_repeat(None, 113, 92756919));
        // A deliberate second press follows a release with a later timestamp.
        assert!(!is_auto_repeat(Some((113, 92756959)), 113, 92757020));
        // A different key is never a repeat, even at the same timestamp.
        assert!(!is_auto_repeat(Some((113, 92756959)), 114, 92756959));
    }

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
    fn a_single_report_that_travelled_is_a_drag() {
        // openbox reports the drop position once: one event, but 900 px of travel.
        let watch = MoveWatch {
            moves: 1,
            at: Instant::now(),
            rect: Rect::new(-156.0, 322.0, 316.0, 238.0),
            start: Rect::new(901.0, 322.0, 316.0, 238.0),
        };
        assert!(is_drag(&watch));
    }

    #[test]
    fn a_placement_at_the_edge_is_not_a_drag() {
        let placed = Rect::new(0.0, 0.0, 400.0, 300.0);
        // The WM clamped a new window to the corner and we never applied a rect:
        // no baseline, so no travel and no moves.
        let clamped = MoveWatch {
            moves: 0,
            at: Instant::now(),
            rect: placed,
            start: placed,
        };
        assert!(!is_drag(&clamped));
        // One event but a few px of placement jitter: still not a drag.
        let jitter = MoveWatch {
            moves: 1,
            at: Instant::now(),
            rect: placed,
            start: Rect::new(6.0, 3.0, 400.0, 300.0),
        };
        assert!(!is_drag(&jitter));
        // The event burst the xfwm4 path produces is still a drag.
        let burst = MoveWatch {
            moves: 3,
            at: Instant::now(),
            rect: placed,
            start: placed,
        };
        assert!(is_drag(&burst));
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
