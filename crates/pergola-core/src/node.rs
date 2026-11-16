//! The layout tree.
//!
//! ```text
//! Node    := Desktop | Panel | Split
//! Desktop := { kind:"desktop", id:"desktop" }
//! Panel   := { kind:"panel", id, views:[SeatID], selected, tabbed? }
//! Split   := { kind:"split", id, axis:"x"|"y", children:[Node], weights:[f64] }
//! ```
//!
//! A `Split` may contain `Split`s: that recursion is the whole fractal model.
//! Exactly one `Desktop` leaf exists and it is never removed.

use serde::{Deserialize, Serialize};

use crate::geom::Axis;

/// The floating-window layer. Always present, always id `"desktop"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Desktop {
    pub id: String,
}

impl Default for Desktop {
    fn default() -> Self {
        Self {
            id: "desktop".to_string(),
        }
    }
}

/// A leaf holding one window, or a tab group of several.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Panel {
    pub id: String,
    pub views: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub selected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tabbed: Option<bool>,
}

impl Panel {
    pub fn new(id: impl Into<String>, views: Vec<String>) -> Self {
        let views: Vec<String> = views;
        let selected = views.first().cloned();
        Self {
            id: id.into(),
            views,
            selected,
            tabbed: None,
        }
    }

    pub fn tab_group(id: impl Into<String>, views: Vec<String>) -> Self {
        let mut p = Self::new(id, views);
        p.tabbed = Some(true);
        p
    }

    pub fn is_tab_group(&self) -> bool {
        self.views.len() > 1
    }

    /// The seat currently shown in this panel.
    pub fn visible(&self) -> Option<&str> {
        self.selected
            .as_deref()
            .filter(|s| self.views.iter().any(|v| v == s))
            .or_else(|| self.views.first().map(String::as_str))
    }

    /// Show `seat` in this panel. Returns false when the seat is not a view, so
    /// a caller cannot select a window the panel does not hold.
    pub fn select(&mut self, seat: &str) -> bool {
        if !self.views.iter().any(|v| v == seat) {
            return false;
        }
        self.selected = Some(seat.to_string());
        true
    }

    /// Move the selection one view forward (or backward), wrapping at the ends.
    /// Returns the newly selected seat; a single-view panel just reports its
    /// one view.
    pub fn cycle(&mut self, forward: bool) -> Option<String> {
        if self.views.len() < 2 {
            return self.visible().map(str::to_string);
        }
        let current = self
            .visible()
            .and_then(|s| self.views.iter().position(|v| v == s))
            .unwrap_or(0);
        let next = if forward {
            (current + 1) % self.views.len()
        } else {
            (current + self.views.len() - 1) % self.views.len()
        };
        self.selected = Some(self.views[next].clone());
        self.selected.clone()
    }

    /// Drop one view. The selection is repaired to the neighbour that takes its
    /// place — the next view, or the previous one when the last view went — so
    /// a panel never points at a window that is gone. Returns true when views
    /// remain; false means the panel is now empty and should be removed.
    pub fn remove_view(&mut self, seat: &str) -> bool {
        let Some(idx) = self.views.iter().position(|v| v == seat) else {
            return !self.views.is_empty();
        };
        let was_selected = self.visible() == Some(seat);
        self.views.remove(idx);
        if self.views.is_empty() {
            self.selected = None;
            return false;
        }
        if was_selected {
            let next = idx.min(self.views.len() - 1);
            self.selected = Some(self.views[next].clone());
        }
        if self.views.len() == 1 {
            // A group of one is not a group; do not claim otherwise in the
            // document the next load reads.
            self.tabbed = None;
        }
        true
    }
}

/// A container. `children.len() == weights.len()` and `sum(weights) == 1`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Split {
    pub id: String,
    pub axis: Axis,
    pub children: Vec<LayoutNode>,
    pub weights: Vec<f64>,
}

impl Split {
    pub fn new(id: impl Into<String>, axis: Axis, children: Vec<LayoutNode>) -> Self {
        let n = children.len().max(1);
        let weights = vec![1.0 / n as f64; children.len()];
        Self {
            id: id.into(),
            axis,
            children,
            weights,
        }
    }

    pub fn with_weights(
        id: impl Into<String>,
        axis: Axis,
        children: Vec<LayoutNode>,
        weights: Vec<f64>,
    ) -> Self {
        let weights = normalize_weights(&weights, children.len());
        Self {
            id: id.into(),
            axis,
            children,
            weights,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum LayoutNode {
    Desktop(Desktop),
    Panel(Panel),
    Split(Split),
}

impl Default for LayoutNode {
    fn default() -> Self {
        LayoutNode::Desktop(Desktop::default())
    }
}

impl LayoutNode {
    pub fn desktop() -> Self {
        LayoutNode::Desktop(Desktop::default())
    }

    pub fn panel(id: impl Into<String>, views: Vec<String>) -> Self {
        LayoutNode::Panel(Panel::new(id, views))
    }

    pub fn split(id: impl Into<String>, axis: Axis, children: Vec<LayoutNode>) -> Self {
        LayoutNode::Split(Split::new(id, axis, children))
    }

    pub fn id(&self) -> &str {
        match self {
            LayoutNode::Desktop(d) => &d.id,
            LayoutNode::Panel(p) => &p.id,
            LayoutNode::Split(s) => &s.id,
        }
    }

    pub fn is_leaf(&self) -> bool {
        !matches!(self, LayoutNode::Split(_))
    }

    pub fn is_desktop(&self) -> bool {
        matches!(self, LayoutNode::Desktop(_))
    }

    pub fn children(&self) -> &[LayoutNode] {
        match self {
            LayoutNode::Split(s) => &s.children,
            _ => &[],
        }
    }

    /// Every node in pre-order, including self.
    pub fn flatten(&self) -> Vec<&LayoutNode> {
        let mut out = vec![self];
        for child in self.children() {
            out.extend(child.flatten());
        }
        out
    }

    pub fn node(&self, wanted: &str) -> Option<&LayoutNode> {
        self.flatten().into_iter().find(|n| n.id() == wanted)
    }

    /// The direct parent split of `child_id`, if any.
    pub fn parent(&self, child_id: &str) -> Option<&Split> {
        self.flatten().into_iter().find_map(|n| match n {
            LayoutNode::Split(s) if s.children.iter().any(|c| c.id() == child_id) => Some(s),
            _ => None,
        })
    }

    pub fn leaves(&self) -> Vec<&LayoutNode> {
        self.flatten().into_iter().filter(|n| n.is_leaf()).collect()
    }

    pub fn panel_containing(&self, seat_id: &str) -> Option<&Panel> {
        self.flatten().into_iter().find_map(|n| match n {
            LayoutNode::Panel(p) if p.views.iter().any(|v| v == seat_id) => Some(p),
            _ => None,
        })
    }

    /// Depth of the deepest split. 0 means a bare desktop.
    pub fn nesting_depth(&self) -> usize {
        match self {
            LayoutNode::Split(s) => {
                1 + s
                    .children
                    .iter()
                    .map(LayoutNode::nesting_depth)
                    .max()
                    .unwrap_or(0)
            }
            _ => 0,
        }
    }

    /// Re-normalize every split. Cheap repair for imported documents.
    pub fn normalized(&self) -> LayoutNode {
        match self {
            LayoutNode::Split(s) => {
                let kids: Vec<LayoutNode> = s.children.iter().map(|c| c.normalized()).collect();
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

    /// Structural invariants: one desktop leaf, children == weights, weights sum to 1.
    pub fn validate(&self) -> Result<(), LayoutError> {
        for n in self.flatten() {
            if let LayoutNode::Split(s) = n {
                if s.children.is_empty() {
                    return Err(LayoutError::Invariant(format!(
                        "split '{}' has no children",
                        s.id
                    )));
                }
                if s.children.len() != s.weights.len() {
                    return Err(LayoutError::Invariant(format!(
                        "split '{}': {} children vs {} weights",
                        s.id,
                        s.children.len(),
                        s.weights.len()
                    )));
                }
                if s.weights.iter().any(|w| *w < 0.0 || !w.is_finite()) {
                    return Err(LayoutError::Invariant(format!(
                        "split '{}' has a negative or non-finite weight",
                        s.id
                    )));
                }
                let sum: f64 = s.weights.iter().sum();
                if (sum - 1.0).abs() > 1e-6 {
                    return Err(LayoutError::Invariant(format!(
                        "split '{}': weights sum to {sum}, expected 1.0",
                        s.id
                    )));
                }
            }
        }
        let desktops = self.flatten().iter().filter(|n| n.is_desktop()).count();
        if desktops != 1 {
            return Err(LayoutError::Invariant(format!(
                "expected exactly one desktop leaf, found {desktops}"
            )));
        }
        Ok(())
    }
}

/// Clamp, pad and renormalize a weight vector to `count` entries summing to 1.
pub fn normalize_weights(weights: &[f64], count: usize) -> Vec<f64> {
    if count == 0 {
        return Vec::new();
    }
    let mut w: Vec<f64> = weights.iter().take(count).copied().collect();
    if w.len() < count {
        w.extend(std::iter::repeat(1.0 / count as f64).take(count - w.len()));
    }
    for v in w.iter_mut() {
        if !v.is_finite() || *v < 0.0 {
            *v = 0.0;
        }
    }
    let sum: f64 = w.iter().sum();
    if sum <= 0.0 {
        return vec![1.0 / count as f64; count];
    }
    w.iter().map(|v| v / sum).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LayoutError {
    #[error("invariant: {0}")]
    Invariant(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid command: {0}")]
    InvalidCommand(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LayoutNode {
        let a = LayoutNode::panel("p-a", vec!["s-a".into()]);
        let b = LayoutNode::panel("p-b", vec!["s-b".into()]);
        let inner = LayoutNode::split("split-inner", Axis::Y, vec![b, LayoutNode::desktop()]);
        LayoutNode::split("root", Axis::X, vec![a, inner])
    }

    #[test]
    fn traversal_and_depth() {
        let t = sample();
        assert_eq!(t.flatten().len(), 5);
        assert_eq!(t.nesting_depth(), 2);
        assert_eq!(t.leaves().len(), 3);
        assert_eq!(t.parent("p-b").unwrap().id, "split-inner");
        assert!(t.node("nope").is_none());
    }

    #[test]
    fn validate_accepts_and_rejects() {
        assert!(sample().validate().is_ok());
        let bad = LayoutNode::Split(Split {
            id: "s".into(),
            axis: Axis::X,
            children: vec![LayoutNode::desktop()],
            weights: vec![0.5],
        });
        assert!(bad.validate().is_err());
        let two_desktops = LayoutNode::split(
            "r",
            Axis::X,
            vec![LayoutNode::desktop(), LayoutNode::desktop()],
        );
        assert!(two_desktops.validate().is_err());
    }

    #[test]
    fn weights_normalize() {
        assert_eq!(normalize_weights(&[2.0, 2.0], 2), vec![0.5, 0.5]);
        assert_eq!(normalize_weights(&[], 2), vec![0.5, 0.5]);
        assert_eq!(normalize_weights(&[0.0, 0.0], 2), vec![0.5, 0.5]);
        let padded = normalize_weights(&[1.0], 2);
        assert!((padded[0] - 2.0 / 3.0).abs() < 1e-9);
        assert!((padded[1] - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn document_shape_round_trips() {
        let t = sample();
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"kind\":\"desktop\""));
        assert!(json.contains("\"kind\":\"split\""));
        let back: LayoutNode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn tab_group_visible_seat() {
        let p = Panel::tab_group("p", vec!["s1".into(), "s2".into()]);
        assert!(p.is_tab_group());
        assert_eq!(p.visible(), Some("s1"));
        let mut p2 = p.clone();
        p2.selected = Some("s2".into());
        assert_eq!(p2.visible(), Some("s2"));
        p2.selected = Some("gone".into());
        assert_eq!(p2.visible(), Some("s1"));
    }

    #[test]
    fn selecting_and_cycling_views() {
        let mut p = Panel::tab_group("p", vec!["s1".into(), "s2".into(), "s3".into()]);
        assert!(p.select("s2"));
        assert_eq!(p.visible(), Some("s2"));
        assert!(!p.select("nope"), "a panel cannot select a seat it does not hold");
        assert_eq!(p.visible(), Some("s2"));
        assert_eq!(p.cycle(true).as_deref(), Some("s3"));
        assert_eq!(p.cycle(true).as_deref(), Some("s1"), "cycling wraps forward");
        assert_eq!(p.cycle(false).as_deref(), Some("s3"), "cycling wraps backward");
        let mut single = Panel::new("q", vec!["s1".into()]);
        assert_eq!(single.cycle(true).as_deref(), Some("s1"));
    }

    #[test]
    fn removing_the_selected_view_promotes_its_neighbour() {
        let mut p = Panel::tab_group("p", vec!["s1".into(), "s2".into(), "s3".into()]);
        p.select("s2");
        assert!(p.remove_view("s2"), "views remain");
        assert_eq!(p.views, vec!["s1", "s3"]);
        assert_eq!(p.visible(), Some("s3"), "the view that took its place is shown");
        // Removing the last view of the group promotes the previous one.
        p.select("s3");
        assert!(p.remove_view("s3"));
        assert_eq!(p.visible(), Some("s1"));
        // The final view takes the panel with it.
        assert!(!p.remove_view("s1"));
        assert!(p.views.is_empty());
        assert_eq!(p.selected, None);
        // A group that drops to one view stops claiming to be a group.
        let mut group = Panel::tab_group("g", vec!["s1".into(), "s2".into()]);
        assert!(group.remove_view("s2"));
        assert!(!group.is_tab_group());
        assert_eq!(group.tabbed, None);
    }

    #[test]
    fn removing_an_unselected_view_keeps_the_selection() {
        let mut p = Panel::tab_group("p", vec!["s1".into(), "s2".into(), "s3".into()]);
        p.select("s2");
        assert!(p.remove_view("s1"));
        assert_eq!(p.visible(), Some("s2"));
    }
}
