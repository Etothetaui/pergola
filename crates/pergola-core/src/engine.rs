//! Pure layout geometry and tree mutations. No X11, no global state: every
//! operation returns a new tree, which is what makes the model testable.

use crate::geom::{Axis, Direction, Rect};
use crate::node::{LayoutError, LayoutNode, Panel, Split};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutEngine {
    /// Space between sibling tiles, in pixels.
    pub gap: f64,
    /// Smallest share a tile may be squeezed to when dragging a gap.
    pub minimum_share: f64,
    /// Share of the space the desktop leaf keeps when a window is docked
    /// beside it. 0 means the docked window takes the whole space and the
    /// desktop becomes a zero-width floating layer, which is what a tiling
    /// desktop expects; raise it to keep a visible desktop strip.
    pub desktop_share: f64,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            gap: 8.0,
            minimum_share: 0.05,
            desktop_share: 0.0,
        }
    }
}

/// A split boundary the pointer can drag: the gap rectangle plus what a pixel
/// delta means in weight units.
#[derive(Debug, Clone, PartialEq)]
pub struct GapHandle {
    pub split: String,
    pub first: String,
    pub second: String,
    pub axis: Axis,
    /// Hit area covering the whole gap between the two children.
    pub band: Rect,
    /// Length of the split along `axis` — divide a pixel delta by this.
    pub extent: f64,
}

impl LayoutEngine {
    pub fn new(gap: f64, minimum_share: f64) -> Self {
        Self {
            gap: gap.max(0.0),
            minimum_share: minimum_share.clamp(0.001, 0.49),
            desktop_share: 0.0,
        }
    }

    /// Keep `share` of the space for the desktop leaf when docking beside it.
    pub fn with_desktop_share(mut self, share: f64) -> Self {
        self.desktop_share = share.clamp(0.0, 0.9);
        self
    }

    // MARK: geometry

    /// Map every node id to its rectangle inside `frame`. Split weights consume
    /// the space left after `(children - 1) * gap` has been removed.
    pub fn rects(&self, root: &LayoutNode, frame: Rect) -> Vec<(String, Rect)> {
        let mut out = Vec::new();
        self.place(root, frame, &mut out);
        out
    }

    /// The draggable boundaries of the tree: one [`GapHandle`] per gap between
    /// two adjacent siblings that both have space.
    ///
    /// This is the pointer-facing view of a layout, so the daemon does not have
    /// to re-derive geometry for the mouse: a handle is a hit rectangle in the
    /// same coordinates as [`Self::rects`], plus the denominator a pixel delta
    /// is divided by to become a weight delta.
    pub fn gap_handles(&self, root: &LayoutNode, frame: Rect) -> Vec<GapHandle> {
        let rects = self.rects(root, frame);
        let find = |id: &str| {
            rects
                .iter()
                .find(|(nid, _)| nid == id)
                .map(|(_, r)| *r)
        };
        let mut out = Vec::new();
        for node in root.flatten() {
            let LayoutNode::Split(split) = node else {
                continue;
            };
            let Some(parent) = find(&split.id) else {
                continue;
            };
            for i in 0..split.children.len().saturating_sub(1) {
                let first = &split.children[i];
                let second = &split.children[i + 1];
                // A zero-weight child is collapsed and has no gap to drag.
                let wa = split.weights.get(i).copied().unwrap_or(0.0);
                let wb = split.weights.get(i + 1).copied().unwrap_or(0.0);
                if wa <= 0.0 || wb <= 0.0 {
                    continue;
                }
                let (Some(a), Some(b)) = (find(first.id()), find(second.id())) else {
                    continue;
                };
                let (band, extent) = match split.axis {
                    Axis::X => (
                        Rect::new(
                            a.max_x(),
                            parent.y,
                            (b.x - a.max_x()).max(0.0),
                            parent.h,
                        ),
                        parent.w,
                    ),
                    Axis::Y => (
                        Rect::new(
                            parent.x,
                            a.max_y(),
                            parent.w,
                            (b.y - a.max_y()).max(0.0),
                        ),
                        parent.h,
                    ),
                };
                // With gap 0 there is no band to hit; the user can still resize
                // by setting a gap first.
                if band.w < 1.0 || band.h < 1.0 {
                    continue;
                }
                out.push(GapHandle {
                    split: split.id.clone(),
                    first: first.id().to_string(),
                    second: second.id().to_string(),
                    axis: split.axis,
                    band,
                    extent,
                });
            }
        }
        out
    }

    /// Convenience lookup over [`Self::rects`].
    pub fn rect_of(&self, root: &LayoutNode, frame: Rect, id: &str) -> Option<Rect> {
        self.rects(root, frame)
            .into_iter()
            .find(|(nid, _)| nid == id)
            .map(|(_, r)| r)
    }

    fn place(&self, node: &LayoutNode, frame: Rect, out: &mut Vec<(String, Rect)>) {
        out.push((node.id().to_string(), frame));
        let LayoutNode::Split(s) = node else { return };
        if s.children.is_empty() {
            return;
        }
        // A zero-weight child (a collapsed desktop leaf) takes no space and no
        // gap: otherwise docking beside the desktop would leave a black strip.
        let visible = s
            .children
            .iter()
            .zip(s.weights.iter())
            .filter(|(_, w)| **w > 0.0)
            .count();
        let total = match s.axis {
            Axis::X => frame.w,
            Axis::Y => frame.h,
        };
        let usable = (total - self.gap * (visible.saturating_sub(1)) as f64).max(0.0);
        let mut cursor = match s.axis {
            Axis::X => frame.x,
            Axis::Y => frame.y,
        };
        let mut placed = 0;
        for (i, child) in s.children.iter().enumerate() {
            let weight = s.weights.get(i).copied().unwrap_or(0.0);
            let length = usable * weight;
            let child_frame = match s.axis {
                Axis::X => Rect::new(cursor, frame.y, length, frame.h),
                Axis::Y => Rect::new(frame.x, cursor, frame.w, length),
            };
            self.place(child, child_frame, out);
            if weight > 0.0 {
                placed += 1;
                cursor += length;
                if placed < visible {
                    cursor += self.gap;
                }
            }
        }
    }

    // MARK: docking

    /// Dock `panel` against `target_id` on `edge`.
    ///
    /// * parent split on the same axis → insert as a sibling, taking half of the
    ///   target's weight;
    /// * parent split on the other axis → wrap the target in a new split on the
    ///   edge's axis, so nesting grows one level;
    /// * target is a root leaf → create a new root split, keeping `desktop` intact.
    pub fn inserting(
        &self,
        panel: LayoutNode,
        target_id: &str,
        edge: Direction,
        root: &LayoutNode,
    ) -> Result<LayoutNode, LayoutError> {
        if root.node(target_id).is_none() {
            return Err(LayoutError::NotFound(format!("target '{target_id}'")));
        }
        // Docking beside the whole tree wraps the root in a new split.
        if target_id == root.id() && !root.is_leaf() {
            let pair = if edge.is_negative() {
                vec![panel, root.clone()]
            } else {
                vec![root.clone(), panel]
            };
            let out = LayoutNode::Split(Split::with_weights(
                self.fresh_split_id(root),
                edge.axis(),
                pair,
                vec![0.5, 0.5],
            ));
            out.validate()?;
            return Ok(out);
        }
        let (result, inserted) = self.insert(panel, target_id, edge, root, root);
        if !inserted {
            return Err(LayoutError::NotFound(format!(
                "target '{target_id}' is not dockable"
            )));
        }
        result.validate()?;
        Ok(result)
    }

    fn insert(
        &self,
        new: LayoutNode,
        target_id: &str,
        edge: Direction,
        node: &LayoutNode,
        root: &LayoutNode,
    ) -> (LayoutNode, bool) {
        if let LayoutNode::Split(s) = node {
            if let Some(idx) = s.children.iter().position(|c| c.id() == target_id) {
                if s.axis == edge.axis() {
                    let mut kids = s.children.clone();
                    let mut weights = s.weights.clone();
                    let share = weights[idx] / 2.0;
                    weights[idx] -= share;
                    let at = if edge.is_negative() { idx } else { idx + 1 };
                    kids.insert(at, new);
                    weights.insert(at, share);
                    let updated = Split::with_weights(s.id.clone(), s.axis, kids, weights);
                    return (LayoutNode::Split(updated), true);
                }
                let mut kids = s.children.clone();
                let pair = if edge.is_negative() {
                    vec![new, kids[idx].clone()]
                } else {
                    vec![kids[idx].clone(), new]
                };
                kids[idx] = LayoutNode::Split(Split::with_weights(
                    self.fresh_split_id(root),
                    edge.axis(),
                    pair,
                    vec![0.5, 0.5],
                ));
                let updated = Split::with_weights(s.id.clone(), s.axis, kids, s.weights.clone());
                return (LayoutNode::Split(updated), true);
            }
            let mut kids = s.children.clone();
            for i in 0..kids.len() {
                let (child, done) = self.insert(new.clone(), target_id, edge, &kids[i], root);
                if done {
                    kids[i] = child;
                    let updated =
                        Split::with_weights(s.id.clone(), s.axis, kids, s.weights.clone());
                    return (LayoutNode::Split(updated), true);
                }
            }
            return (node.clone(), false);
        }

        if node.id() == target_id {
            // Never dock *into* the desktop; put the panel beside it. The
            // desktop keeps `desktop_share` of the space (0 by default, so the
            // docked window takes it all and no black strip appears).
            let share = self.desktop_share;
            let pair = if edge.is_negative() {
                vec![new, node.clone()]
            } else {
                vec![node.clone(), new]
            };
            let weights = if edge.is_negative() {
                vec![1.0 - share, share]
            } else {
                vec![share, 1.0 - share]
            };
            return (
                LayoutNode::Split(Split::with_weights(
                    self.fresh_split_id(root),
                    edge.axis(),
                    pair,
                    weights,
                )),
                true,
            );
        }
        (node.clone(), false)
    }

    fn fresh_split_id(&self, root: &LayoutNode) -> String {
        let existing: Vec<&str> = root.flatten().iter().map(|n| n.id()).collect();
        let mut n = existing.len();
        loop {
            let candidate = format!("split-gen{n}");
            if !existing.contains(&candidate.as_str()) {
                return candidate;
            }
            n += 1;
        }
    }

    // MARK: removal

    /// Remove a docked panel (or an entire sub-split). A split left with one
    /// child collapses into that child. The `desktop` leaf is protected.
    pub fn removing(&self, node_id: &str, root: &LayoutNode) -> Result<LayoutNode, LayoutError> {
        if root.node(node_id).is_none() {
            return Err(LayoutError::NotFound(format!("node '{node_id}'")));
        }
        if node_id == "desktop" {
            return Err(LayoutError::Invariant(
                "the desktop leaf cannot be removed".into(),
            ));
        }
        let (result, removed) = self.remove(node_id, root);
        if !removed {
            return Err(LayoutError::NotFound(format!(
                "node '{node_id}' is not removable"
            )));
        }
        result.validate()?;
        Ok(result)
    }

    fn remove(&self, id: &str, node: &LayoutNode) -> (LayoutNode, bool) {
        let LayoutNode::Split(s) = node else {
            return (node.clone(), false);
        };
        if let Some(idx) = s.children.iter().position(|c| c.id() == id) {
            if s.children[idx].is_desktop() {
                return (node.clone(), false);
            }
            let mut kids = s.children.clone();
            kids.remove(idx);
            let mut weights = s.weights.clone();
            weights.remove(idx);
            if kids.len() == 1 {
                return (kids.remove(0), true); // collapse
            }
            return (
                LayoutNode::Split(Split::with_weights(s.id.clone(), s.axis, kids, weights)),
                true,
            );
        }
        let mut kids = s.children.clone();
        for i in 0..kids.len() {
            let (child, done) = self.remove(id, &kids[i]);
            if done {
                kids[i] = child;
                let updated = Split::with_weights(s.id.clone(), s.axis, kids, s.weights.clone());
                return (LayoutNode::Split(updated), true);
            }
        }
        (node.clone(), false)
    }

    // MARK: gap resize

    /// Drag the gap between two adjacent siblings. `delta` is a fraction of the
    /// parent's usable length; positive moves the gap toward `second_id`.
    pub fn resizing_gap(
        &self,
        first_id: &str,
        second_id: &str,
        delta: f64,
        root: &LayoutNode,
    ) -> Result<LayoutNode, LayoutError> {
        let parent = root
            .parent(first_id)
            .ok_or_else(|| LayoutError::NotFound(format!("parent of '{first_id}'")))?;
        if !parent.children.iter().any(|c| c.id() == second_id) {
            return Err(LayoutError::NotFound(format!(
                "'{first_id}' and '{second_id}' are not siblings"
            )));
        }
        let a = parent
            .children
            .iter()
            .position(|c| c.id() == first_id)
            .unwrap();
        let b = parent
            .children
            .iter()
            .position(|c| c.id() == second_id)
            .unwrap();
        if a.abs_diff(b) != 1 {
            return Err(LayoutError::Invariant(
                "gap drag requires adjacent siblings".into(),
            ));
        }
        let result = self.resize(
            &parent.id,
            a.min(b),
            a.max(b),
            delta,
            root,
        );
        result.validate()?;
        Ok(result)
    }

    fn resize(&self, parent_id: &str, a: usize, b: usize, delta: f64, node: &LayoutNode) -> LayoutNode {
        let LayoutNode::Split(s) = node else {
            return node.clone();
        };
        if s.id == parent_id {
            let mut weights = s.weights.clone();
            let upper = (weights[b] - self.minimum_share).max(0.0);
            let lower = (self.minimum_share - weights[a]).min(0.0);
            let clamped = if delta >= 0.0 {
                delta.min(upper)
            } else {
                delta.max(lower)
            };
            weights[a] += clamped;
            weights[b] -= clamped;
            return LayoutNode::Split(Split::with_weights(
                s.id.clone(),
                s.axis,
                s.children.clone(),
                weights,
            ));
        }
        let kids: Vec<LayoutNode> = s
            .children
            .iter()
            .map(|c| self.resize(parent_id, a, b, delta, c))
            .collect();
        LayoutNode::Split(Split::with_weights(
            s.id.clone(),
            s.axis,
            kids,
            s.weights.clone(),
        ))
    }

    // MARK: neighbours and camera targets

    /// The nearest leaf in `direction` from `from_id`, measured in laid-out space.
    pub fn neighbour(
        &self,
        from_id: &str,
        direction: Direction,
        root: &LayoutNode,
        frame: Rect,
    ) -> Option<String> {
        let rects = self.rects(root, frame);
        let origin = rects.iter().find(|(id, _)| id == from_id)?.1;
        let leaves: Vec<&str> = root
            .leaves()
            .into_iter()
            .filter(|n| !n.is_desktop())
            .map(|n| n.id())
            .collect();
        let mut best: Option<(String, f64)> = None;
        for (id, c) in &rects {
            if id == from_id || !leaves.contains(&id.as_str()) {
                continue;
            }
            let (along, cross) = match direction {
                Direction::Left => (origin.mid_x() - c.mid_x(), (origin.mid_y() - c.mid_y()).abs()),
                Direction::Right => (c.mid_x() - origin.mid_x(), (origin.mid_y() - c.mid_y()).abs()),
                Direction::Up => (origin.mid_y() - c.mid_y(), (origin.mid_x() - c.mid_x()).abs()),
                Direction::Down => (c.mid_y() - origin.mid_y(), (origin.mid_x() - c.mid_x()).abs()),
            };
            if along <= 0.0 {
                continue;
            }
            let score = along + cross * 0.001;
            if best.as_ref().map(|(_, s)| score < *s).unwrap_or(true) {
                best = Some((id.clone(), score));
            }
        }
        best.map(|(id, _)| id)
    }

    /// Ancestor chain from root to `node_id`, used for step in/out.
    pub fn path(&self, node_id: &str, root: &LayoutNode) -> Vec<String> {
        fn walk(node: &LayoutNode, acc: &mut Vec<String>, target: &str) -> bool {
            acc.push(node.id().to_string());
            if node.id() == target {
                return true;
            }
            for child in node.children() {
                if walk(child, acc, target) {
                    return true;
                }
            }
            acc.pop();
            false
        }
        let mut acc = Vec::new();
        if walk(root, &mut acc, node_id) {
            acc
        } else {
            vec![root.id().to_string()]
        }
    }

    /// The deepest split containing `seat_id`, or the panel itself when it is a
    /// direct child of the root. Used by `zoom-in`.
    pub fn zoom_target(&self, seat_id: &str, root: &LayoutNode) -> Option<String> {
        fn walk<'a>(node: &'a LayoutNode, parent: Option<&'a str>, seat_id: &str, root: &'a LayoutNode) -> Option<String> {
            match node {
                LayoutNode::Desktop(_) => None,
                LayoutNode::Panel(p) => {
                    if p.views.iter().any(|v| v == seat_id) {
                        match parent {
                            // A panel that is a direct child of the root frames
                            // itself; anything deeper frames its parent split.
                            Some(pid) if pid != root.id() => Some(pid.to_string()),
                            _ => Some(p.id.clone()),
                        }
                    } else {
                        None
                    }
                }
                LayoutNode::Split(s) => {
                    for child in &s.children {
                        if let Some(hit) = walk(child, Some(&s.id), seat_id, root) {
                            return Some(hit);
                        }
                    }
                    None
                }
            }
        }
        walk(root, None, seat_id, root)
    }

    /// Build a balanced fractal tree from `panels`, keeping the desktop leaf.
    /// Alternates the split axis with depth so the result nests rather than
    /// producing one long row.
    pub fn arrange(&self, panels: Vec<LayoutNode>, desktop_share: f64) -> LayoutNode {
        if panels.is_empty() {
            return LayoutNode::desktop();
        }
        let desktop_share = desktop_share.clamp(0.0, 0.9);
        let body = self.balanced(panels, Axis::X, 0);
        // The desktop leaf is always kept, even at share 0 (a zero-width space)
        // so the one-desktop invariant holds.
        LayoutNode::Split(Split::with_weights(
            "root",
            Axis::X,
            vec![body, LayoutNode::desktop()],
            vec![1.0 - desktop_share, desktop_share],
        ))
    }

    fn balanced(&self, mut panels: Vec<LayoutNode>, axis: Axis, depth: usize) -> LayoutNode {
        if panels.len() == 1 {
            return panels.remove(0);
        }
        let mid = panels.len() / 2;
        let right = panels.split_off(mid);
        let left = panels;
        let next = if axis == Axis::X { Axis::Y } else { Axis::X };
        let a = self.balanced(left, next, depth + 1);
        let b = self.balanced(right, next, depth + 1);
        LayoutNode::Split(Split::with_weights(
            format!("split-arrange-{depth}-{}", a.id()),
            axis,
            vec![a, b],
            vec![0.5, 0.5],
        ))
    }

    /// Replace the panel holding `seat_id` with a tab group of `views`.
    pub fn tabbing(&self, seat_id: &str, views: Vec<String>, root: &LayoutNode) -> Option<LayoutNode> {
        let panel = root.panel_containing(seat_id)?;
        let id = panel.id.clone();
        let next = views;
        if next.is_empty() {
            return None;
        }
        let replacement = LayoutNode::Panel(Panel::tab_group(id, next));
        Some(replace_node(root, &panel.id.clone(), &replacement))
    }
}

/// Structural replacement by id, preserving weights.
pub fn replace_node(root: &LayoutNode, id: &str, replacement: &LayoutNode) -> LayoutNode {
    if root.id() == id {
        return replacement.clone();
    }
    match root {
        LayoutNode::Split(s) => {
            let kids: Vec<LayoutNode> = s
                .children
                .iter()
                .map(|c| replace_node(c, id, replacement))
                .collect();
            LayoutNode::Split(Split::with_weights(
                s.id.clone(),
                s.axis,
                kids,
                s.weights.clone(),
            ))
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> Rect {
        Rect::new(0.0, 0.0, 1000.0, 600.0)
    }

    fn panel(id: &str) -> LayoutNode {
        LayoutNode::panel(format!("p-{id}"), vec![format!("s-{id}")])
    }

    #[test]
    fn rects_respect_gap_and_weights() {
        let e = LayoutEngine::new(10.0, 0.05);
        let tree = LayoutNode::Split(Split::with_weights(
            "root",
            Axis::X,
            vec![panel("a"), panel("b")],
            vec![0.25, 0.75],
        ));
        let r = e.rects(&tree, frame());
        let a = r.iter().find(|(id, _)| id == "p-a").unwrap().1;
        let b = r.iter().find(|(id, _)| id == "p-b").unwrap().1;
        assert!((a.w - 247.5).abs() < 1e-9);
        assert!((b.x - 257.5).abs() < 1e-9);
        assert!((b.w - 742.5).abs() < 1e-9);
        assert_eq!(a.h, 600.0);
    }

    #[test]
    fn gap_handles_describe_each_draggable_boundary() {
        let e = LayoutEngine::new(10.0, 0.05);
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), panel("b")]);
        let handles = e.gap_handles(&tree, frame());
        assert_eq!(handles.len(), 1);
        let h = &handles[0];
        assert_eq!(h.split, "root");
        assert_eq!((h.first.as_str(), h.second.as_str()), ("p-a", "p-b"));
        assert_eq!(h.axis, Axis::X);
        assert_eq!(h.extent, 1000.0);
        // 1000 px split, 10 px gap: two 495 px tiles, the band is the gap.
        assert_eq!(h.band, Rect::new(495.0, 0.0, 10.0, 600.0));
    }

    #[test]
    fn gap_handles_skip_collapsed_children() {
        let e = LayoutEngine::new(6.0, 0.05);
        let tree = LayoutNode::Split(Split::with_weights(
            "root",
            Axis::X,
            vec![panel("a"), panel("b"), LayoutNode::desktop()],
            vec![0.5, 0.5, 0.0],
        ));
        let handles = e.gap_handles(&tree, frame());
        // a|b is draggable; b|desktop has no space on the desktop side.
        assert_eq!(handles.len(), 1);
        assert_eq!(handles[0].second, "p-b");
    }

    #[test]
    fn gap_handles_cover_nested_splits_and_both_axes() {
        let e = LayoutEngine::new(6.0, 0.05);
        let inner = LayoutNode::split("inner", Axis::Y, vec![panel("b"), panel("c")]);
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), inner]);
        let handles = e.gap_handles(&tree, frame());
        assert_eq!(handles.len(), 2);
        let root = handles.iter().find(|h| h.split == "root").unwrap();
        let nested = handles.iter().find(|h| h.split == "inner").unwrap();
        assert_eq!(root.axis, Axis::X);
        assert_eq!(nested.axis, Axis::Y);
        // The nested handle spans the inner split's width and sits in its gap.
        assert!((nested.band.w - (1000.0 - 6.0) / 2.0).abs() < 1e-9);
        assert!(nested.band.h >= 6.0 - 1e-9);
    }

    #[test]
    fn a_single_panel_has_no_gap_to_drag() {
        let e = LayoutEngine::default();
        assert!(e.gap_handles(&panel("a"), frame()).is_empty());
    }

    #[test]
    fn nested_rects_are_contained() {
        let e = LayoutEngine::default();
        let inner = LayoutNode::split("inner", Axis::Y, vec![panel("b"), panel("c")]);
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), inner, LayoutNode::desktop()]);
        let r = e.rects(&tree, frame());
        let a = r.iter().find(|(id, _)| id == "p-a").unwrap().1;
        let b = r.iter().find(|(id, _)| id == "p-b").unwrap().1;
        assert!(b.x >= a.max_x());
        assert!(b.max_y() <= 600.0 + 1e-9);
    }

    #[test]
    fn dock_same_axis_inserts_sibling() {
        let e = LayoutEngine::default();
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), panel("b"), LayoutNode::desktop()]);
        let out = e.inserting(panel("c"), "p-a", Direction::Right, &tree).unwrap();
        let LayoutNode::Split(s) = &out else { panic!() };
        assert_eq!(s.children.len(), 4);
        assert_eq!(s.children[1].id(), "p-c");
        assert!((s.weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn dock_other_axis_nests() {
        let e = LayoutEngine::default();
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), panel("b"), LayoutNode::desktop()]);
        let out = e.inserting(panel("c"), "p-a", Direction::Down, &tree).unwrap();
        assert_eq!(out.nesting_depth(), 2);
        let LayoutNode::Split(s) = &out else { panic!() };
        let LayoutNode::Split(inner) = &s.children[0] else { panic!() };
        assert_eq!(inner.axis, Axis::Y);
        assert_eq!(inner.children[1].id(), "p-c");
    }

    #[test]
    fn dock_beside_desktop_takes_the_space_by_default() {
        let e = LayoutEngine::default();
        let out = e
            .inserting(panel("a"), "desktop", Direction::Left, &LayoutNode::desktop())
            .unwrap();
        let LayoutNode::Split(s) = &out else { panic!() };
        assert_eq!(s.weights, vec![1.0, 0.0]);
        let r = e.rects(&out, frame());
        let a = r.iter().find(|(id, _)| id == "p-a").unwrap().1;
        assert!((a.w - 1000.0).abs() < 1e-9, "the docked window takes the whole frame");
        // A configured share keeps a visible desktop strip.
        let e2 = LayoutEngine::default().with_desktop_share(0.25);
        let out2 = e2
            .inserting(panel("a"), "desktop", Direction::Left, &LayoutNode::desktop())
            .unwrap();
        let LayoutNode::Split(s2) = &out2 else { panic!() };
        assert!((s2.weights[1] - 0.25).abs() < 1e-9);
    }

    #[test]
    fn dock_beside_desktop_keeps_desktop_leaf() {
        let e = LayoutEngine::default();
        let out = e.inserting(panel("a"), "desktop", Direction::Left, &LayoutNode::desktop()).unwrap();
        out.validate().unwrap();
        assert_eq!(out.nesting_depth(), 1);
        let LayoutNode::Split(s) = &out else { panic!() };
        assert_eq!(s.children[0].id(), "p-a");
        assert!(s.children[1].is_desktop());
    }

    #[test]
    fn remove_collapses_single_child() {
        let e = LayoutEngine::default();
        let inner = LayoutNode::split("inner", Axis::Y, vec![panel("b"), LayoutNode::desktop()]);
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), inner]);
        let out = e.removing("p-a", &tree).unwrap();
        assert_eq!(out.id(), "inner");
        assert!(e.removing("desktop", &tree).is_err());
        assert!(e.removing("nope", &tree).is_err());
    }

    #[test]
    fn gap_resize_is_clamped() {
        let e = LayoutEngine::new(0.0, 0.1);
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), panel("b"), LayoutNode::desktop()]);
        let out = e.resizing_gap("p-a", "p-b", 0.9, &tree).unwrap();
        let LayoutNode::Split(s) = &out else { panic!() };
        assert!((s.weights[1] - 0.1).abs() < 1e-9);
        let out2 = e.resizing_gap("p-a", "p-b", -0.9, &tree).unwrap();
        let LayoutNode::Split(s2) = &out2 else { panic!() };
        assert!((s2.weights[0] - 0.1).abs() < 1e-9);
    }

    #[test]
    fn neighbour_picks_the_adjacent_tile() {
        let e = LayoutEngine::default();
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), panel("b"), panel("c"), LayoutNode::desktop()]);
        assert_eq!(e.neighbour("p-a", Direction::Right, &tree, frame()).as_deref(), Some("p-b"));
        assert_eq!(e.neighbour("p-c", Direction::Left, &tree, frame()).as_deref(), Some("p-b"));
        assert_eq!(e.neighbour("p-a", Direction::Left, &tree, frame()), None);
    }

    #[test]
    fn path_and_zoom_target() {
        let e = LayoutEngine::default();
        let inner = LayoutNode::split("inner", Axis::Y, vec![panel("b"), panel("c")]);
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), inner, LayoutNode::desktop()]);
        assert_eq!(e.path("p-c", &tree), vec!["root", "inner", "p-c"]);
        assert_eq!(e.zoom_target("s-c", &tree).as_deref(), Some("inner"));
        assert_eq!(e.zoom_target("s-a", &tree).as_deref(), Some("p-a"));
    }

    #[test]
    fn arrange_is_balanced_and_valid() {
        let e = LayoutEngine::default();
        let panels = vec![panel("a"), panel("b"), panel("c"), panel("d"), panel("e")];
        let tree = e.arrange(panels, 0.2);
        tree.validate().unwrap();
        assert!(tree.nesting_depth() >= 3);
        let r = e.rects(&tree, frame());
        assert_eq!(r.len(), 11); // 5 panels + 4 splits + desktop + root
    }

    #[test]
    fn tabbing_replaces_panel() {
        let e = LayoutEngine::default();
        let tree = LayoutNode::split("root", Axis::X, vec![panel("a"), panel("b"), LayoutNode::desktop()]);
        let out = e
            .tabbing("s-a", vec!["s-a".into(), "s-x".into()], &tree)
            .unwrap();
        let p = out.panel_containing("s-x").unwrap();
        assert!(p.is_tab_group());
        assert_eq!(p.id, "p-a");
    }
}
