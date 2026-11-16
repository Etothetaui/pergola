//! A bounded history of states, so a wrong layout change can be undone.
//!
//! The daemon snapshots the layout before a mutation and restores the newest
//! snapshot on `pergola undo`. Keeping the ring here, with no X11 or clock,
//! makes the bounds and the de-duplication testable on their own.

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History<T> {
    states: VecDeque<T>,
    depth: usize,
}

impl<T: PartialEq> History<T> {
    /// A ring that remembers at most `depth` states. A depth of zero is treated
    /// as one: a history that remembers nothing is not a history.
    pub fn new(depth: usize) -> Self {
        Self {
            states: VecDeque::new(),
            depth: depth.max(1),
        }
    }

    /// Record `state`. Returns false when it is identical to the newest one, so
    /// a command that changed nothing cannot fill the ring with duplicates.
    pub fn push(&mut self, state: T) -> bool {
        if self.states.back() == Some(&state) {
            return false;
        }
        self.states.push_back(state);
        while self.states.len() > self.depth {
            self.states.pop_front();
        }
        true
    }

    /// Take the newest state back off the ring, for an undo.
    pub fn pop(&mut self) -> Option<T> {
        self.states.pop_back()
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub fn clear(&mut self) {
        self.states.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_last_states() {
        let mut history = History::new(3);
        for n in 1..=5 {
            assert!(history.push(n));
        }
        assert_eq!(history.len(), 3);
        assert_eq!(history.pop(), Some(5));
        assert_eq!(history.pop(), Some(4));
        assert_eq!(history.pop(), Some(3));
        assert_eq!(history.pop(), None);
    }

    #[test]
    fn a_repeated_state_is_not_recorded_twice() {
        let mut history = History::new(4);
        assert!(history.push(1));
        assert!(!history.push(1), "the same state must not fill the ring");
        assert_eq!(history.len(), 1);
        assert!(history.push(2));
        assert!(history.push(1), "A, B, A is a real history");
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn a_zero_depth_still_remembers_one() {
        let mut history = History::new(0);
        assert!(history.push("only"));
        assert_eq!(history.pop(), Some("only"));
        assert_eq!(history.pop(), None);
    }

    #[test]
    fn undo_walks_back_in_order_and_empties() {
        let mut history = History::new(10);
        history.push("a");
        history.push("b");
        assert!(!history.is_empty());
        assert_eq!(history.pop(), Some("b"));
        history.push("c");
        assert_eq!(history.pop(), Some("c"));
        assert_eq!(history.pop(), Some("a"));
        assert!(history.is_empty());
        history.clear();
        assert_eq!(history.len(), 0);
    }
}
