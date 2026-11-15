//! X11/EWMH backend.
//!
//! Pergola is not an X window manager: it never takes `SubstructureRedirect` on
//! the root window, so it runs *inside* whatever desktop is already there
//! (xfwm4, mutter, kwin_x11, i3, openbox, …). It reads the EWMH client list and
//! asks the running WM to move windows with `_NET_MOVERESIZE_WINDOW`, exactly
//! the way `wmctrl` does.

use anyhow::{anyhow, Context, Result};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConfigureWindowAux, ConnectionExt as _, CreateWindowAux,
    EventMask, GrabMode, ModMask, PropMode, Window, WindowClass,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use pergola_core::{IRect, Rect};

/// A managed top-level window, as Pergola sees it.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    pub id: u32,
    /// Client-area rectangle in root coordinates.
    pub rect: IRect,
    /// Frame extents (left, right, top, bottom) reported by the WM.
    pub extents: (i32, i32, i32, i32),
    pub title: String,
    pub class: String,
    pub pid: Option<u32>,
    pub hidden: bool,
    pub active: bool,
}

impl WindowInfo {
    /// The rectangle including WM decorations, which is what the user sees.
    pub fn frame_rect(&self) -> IRect {
        let (l, r, t, b) = self.extents;
        IRect::new(
            self.rect.x - l,
            self.rect.y - t,
            self.rect.w + l + r,
            self.rect.h + t + b,
        )
    }
}

/// A key binding: modifiers plus an X keysym name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    pub mods: u16,
    pub keysym: u32,
    pub label: String,
}

pub struct X11 {
    pub conn: RustConnection,
    pub screen: usize,
    pub root: Window,
    atoms: Atoms,
    overlay: Vec<Window>,
}

struct Atoms {
    net_client_list: u32,
    net_active_window: u32,
    net_wm_name: u32,
    net_wm_pid: u32,
    net_wm_window_type: u32,
    net_wm_state: u32,
    net_wm_state_hidden: u32,
    net_frame_extents: u32,
    net_workarea: u32,
    net_current_desktop: u32,
    net_moveresize_window: u32,
    utf8_string: u32,
    wm_class: u32,
    type_skip: Vec<u32>,
}

impl X11 {
    pub fn connect() -> Result<Self> {
        let (conn, screen) = x11rb::connect(None).context(
            "cannot open the X display (is DISPLAY set? Pergola needs an X11 session)",
        )?;
        let root = conn.setup().roots[screen].root;
        let atoms = Atoms::new(&conn)?;
        Ok(Self {
            conn,
            screen,
            root,
            atoms,
            overlay: Vec::new(),
        })
    }

    /// The whole display: always starts at 0,0 and ignores panels.
    pub fn screen_rect(&self) -> Result<IRect> {
        let g = self.conn.get_geometry(self.root)?.reply()?;
        Ok(IRect::new(0, 0, g.width as i32, g.height as i32))
    }

    /// The area the window manager leaves for managed windows: `_NET_WORKAREA`
    /// for the current desktop, so panels and docks are not tiled over. Falls
    /// back to [`Self::screen_rect`] when the property is missing or degenerate.
    pub fn workarea_rect(&self) -> Result<IRect> {
        let screen = self.screen_rect()?;
        let desktop = self
            .cardinal_prop(self.root, self.atoms.net_current_desktop)
            .and_then(|v| v.first().copied())
            .unwrap_or(0) as usize;
        Ok(pick_workarea(
            self.cardinal_prop(self.root, self.atoms.net_workarea),
            desktop,
            screen,
        ))
    }

    /// EWMH `_NET_CLIENT_LIST`, in stacking order.
    pub fn client_list(&self) -> Result<Vec<u32>> {
        let reply = self
            .conn
            .get_property(
                false,
                self.root,
                self.atoms.net_client_list,
                AtomEnum::WINDOW,
                0,
                4096,
            )?
            .reply()?;
        Ok(reply.value32().map(|it| it.collect()).unwrap_or_default())
    }

    pub fn active_window(&self) -> Option<u32> {
        let reply = self
            .conn
            .get_property(
                false,
                self.root,
                self.atoms.net_active_window,
                AtomEnum::WINDOW,
                0,
                1,
            )
            .ok()?
            .reply()
            .ok()?;
        let value = reply.value32().and_then(|mut it| it.next());
        value
    }

    /// Describe one window, or `None` when it is not a normal managed window.
    pub fn info(&self, win: u32) -> Result<Option<WindowInfo>> {
        let attrs = self.conn.get_window_attributes(win)?.reply()?;
        if attrs.override_redirect || attrs.map_state != x11rb::protocol::xproto::MapState::VIEWABLE
        {
            return Ok(None);
        }
        let types = self.atom_list(win, self.atoms.net_wm_window_type)?;
        if types.iter().any(|t| self.atoms.type_skip.contains(t)) {
            return Ok(None);
        }
        let states = self.atom_list(win, self.atoms.net_wm_state)?;
        let hidden = states.contains(&self.atoms.net_wm_state_hidden);

        let geom = self.conn.get_geometry(win)?.reply()?;
        let translated = self
            .conn
            .translate_coordinates(win, self.root, 0, 0)?
            .reply()?;
        let rect = IRect::new(
            translated.dst_x as i32,
            translated.dst_y as i32,
            geom.width as i32,
            geom.height as i32,
        );

        let extents = self.frame_extents(win);

        let title = self
            .string_prop(win, self.atoms.net_wm_name)
            .or_else(|| self.string_prop(win, AtomEnum::WM_NAME.into()))
            .unwrap_or_default();
        let class = self
            .string_prop(win, self.atoms.wm_class)
            .unwrap_or_default()
            .split('\0')
            .next_back()
            .unwrap_or("")
            .to_string();
        let pid = self
            .conn
            .get_property(false, win, self.atoms.net_wm_pid, AtomEnum::CARDINAL, 0, 1)
            .ok()
            .and_then(|c| c.reply().ok())
            .and_then(|r| r.value32().and_then(|mut it| it.next()));

        Ok(Some(WindowInfo {
            id: win,
            rect,
            extents,
            title,
            class,
            pid,
            hidden,
            active: self.active_window() == Some(win),
        }))
    }

    /// Every normal managed window, in stacking order.
    pub fn windows(&self) -> Result<Vec<WindowInfo>> {
        let mut out = Vec::new();
        for win in self.client_list()? {
            if let Ok(Some(info)) = self.info(win) {
                out.push(info);
            }
        }
        Ok(out)
    }

    /// Ask the running WM to place `win` so that its *frame* occupies `frame`.
    ///
    /// `_NET_MOVERESIZE_WINDOW` is the EWMH way for an external tool to move a
    /// window: the WM keeps its own gravity and decoration bookkeeping. Gravity
    /// is NorthWest (0), so x/y are the frame origin, and the width/height are
    /// client dimensions — hence the frame extents are subtracted here.
    pub fn move_resize(&self, win: u32, frame: IRect) -> Result<()> {
        let (l, r, t, b) = self.frame_extents(win);
        let gravity = 0u32; // NorthWestGravity
        let flags = 0xFu32; // x, y, w, h all present
        let source = 2u32; // pager
        let data = [
            gravity | (flags << 8) | (source << 12),
            frame.x as u32,
            frame.y as u32,
            (frame.w - l - r).max(1) as u32,
            (frame.h - t - b).max(1) as u32,
        ];
        if std::env::var_os("PERGOLA_DEBUG").is_some() {
            eprintln!(
                "pergola: move 0x{win:x} frame={:?} extents={:?} data={:?}",
                frame, (l, r, t, b), data
            );
        }
        let event = ClientMessageEvent::new(32, win, self.atoms.net_moveresize_window, data);
        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        self.conn.flush()?;
        Ok(())
    }

    /// Direct configure of the client area, used as a fallback.
    pub fn configure(&self, win: u32, rect: IRect) -> Result<()> {
        self.conn.configure_window(
            win,
            &ConfigureWindowAux::new()
                .x(rect.x)
                .y(rect.y)
                .width(rect.w.max(1) as u32)
                .height(rect.h.max(1) as u32),
        )?;
        self.conn.flush()?;
        Ok(())
    }

    /// `_NET_FRAME_EXTENTS` (left, right, top, bottom), or zeros.
    pub fn frame_extents(&self, win: u32) -> (i32, i32, i32, i32) {
        self.conn
            .get_property(
                false,
                win,
                self.atoms.net_frame_extents,
                AtomEnum::CARDINAL,
                0,
                4,
            )
            .ok()
            .and_then(|c| c.reply().ok())
            .and_then(|r| {
                let v: Vec<u32> = r.value32()?.collect();
                (v.len() == 4).then(|| (v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32))
            })
            .unwrap_or((0, 0, 0, 0))
    }

    pub fn activate(&self, win: u32) -> Result<()> {
        let data = [2u32, 0, 0, 0, 0]; // source = pager, CurrentTime
        let event = ClientMessageEvent::new(32, win, self.atoms.net_active_window, data);
        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        self.conn.flush()?;
        Ok(())
    }

    /// Raise a window without focusing it.
    pub fn raise(&self, win: u32) -> Result<()> {
        self.conn.configure_window(
            win,
            &ConfigureWindowAux::new().stack_mode(x11rb::protocol::xproto::StackMode::ABOVE),
        )?;
        self.conn.flush()?;
        Ok(())
    }

    // MARK: overlay

    /// Draw filled bars (each with its own colour). The daemon uses this for
    /// split lines that live in the gaps and for the viewport frame, so the
    /// overlay never covers a window's own decorations.
    pub fn overlay_show(&mut self, bars: &[(IRect, u32)]) -> Result<()> {
        self.overlay_hide()?;
        let screen = &self.conn.setup().roots[self.screen];
        let root_visual = screen.root_visual;
        let depth = screen.root_depth;
        let mut ids = Vec::new();
        for (bar, color) in bars {
            if bar.w <= 0 || bar.h <= 0 {
                continue;
            }
            let wid = self.conn.generate_id()?;
            self.conn.create_window(
                depth,
                wid,
                self.root,
                bar.x as i16,
                bar.y as i16,
                bar.w as u16,
                bar.h as u16,
                0,
                WindowClass::INPUT_OUTPUT,
                root_visual,
                &CreateWindowAux::new()
                    .background_pixel(*color)
                    .override_redirect(1)
                    .event_mask(EventMask::EXPOSURE),
            )?;
            self.conn.map_window(wid)?;
            // Override-redirect windows are unmanaged, so the WM will not
            // restack them; raise them explicitly to sit above the tiles.
            self.conn.configure_window(
                wid,
                &ConfigureWindowAux::new()
                    .stack_mode(x11rb::protocol::xproto::StackMode::ABOVE),
            )?;
            ids.push(wid);
        }
        self.conn.flush()?;
        self.overlay = ids;
        Ok(())
    }

    pub fn overlay_hide(&mut self) -> Result<()> {
        for wid in self.overlay.drain(..) {
            let _ = self.conn.destroy_window(wid);
        }
        self.conn.flush()?;
        Ok(())
    }

    pub fn overlay_active(&self) -> bool {
        !self.overlay.is_empty()
    }

    // MARK: hotkeys

    /// Grab `bindings` on the root window. Returns the labels that could not be
    /// grabbed (usually because the running WM already owns them).
    pub fn grab_keys(&self, bindings: &[KeyBinding]) -> Result<Vec<String>> {
        let min = self.conn.setup().min_keycode;
        let max = self.conn.setup().max_keycode;
        let count = (max - min + 1) as u8;
        let mapping = self.conn.get_keyboard_mapping(min, count)?.reply()?;
        let per = mapping.keysyms_per_keycode as usize;
        let mut failed = Vec::new();
        for binding in bindings {
            let mut keycode = None;
            for (i, chunk) in mapping.keysyms.chunks(per).enumerate() {
                if chunk.contains(&binding.keysym) {
                    keycode = Some(min + i as u8);
                    break;
                }
            }
            let Some(keycode) = keycode else {
                failed.push(format!("{} (no keycode)", binding.label));
                continue;
            };
            // BadAccess arrives asynchronously, so check the reply rather than
            // just the send.
            let ok = self
                .conn
                .grab_key(
                    false,
                    self.root,
                    ModMask::from(binding.mods),
                    keycode,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                )
                .map(|cookie| cookie.check().is_ok())
                .unwrap_or(false);
            if !ok {
                failed.push(binding.label.clone());
            }
        }
        self.conn.flush()?;
        Ok(failed)
    }

    /// Block for the next event, with a timeout in milliseconds.
    pub fn next_event(&self, timeout_ms: i32) -> Result<Option<Event>> {
        if let Some(event) = self.conn.poll_for_event()? {
            return Ok(Some(event));
        }
        // x11rb has no portable poll timeout; sleep briefly and retry. The
        // daemon's socket thread wakes it through a channel, so this only
        // bounds hotkey latency.
        std::thread::sleep(std::time::Duration::from_millis(timeout_ms.max(1) as u64));
        Ok(self.conn.poll_for_event()?)
    }

    // MARK: helpers

    fn cardinal_prop(&self, win: u32, atom: u32) -> Option<Vec<u32>> {
        let reply = self
            .conn
            .get_property(false, win, atom, AtomEnum::CARDINAL, 0, 64)
            .ok()?
            .reply()
            .ok()?;
        Some(reply.value32().map(|it| it.collect()).unwrap_or_default())
    }

    fn atom_list(&self, win: u32, atom: u32) -> Result<Vec<u32>> {
        let reply = self
            .conn
            .get_property(false, win, atom, AtomEnum::ATOM, 0, 64)?
            .reply()?;
        Ok(reply.value32().map(|it| it.collect()).unwrap_or_default())
    }

    fn string_prop(&self, win: u32, atom: u32) -> Option<String> {
        let reply = self
            .conn
            .get_property(false, win, atom, AtomEnum::ANY, 0, 1024)
            .ok()?
            .reply()
            .ok()?;
        if reply.value.is_empty() {
            return None;
        }
        Some(String::from_utf8_lossy(&reply.value).trim_end_matches('\0').to_string())
    }

    /// Set a UTF-8 string property (used for the daemon's status window title).
    pub fn set_utf8(&self, win: u32, atom: u32, value: &str) -> Result<()> {
        self.conn.change_property(
            PropMode::REPLACE,
            win,
            atom,
            self.atoms.utf8_string,
            8,
            value.len() as u32,
            value.as_bytes(),
        )?;
        self.conn.flush()?;
        Ok(())
    }

    pub fn atom(&self, name: &str) -> Result<u32> {
        Ok(self.conn.intern_atom(false, name.as_bytes())?.reply()?.atom)
    }
}

impl Atoms {
    fn new(conn: &RustConnection) -> Result<Self> {
        let intern = |name: &str| -> Result<u32> {
            Ok(conn.intern_atom(false, name.as_bytes())?.reply()?.atom)
        };
        let mut type_skip = Vec::new();
        for name in [
            "_NET_WM_WINDOW_TYPE_DOCK",
            "_NET_WM_WINDOW_TYPE_DESKTOP",
            "_NET_WM_WINDOW_TYPE_TOOLBAR",
            "_NET_WM_WINDOW_TYPE_MENU",
            "_NET_WM_WINDOW_TYPE_SPLASH",
            "_NET_WM_WINDOW_TYPE_NOTIFICATION",
            "_NET_WM_WINDOW_TYPE_DROPDOWN_MENU",
            "_NET_WM_WINDOW_TYPE_POPUP_MENU",
            "_NET_WM_WINDOW_TYPE_TOOLTIP",
            "_NET_WM_WINDOW_TYPE_COMBO",
            "_NET_WM_WINDOW_TYPE_DND",
        ] {
            type_skip.push(intern(name)?);
        }
        Ok(Self {
            net_client_list: intern("_NET_CLIENT_LIST")?,
            net_active_window: intern("_NET_ACTIVE_WINDOW")?,
            net_wm_name: intern("_NET_WM_NAME")?,
            net_wm_pid: intern("_NET_WM_PID")?,
            net_wm_window_type: intern("_NET_WM_WINDOW_TYPE")?,
            net_wm_state: intern("_NET_WM_STATE")?,
            net_wm_state_hidden: intern("_NET_WM_STATE_HIDDEN")?,
            net_frame_extents: intern("_NET_FRAME_EXTENTS")?,
            net_workarea: intern("_NET_WORKAREA")?,
            net_current_desktop: intern("_NET_CURRENT_DESKTOP")?,
            net_moveresize_window: intern("_NET_MOVERESIZE_WINDOW")?,
            utf8_string: intern("UTF8_STRING")?,
            wm_class: AtomEnum::WM_CLASS.into(),
            type_skip,
        })
    }
}

/// Pick the work-area quad for `desktop` out of an `_NET_WORKAREA` value, or the
/// whole display when the property is missing, short, or degenerate.
fn pick_workarea(values: Option<Vec<u32>>, desktop: usize, screen: IRect) -> IRect {
    let Some(values) = values else { return screen };
    let Some(quad) = values
        .chunks(4)
        .nth(desktop)
        .or_else(|| values.chunks(4).next())
    else {
        return screen;
    };
    if quad.len() < 4 {
        return screen;
    }
    let rect = IRect::new(quad[0] as i32, quad[1] as i32, quad[2] as i32, quad[3] as i32);
    if rect.w <= 0 || rect.h <= 0 {
        return screen;
    }
    rect
}

/// Modifier masks used by the default bindings.
pub mod mods {
    // X modifier bits: Shift = 1, Control = 4, Mod1 = 8.
    pub const CTRL_ALT: u16 = 4 | 8;
    pub const CTRL_ALT_SHIFT: u16 = 4 | 8 | 1;
}

/// X keysyms for the default bindings (values from `<X11/keysymdef.h>`).
pub mod keysym {
    pub const SPACE: u32 = 0x0020;
    pub const RETURN: u32 = 0xff0d;
    pub const UP: u32 = 0xff52;
    pub const DOWN: u32 = 0xff54;
    pub const LEFT: u32 = 0xff51;
    pub const RIGHT: u32 = 0xff53;
    pub const BRACKET_LEFT: u32 = 0x005b;
    pub const BRACKET_RIGHT: u32 = 0x005d;
    pub const D: u32 = 0x0064;
    pub const N: u32 = 0x006e;
    pub const A: u32 = 0x0061;
    pub const F: u32 = 0x0066;
    pub const O: u32 = 0x006f;
    pub const DIGIT_1: u32 = 0x0031;
    pub const DIGIT_9: u32 = 0x0039;
}

/// The key map: each command has a primary binding (matching the TrellisWM
/// guide's `⌃⌥` shortcuts) and a `Super` fallback for desktops that already own
/// the `Ctrl+Alt` combination — xfwm4 does, for workspace switching.
pub fn binding_plan() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("everything", vec!["Ctrl+Alt+Space", "Super+Space"]),
        ("zoom-out", vec!["Ctrl+Alt+Up", "Super+Up"]),
        ("zoom-in", vec!["Ctrl+Alt+Down", "Super+Down"]),
        ("neighbour?dir=left", vec!["Ctrl+Alt+Left", "Super+Left"]),
        ("neighbour?dir=right", vec!["Ctrl+Alt+Right", "Super+Right"]),
        ("neighbour?dir=up", vec!["Ctrl+Alt+Shift+Up", "Super+Shift+Up"]),
        ("neighbour?dir=down", vec!["Ctrl+Alt+Shift+Down", "Super+Shift+Down"]),
        ("back", vec!["Ctrl+Alt+[", "Super+["]),
        ("forward", vec!["Ctrl+Alt+]", "Super+]"]),
        ("frame-window", vec!["Ctrl+Alt+Return", "Super+Return"]),
        ("toggle-desktop", vec!["Ctrl+Alt+d", "Super+d"]),
        ("split-terminal", vec!["Ctrl+Alt+n", "Super+n"]),
        ("arrange", vec!["Ctrl+Alt+a", "Super+a"]),
        ("float-all", vec!["Ctrl+Alt+f", "Super+f"]),
        ("overlay", vec!["Ctrl+Alt+o", "Super+o"]),
    ]
}

/// A plan entry that could not be bound on any candidate key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindFailure {
    pub command: String,
    pub tried: Vec<String>,
}

impl X11 {
    /// Bind the first free candidate for every command. Returns the bindings
    /// that were installed and the ones that had to be skipped.
    pub fn install_bindings(
        &self,
        plan: &[(&'static str, Vec<&'static str>)],
    ) -> Result<(Vec<(KeyBinding, &'static str)>, Vec<BindFailure>)> {
        let min = self.conn.setup().min_keycode;
        let max = self.conn.setup().max_keycode;
        let mapping = self
            .conn
            .get_keyboard_mapping(min, (max - min + 1) as u8)?
            .reply()?;
        let per = mapping.keysyms_per_keycode as usize;
        let keycode_for = |sym: u32| -> Option<u8> {
            mapping
                .keysyms
                .chunks(per)
                .position(|chunk| chunk.contains(&sym))
                .map(|i| min + i as u8)
        };
        let mut installed = Vec::new();
        let mut failures = Vec::new();
        for (command, specs) in plan {
            let mut bound = false;
            for spec in specs {
                let Some(binding) = parse_binding(spec) else {
                    continue;
                };
                let Some(keycode) = keycode_for(binding.keysym) else {
                    continue;
                };
                let ok = self
                    .conn
                    .grab_key(
                        false,
                        self.root,
                        ModMask::from(binding.mods),
                        keycode,
                        GrabMode::ASYNC,
                        GrabMode::ASYNC,
                    )
                    .map(|cookie| cookie.check().is_ok())
                    .unwrap_or(false);
                if ok {
                    installed.push((binding, *command));
                    bound = true;
                    break;
                }
            }
            if !bound {
                failures.push(BindFailure {
                    command: command.to_string(),
                    tried: specs.iter().map(|s| s.to_string()).collect(),
                });
            }
        }
        self.conn.flush()?;
        Ok((installed, failures))
    }
}

/// Look up a keysym by its X name (lowercase letters, digits, and the common
/// named keys are enough for binding specs).
pub fn keysym_from_name(name: &str) -> Option<u32> {
    let lower = name.to_ascii_lowercase();
    let named = match lower.as_str() {
        "space" => Some(keysym::SPACE),
        "return" | "enter" => Some(keysym::RETURN),
        "left" => Some(keysym::LEFT),
        "right" => Some(keysym::RIGHT),
        "up" => Some(keysym::UP),
        "down" => Some(keysym::DOWN),
        "[" | "bracketleft" => Some(keysym::BRACKET_LEFT),
        "]" | "bracketright" => Some(keysym::BRACKET_RIGHT),
        "tab" => Some(0xff09),
        "escape" | "esc" => Some(0xff1b),
        "home" => Some(0xff50),
        "end" => Some(0xff57),
        "page_up" | "prior" | "pageup" => Some(0xff55),
        "page_down" | "next" | "pagedown" => Some(0xff56),
        "comma" => Some(0x002c),
        "period" => Some(0x002e),
        "slash" => Some(0x002f),
        "semicolon" => Some(0x003b),
        "apostrophe" => Some(0x0027),
        "minus" => Some(0x002d),
        "equal" => Some(0x003d),
        "backslash" => Some(0x005c),
        "grave" => Some(0x0060),
        _ => None,
    };
    if named.is_some() {
        return named;
    }
    let mut chars = lower.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            return Some(c as u32);
        }
    }
    None
}

/// Parse `Ctrl+Alt+Space` or `Super+Left` into a binding.
pub fn parse_binding(spec: &str) -> Option<KeyBinding> {
    let mut mods = 0u16;
    let mut key = None;
    for part in spec.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => mods |= 4,
            "alt" | "mod1" | "meta" => mods |= 8,
            "shift" => mods |= 1,
            "super" | "mod4" | "win" => mods |= 64,
            _ => {
                if key.is_some() {
                    return None;
                }
                key = keysym_from_name(part);
            }
        }
    }
    Some(KeyBinding {
        mods,
        keysym: key?,
        label: spec.to_string(),
    })
}

/// Map a key event back to a command string.
pub fn command_for_key(bindings: &[(KeyBinding, &'static str)], mods: u16, keysym: u32) -> Option<&'static str> {
    bindings
        .iter()
        .find(|(b, _)| b.mods == mods && b.keysym == keysym)
        .map(|(_, cmd)| *cmd)
}

/// Convert a core `Rect` to an `IRect`.
pub fn to_irect(r: Rect) -> IRect {
    r.to_irect()
}

/// Human-readable summary of a window, for `pergola list`.
pub fn describe(info: &WindowInfo) -> String {
    format!(
        "0x{:08x}  {:>4}x{:<4} +{:<5}+{:<5}  {:<20}  {}",
        info.id,
        info.rect.w,
        info.rect.h,
        info.rect.x,
        info.rect.y,
        truncate(&info.class, 20),
        truncate(&info.title, 60)
    )
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

/// Check that the display is reachable and EWMH is present.
pub fn doctor() -> Result<Vec<String>> {
    let x = X11::connect()?;
    let mut notes = Vec::new();
    notes.push(format!("display: {}", std::env::var("DISPLAY").unwrap_or_default()));
    let screen = x.screen_rect()?;
    notes.push(format!("screen: {}x{}", screen.w, screen.h));
    let clients = x.client_list()?;
    notes.push(format!("EWMH client list: {} windows", clients.len()));
    let wm = x
        .conn
        .get_property(
            false,
            x.root,
            x.atom("_NET_SUPPORTING_WM_CHECK")?,
            AtomEnum::WINDOW,
            0,
            1,
        )?
        .reply()?;
    let wm_name = wm
        .value32()
        .and_then(|mut it| it.next())
        .and_then(|w| x.string_prop(w, x.atom("_NET_WM_NAME").ok()?))
        .unwrap_or_else(|| "unknown".to_string());
    notes.push(format!("running WM: {wm_name}"));
    if clients.is_empty() {
        return Err(anyhow!(
            "no EWMH client list — this desktop does not advertise _NET_CLIENT_LIST"
        ));
    }
    Ok(notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: IRect = IRect { x: 0, y: 0, w: 1600, h: 900 };

    #[test]
    fn workarea_uses_the_current_desktop_quad() {
        // Two desktops: only the second has a top panel.
        let values = vec![0, 0, 1600, 900, 0, 27, 1600, 873];
        assert_eq!(
            pick_workarea(Some(values.clone()), 1, SCREEN),
            IRect::new(0, 27, 1600, 873)
        );
        assert_eq!(pick_workarea(Some(values), 0, SCREEN), SCREEN);
    }

    #[test]
    fn workarea_falls_back_to_the_display() {
        assert_eq!(pick_workarea(None, 0, SCREEN), SCREEN);
        assert_eq!(pick_workarea(Some(vec![]), 0, SCREEN), SCREEN);
        assert_eq!(pick_workarea(Some(vec![1, 2, 0, 5]), 0, SCREEN), SCREEN);
        // Out-of-range desktop falls back to the first quad, not a panic.
        assert_eq!(
            pick_workarea(Some(vec![0, 27, 1600, 873]), 4, SCREEN),
            IRect::new(0, 27, 1600, 873)
        );
    }
}
