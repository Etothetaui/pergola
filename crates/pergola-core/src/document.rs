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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutDocument {
    pub root: LayoutNode,
    #[serde(default)]
    pub floating: Vec<FloatingWindow>,
    #[serde(default)]
    pub hidden: Vec<String>,
}

impl Default for LayoutDocument {
    fn default() -> Self {
        Self {
            root: LayoutNode::desktop(),
            floating: Vec::new(),
            hidden: Vec::new(),
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
