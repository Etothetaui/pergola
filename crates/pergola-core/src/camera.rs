//! The camera: a framed node id plus browser-like history. There is no hidden
//! zoom engine — `framed` points at the node being looked at, and the back and
//! forward stacks remember where to return to.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine::LayoutEngine;
use crate::geom::{Direction, Rect};
use crate::node::LayoutNode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraState {
    /// Node id currently framed. `"desktop"` is both the floating-layer leaf id
    /// and the value used when everything is floating.
    pub framed: String,
    #[serde(default)]
    pub back_stack: Vec<String>,
    #[serde(default)]
    pub forward_stack: Vec<String>,
    /// Slots 1..=9.
    #[serde(default)]
    pub saved_views: BTreeMap<u8, String>,
    #[serde(default)]
    pub desktop_hidden: bool,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            framed: String::new(),
            back_stack: Vec::new(),
            forward_stack: Vec::new(),
            saved_views: BTreeMap::new(),
            desktop_hidden: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CameraAction {
    Everything,
    ZoomIn { toward_seat: Option<String> },
    ZoomOut,
    Frame { node_id: String },
    FrameDesktop,
    ToggleDesktop,
    Back,
    Forward,
    SaveView(u8),
    GoToView(u8),
    Neighbour { direction: Direction, frame: Rect },
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Camera {
    pub engine: LayoutEngine,
}

impl Camera {
    pub fn new(engine: LayoutEngine) -> Self {
        Self { engine }
    }

    pub fn apply(
        &self,
        action: &CameraAction,
        state: &CameraState,
        tree: &LayoutNode,
        _frame: Rect,
    ) -> CameraState {
        match action {
            CameraAction::Everything => self.move_to(tree.id(), state),

            CameraAction::ZoomOut => {
                let path = self.engine.path(&state.framed, tree);
                let target = if path.len() > 1 {
                    path[path.len() - 2].clone()
                } else {
                    tree.id().to_string()
                };
                self.move_to(&target, state)
            }

            CameraAction::ZoomIn { toward_seat } => {
                let target = toward_seat
                    .as_deref()
                    .and_then(|seat| self.engine.zoom_target(seat, tree))
                    .or_else(|| {
                        tree.node(&state.framed)
                            .and_then(|n| n.children().iter().find(|c| !c.is_desktop()))
                            .map(|c| c.id().to_string())
                    })
                    .or_else(|| {
                        tree.leaves()
                            .into_iter()
                            .find(|n| !n.is_desktop())
                            .map(|n| n.id().to_string())
                    })
                    .unwrap_or_else(|| state.framed.clone());
                self.move_to(&target, state)
            }

            CameraAction::Frame { node_id } => {
                if tree.node(node_id).is_none() {
                    return state.clone();
                }
                // "again to go back": framing the same node returns to the root.
                if &state.framed == node_id {
                    return self.move_to(tree.id(), state);
                }
                self.move_to(node_id, state)
            }

            CameraAction::FrameDesktop => {
                if state.framed == "desktop" {
                    self.move_to(tree.id(), state)
                } else {
                    self.move_to("desktop", state)
                }
            }

            CameraAction::ToggleDesktop => {
                let mut next = state.clone();
                next.desktop_hidden = !next.desktop_hidden;
                next
            }

            CameraAction::Back => match state.back_stack.last() {
                Some(previous) => {
                    let mut next = state.clone();
                    next.back_stack.pop();
                    next.forward_stack.push(state.framed.clone());
                    next.framed = previous.clone();
                    next
                }
                None => state.clone(),
            },

            CameraAction::Forward => match state.forward_stack.last() {
                Some(following) => {
                    let mut next = state.clone();
                    next.forward_stack.pop();
                    next.back_stack.push(state.framed.clone());
                    next.framed = following.clone();
                    next
                }
                None => state.clone(),
            },

            CameraAction::SaveView(n) => {
                if !(1..=9).contains(n) {
                    return state.clone();
                }
                let mut next = state.clone();
                next.saved_views.insert(*n, state.framed.clone());
                next
            }

            CameraAction::GoToView(n) => {
                if !(1..=9).contains(n) {
                    return state.clone();
                }
                match state.saved_views.get(n) {
                    Some(target) if tree.node(target).is_some() || target == "desktop" => {
                        self.move_to(target, state)
                    }
                    _ => state.clone(),
                }
            }

            CameraAction::Neighbour { direction, frame } => {
                match self
                    .engine
                    .neighbour(&state.framed, *direction, tree, *frame)
                {
                    Some(target) => self.move_to(&target, state),
                    None => state.clone(),
                }
            }
        }
    }

    fn move_to(&self, target: &str, state: &CameraState) -> CameraState {
        if target == state.framed {
            return state.clone();
        }
        let mut next = state.clone();
        next.back_stack.push(state.framed.clone());
        next.forward_stack.clear();
        next.framed = target.to_string();
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Axis;

    fn tree() -> LayoutNode {
        let inner = LayoutNode::split(
            "inner",
            Axis::Y,
            vec![
                LayoutNode::panel("p-b", vec!["s-b".into()]),
                LayoutNode::panel("p-c", vec!["s-c".into()]),
            ],
        );
        LayoutNode::split(
            "root",
            Axis::X,
            vec![
                LayoutNode::panel("p-a", vec!["s-a".into()]),
                inner,
                LayoutNode::desktop(),
            ],
        )
    }

    fn frame() -> Rect {
        Rect::new(0.0, 0.0, 1000.0, 600.0)
    }

    #[test]
    fn zoom_in_and_out_walk_the_tree() {
        let cam = Camera::default();
        let t = tree();
        let s0 = CameraState::default();
        let s1 = cam.apply(&CameraAction::Everything, &s0, &t, frame());
        assert_eq!(s1.framed, "root");
        let s2 = cam.apply(&CameraAction::ZoomIn { toward_seat: Some("s-c".into()) }, &s1, &t, frame());
        assert_eq!(s2.framed, "inner");
        let s3 = cam.apply(&CameraAction::ZoomOut, &s2, &t, frame());
        assert_eq!(s3.framed, "root");
    }

    #[test]
    fn back_and_forward_are_browser_like() {
        let cam = Camera::default();
        let t = tree();
        let s0 = CameraState::default();
        let s1 = cam.apply(&CameraAction::Frame { node_id: "p-a".into() }, &s0, &t, frame());
        let s2 = cam.apply(&CameraAction::Frame { node_id: "p-b".into() }, &s1, &t, frame());
        let back = cam.apply(&CameraAction::Back, &s2, &t, frame());
        assert_eq!(back.framed, "p-a");
        let fwd = cam.apply(&CameraAction::Forward, &back, &t, frame());
        assert_eq!(fwd.framed, "p-b");
    }

    #[test]
    fn framing_the_same_node_returns_to_root() {
        let cam = Camera::default();
        let t = tree();
        let s1 = cam.apply(&CameraAction::Frame { node_id: "p-a".into() }, &CameraState::default(), &t, frame());
        let s2 = cam.apply(&CameraAction::Frame { node_id: "p-a".into() }, &s1, &t, frame());
        assert_eq!(s2.framed, "root");
    }

    #[test]
    fn saved_views_round_trip() {
        let cam = Camera::default();
        let t = tree();
        let s1 = cam.apply(&CameraAction::Frame { node_id: "p-b".into() }, &CameraState::default(), &t, frame());
        let s2 = cam.apply(&CameraAction::SaveView(3), &s1, &t, frame());
        let s3 = cam.apply(&CameraAction::Everything, &s2, &t, frame());
        let s4 = cam.apply(&CameraAction::GoToView(3), &s3, &t, frame());
        assert_eq!(s4.framed, "p-b");
        // Out-of-range slots are ignored.
        let s5 = cam.apply(&CameraAction::GoToView(0), &s4, &t, frame());
        assert_eq!(s5.framed, "p-b");
    }

    #[test]
    fn neighbour_moves_sideways() {
        let cam = Camera::default();
        let t = tree();
        let s1 = cam.apply(&CameraAction::Frame { node_id: "p-a".into() }, &CameraState::default(), &t, frame());
        let s2 = cam.apply(
            &CameraAction::Neighbour { direction: Direction::Right, frame: frame() },
            &s1,
            &t,
            frame(),
        );
        assert_eq!(s2.framed, "p-b");
    }

    #[test]
    fn toggle_desktop_flips_the_flag() {
        let cam = Camera::default();
        let t = tree();
        let s1 = cam.apply(&CameraAction::ToggleDesktop, &CameraState::default(), &t, frame());
        assert!(s1.desktop_hidden);
        let s2 = cam.apply(&CameraAction::ToggleDesktop, &s1, &t, frame());
        assert!(!s2.desktop_hidden);
    }
}
