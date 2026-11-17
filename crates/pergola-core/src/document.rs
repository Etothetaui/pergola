//! The persisted layout document.
//!
//! Shape follows the `version: 2` layout files described in `docs/RESEARCH.md`
//! (a `doc` tree plus camera and floating-window bookkeeping), with Pergola's
//! own additions under `pergola` so a foreign document still loads.

use serde::{Deserialize, Serialize};

use crate::camera::CameraState;
use crate::geom::{IRect, Rect, Size};
use crate::node::LayoutNode;

/// One floating window on the desktop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloatingWindow {
    pub panel: crate::node::Panel,
    pub rect: Rect,
    #[serde(default)]
    pub size: Size,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z: Option<i32>,
}

/// A named workspace: a set of views that are shown together. A view belongs to
/// at most one tag, so two tags never fight over a window; the active tag
/// decides which views the daemon lays out. Tags are additive: a document with
/// no tags loads exactly as before, and an older binary reads a tagged document
/// by ignoring the `tags` key — the views are all still in `root`, so nothing is
/// lost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(default)]
    pub views: Vec<String>,
    /// The seat that had the keyboard when this tag was last left, so returning
    /// to it puts the user back where they were instead of on an arbitrary
    /// window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
}

impl Tag {
    pub fn new(name: impl Into<String>, views: Vec<String>) -> Self {
        Self {
            name: name.into(),
            views,
            focus: None,
        }
    }

    pub fn holds(&self, seat: &str) -> bool {
        self.views.iter().any(|v| v == seat)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutDocument {
    pub root: LayoutNode,
    #[serde(default)]
    pub floating: Vec<FloatingWindow>,
    #[serde(default)]
    pub hidden: Vec<String>,
    /// Named workspaces. Empty means "no tags in use", which is the old
    /// single-tree behaviour.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    /// The tag currently shown. `None` means every view is shown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_tag: Option<String>,
}

impl Default for LayoutDocument {
    fn default() -> Self {
        Self {
            root: LayoutNode::desktop(),
            floating: Vec::new(),
            hidden: Vec::new(),
            tags: Vec::new(),
            active_tag: None,
        }
    }
}

impl LayoutDocument {
    pub fn tag(&self, name: &str) -> Option<&Tag> {
        self.tags.iter().find(|t| t.name == name)
    }

    pub fn tag_mut(&mut self, name: &str) -> Option<&mut Tag> {
        self.tags.iter_mut().find(|t| t.name == name)
    }

    /// Create the tag if it is not there. Names are compared exactly, so a
    /// typo makes a new tag rather than silently moving a window.
    pub fn ensure_tag(&mut self, name: &str) -> &mut Tag {
        if !self.tags.iter().any(|t| t.name == name) {
            self.tags.push(Tag::new(name, Vec::new()));
        }
        self.tag_mut(name).expect("tag was just ensured")
    }

    /// The name of the tag holding `seat`, if any.
    pub fn tag_of(&self, seat: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|t| t.holds(seat))
            .map(|t| t.name.as_str())
    }

    /// Move `seat` into `tag`, removing it from every other tag first so the
    /// sets stay disjoint.
    pub fn assign_view(&mut self, seat: &str, tag: &str) {
        for t in &mut self.tags {
            t.views.retain(|v| v != seat);
        }
        let t = self.ensure_tag(tag);
        if !t.holds(seat) {
            t.views.push(seat.to_string());
        }
    }

    /// Drop a seat from every tag (a window closed). Tags left empty are kept
    /// so the user's workspace names survive a closed window — deleting one is
    /// `pergola tag-remove`, never a side effect of a window going away.
    pub fn forget_view(&mut self, seat: &str) {
        for t in &mut self.tags {
            t.views.retain(|v| v != seat);
            if t.focus.as_deref() == Some(seat) {
                t.focus = None;
            }
        }
    }

    /// The views the active tag shows. `None` means "no tag is active, so every
    /// view shows" — the pre-tag behaviour.
    pub fn active_views(&self) -> Option<std::collections::HashSet<String>> {
        let name = self.active_tag.as_deref()?;
        let tag = self.tag(name)?;
        Some(tag.views.iter().cloned().collect())
    }

    /// Is this view on screen under the active tag?
    pub fn view_visible(&self, seat: &str) -> bool {
        match self.active_views() {
            None => true,
            Some(views) => views.contains(seat),
        }
    }

    /// The tree as the active tag sees it. Panels with no visible view are
    /// dropped, single-child splits collapse, and split ids are preserved so
    /// seats and the camera keep pointing at the same places. With no active
    /// tag this is the whole tree, unchanged.
    pub fn active_tree(&self) -> LayoutNode {
        let Some(views) = self.active_views() else {
            return self.root.clone();
        };
        prune_to(&self.root, &views).unwrap_or_else(LayoutNode::desktop)
    }
}

/// Keep only the panels that hold a view in `keep`, collapsing splits that lose
/// all but one child. Returns `None` when nothing survives, so a caller can
/// fall back to an empty desktop.
fn prune_to(node: &LayoutNode, keep: &std::collections::HashSet<String>) -> Option<LayoutNode> {
    match node {
        LayoutNode::Desktop(_) => Some(node.clone()),
        LayoutNode::Panel(p) => {
            let views: Vec<String> = p
                .views
                .iter()
                .filter(|v| keep.contains(*v))
                .cloned()
                .collect();
            if views.is_empty() {
                return None;
            }
            let mut q = p.clone();
            q.views = views;
            if q.selected.as_ref().map_or(true, |s| !q.views.contains(s)) {
                q.selected = q.views.first().cloned();
            }
            if q.views.len() < 2 {
                q.tabbed = None;
            }
            Some(LayoutNode::Panel(q))
        }
        LayoutNode::Split(s) => {
            let mut kids = Vec::new();
            let mut weights = Vec::new();
            for (i, child) in s.children.iter().enumerate() {
                if let Some(k) = prune_to(child, keep) {
                    kids.push(k);
                    weights.push(s.weights.get(i).copied().unwrap_or(0.0));
                }
            }
            match kids.len() {
                0 => None,
                1 => kids.pop(),
                _ => Some(LayoutNode::Split(crate::node::Split::with_weights(
                    s.id.clone(),
                    s.axis,
                    kids,
                    weights,
                ))),
            }
        }
    }
}

/// Pergola's own bookkeeping, kept out of the shared keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PergolaMeta {
    /// Window id -> seat id, so a restart can re-attach panels to windows.
    #[serde(default)]
    pub seats: Vec<SeatRecord>,
    #[serde(default)]
    pub gap: Option<f64>,
    #[serde(default)]
    pub auto_dock: Option<bool>,
    #[serde(default)]
    pub overlay: Option<bool>,
}

/// A window fingerprint. `window` is only valid for the session that wrote it;
/// `class` + `title` are the fallback used after a restart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeatRecord {
    pub seat: String,
    pub panel: String,
    pub window: u32,
    #[serde(default)]
    pub class: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutFile {
    pub doc: LayoutDocument,
    #[serde(default)]
    pub dock_spots: std::collections::BTreeMap<String, Rect>,
    #[serde(default)]
    pub float_sizes: std::collections::BTreeMap<String, Size>,
    #[serde(default = "default_framed")]
    pub framed: String,
    #[serde(default)]
    pub framings: std::collections::BTreeMap<String, Rect>,
    #[serde(default)]
    pub framed_leaves: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saved: Option<f64>,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera: Option<CameraState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pergola: Option<PergolaMeta>,
}

fn default_framed() -> String {
    String::new()
}

fn default_version() -> u32 {
    2
}

impl Default for LayoutFile {
    fn default() -> Self {
        Self {
            doc: LayoutDocument::default(),
            dock_spots: Default::default(),
            float_sizes: Default::default(),
            framed: default_framed(),
            framings: Default::default(),
            framed_leaves: Vec::new(),
            saved: None,
            version: default_version(),
            world: None,
            camera: None,
            pergola: None,
        }
    }
}

impl LayoutFile {
    pub fn from_json(raw: &str) -> Result<Self, serde_json::Error> {
        let mut file: LayoutFile = serde_json::from_str(raw)?;
        // Repair drift rather than refusing to load.
        file.doc.root = file.doc.root.normalized();
        if file.camera.is_none() {
            file.camera = Some(CameraState {
                framed: file.framed.clone(),
                ..CameraState::default()
            });
        }
        Ok(file)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("layout document is serializable")
    }

    pub fn camera(&self) -> CameraState {
        self.camera.clone().unwrap_or_else(|| CameraState {
            framed: self.framed.clone(),
            ..CameraState::default()
        })
    }

    pub fn set_camera(&mut self, camera: &CameraState) {
        self.framed = camera.framed.clone();
        self.camera = Some(camera.clone());
    }

    /// Rect of a node in the given screen frame, if it exists.
    pub fn rect_of(&self, engine: &crate::engine::LayoutEngine, frame: Rect, id: &str) -> Option<IRect> {
        engine.rect_of(&self.doc.root, frame, id).map(Rect::to_irect)
    }
}

/// A three-level fractal tree for demos and tests. Mirrors the *shape* of the
/// observed documents, not their data.
pub fn fractal_tree() -> LayoutNode {
    use crate::geom::Axis;
    use crate::node::{Panel, Split};
    let a = LayoutNode::panel("p-notes", vec!["s-notes".into()]);
    let b = LayoutNode::panel("p-browser", vec!["s-browser".into()]);
    let c = LayoutNode::Panel(Panel::tab_group(
        "p-terminal",
        vec!["s-terminal".into(), "s-logs".into()],
    ));
    let d = LayoutNode::panel("p-editor", vec!["s-editor".into()]);
    let inner = LayoutNode::Split(Split::with_weights(
        "split-inner",
        Axis::Y,
        vec![c, d],
        vec![0.5, 0.5],
    ));
    let row = LayoutNode::Split(Split::with_weights(
        "split-row",
        Axis::X,
        vec![a, b, inner],
        vec![0.3, 0.3, 0.4],
    ));
    LayoutNode::Split(Split::with_weights(
        "root",
        Axis::X,
        vec![row, LayoutNode::desktop()],
        vec![0.72, 0.28],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_is_valid_and_round_trips() {
        let tree = fractal_tree();
        tree.validate().unwrap();
        let file = LayoutFile {
            doc: LayoutDocument {
                root: tree,
                ..Default::default()
            },
            world: Some("simulated-display|desktop-1".into()),
            ..Default::default()
        };
        let json = file.to_json();
        let back = LayoutFile::from_json(&json).unwrap();
        assert_eq!(back.doc.root, file.doc.root);
        assert_eq!(back.version, 2);
        assert!(back.camera.is_some());
    }

    #[test]
    fn foreign_document_with_unknown_keys_loads() {
        let raw = r#"{
            "doc": {"root": {"kind":"desktop","id":"desktop"}, "floating": [], "hidden": []},
            "framed": "desktop",
            "version": 2,
            "seats": {"s-1": {"appName": "x"}},
            "owners": {},
            "mirrorDigest": "abc"
        }"#;
        let file = LayoutFile::from_json(raw).unwrap();
        assert_eq!(file.doc.root.id(), "desktop");
        assert_eq!(file.framed, "desktop");
    }

    /// root (X)
    /// ├── split-a (Y): pa[s-a], pb[s-b]
    /// ├── pc [s-c, s-d]           a tab group
    /// └── desktop
    fn workspace_tree() -> LayoutNode {
        use crate::geom::Axis;
        let pa = LayoutNode::panel("pa", vec!["s-a".into()]);
        let pb = LayoutNode::panel("pb", vec!["s-b".into()]);
        let pc = LayoutNode::Panel(crate::node::Panel::tab_group(
            "pc",
            vec!["s-c".into(), "s-d".into()],
        ));
        LayoutNode::split(
            "root",
            Axis::X,
            vec![
                LayoutNode::split("split-a", Axis::Y, vec![pa, pb]),
                pc,
                LayoutNode::desktop(),
            ],
        )
    }

    fn tagged() -> LayoutDocument {
        let mut doc = LayoutDocument {
            root: workspace_tree(),
            ..Default::default()
        };
        doc.assign_view("s-a", "work");
        doc.assign_view("s-c", "work");
        doc.active_tag = Some("work".into());
        doc
    }

    #[test]
    fn a_view_belongs_to_at_most_one_tag() {
        let mut doc = LayoutDocument::default();
        doc.assign_view("s-a", "work");
        doc.assign_view("s-b", "work");
        doc.assign_view("s-a", "play");
        assert_eq!(doc.tag("work").unwrap().views, vec!["s-b".to_string()]);
        assert_eq!(doc.tag("play").unwrap().views, vec!["s-a".to_string()]);
        assert_eq!(doc.tag_of("s-a"), Some("play"));
        assert_eq!(doc.tag_of("s-b"), Some("work"));
        assert_eq!(doc.tag_of("s-z"), None);
    }

    #[test]
    fn the_active_tree_holds_only_the_active_tags_views() {
        let doc = tagged();
        let shown = doc.active_tree();
        assert!(shown.node("pa").is_some(), "the active panel survives");
        assert!(
            shown.node("split-a").is_none(),
            "a split left with one child collapses into it"
        );
        assert!(shown.node("pb").is_none(), "a view on another tag is gone");
        let Some(LayoutNode::Panel(pc)) = shown.node("pc") else {
            panic!("pc should survive with its one active view");
        };
        assert_eq!(pc.views, vec!["s-c".to_string()]);
        assert_eq!(pc.tabbed, None, "a one-view panel is not a tab group");
        assert!(
            shown.node("desktop").is_some(),
            "the desktop layer is shared by every tag"
        );
    }

    #[test]
    fn without_a_tag_the_active_tree_is_the_whole_tree() {
        let doc = LayoutDocument {
            root: workspace_tree(),
            ..Default::default()
        };
        assert_eq!(doc.active_tree(), doc.root);
        assert!(doc.view_visible("s-b"), "no tag shows every view");
    }

    #[test]
    fn an_empty_tag_shows_only_the_desktop() {
        let mut doc = LayoutDocument {
            root: workspace_tree(),
            ..Default::default()
        };
        doc.ensure_tag("empty");
        doc.active_tag = Some("empty".into());
        assert_eq!(doc.active_tree(), LayoutNode::desktop());
    }

    #[test]
    fn closing_the_last_window_keeps_the_workspace_name() {
        let mut doc = LayoutDocument::default();
        doc.assign_view("s-a", "work");
        doc.forget_view("s-a");
        let tag = doc.tag("work").expect("the name survives the window");
        assert!(tag.views.is_empty());
    }

    #[test]
    fn tags_and_the_active_tag_survive_a_round_trip() {
        let doc = tagged();
        let file = LayoutFile {
            doc,
            ..Default::default()
        };
        let back = LayoutFile::from_json(&file.to_json()).unwrap();
        assert_eq!(back.doc.tags, file.doc.tags);
        assert_eq!(back.doc.active_tag.as_deref(), Some("work"));
    }

    #[test]
    fn an_older_reader_still_sees_every_view() {
        // The claim in the type's doc comment: a binary that predates tags knows
        // only `root`, and the views must all still be there. Parsing into a
        // struct that does not mention `tags` is exactly what serde does in an
        // older build.
        #[derive(serde::Deserialize)]
        struct OldFile {
            doc: OldDoc,
        }
        #[derive(serde::Deserialize)]
        struct OldDoc {
            root: LayoutNode,
        }
        let file = LayoutFile {
            doc: tagged(),
            ..Default::default()
        };
        let json = file.to_json();
        assert!(json.contains("\"tags\""), "the tag list is in the file");
        let old: OldFile = serde_json::from_str(&json).unwrap();
        let views: Vec<&str> = old
            .doc
            .root
            .flatten()
            .iter()
            .filter_map(|n| match n {
                LayoutNode::Panel(p) => Some(p.views.iter().map(String::as_str)),
                _ => None,
            })
            .flatten()
            .collect();
        assert_eq!(views, vec!["s-a", "s-b", "s-c", "s-d"]);
    }

    #[test]
    fn broken_weights_are_repaired_on_load() {
        let raw = r#"{
            "doc": {"root": {"kind":"split","id":"r","axis":"x",
                "children":[{"kind":"panel","id":"p","views":["s"]},{"kind":"desktop","id":"desktop"}],
                "weights":[3.0, 1.0]}}
        }"#;
        let file = LayoutFile::from_json(raw).unwrap();
        file.doc.root.validate().unwrap();
    }
}
