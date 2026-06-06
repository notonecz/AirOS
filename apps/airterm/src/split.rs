use crate::pane::PaneId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDir {
    Horizontal, // dělení na levý a pravý
    Vertical,   // dělení na horní a dolní
}

/// Binární strom layoutu panelů.
/// Leaf = jeden panel, Split = dva podstromy rozdělené daným směrem.
#[derive(Debug, Clone)]
pub enum PaneNode {
    Leaf(PaneId),
    Split(SplitDir, Box<PaneNode>, Box<PaneNode>),
}

impl PaneNode {
    /// Vrátí seznam všech PaneId v DFS pořadí (zleva doprava / shora dolů).
    pub fn leaves(&self) -> Vec<PaneId> {
        match self {
            PaneNode::Leaf(id) => vec![*id],
            PaneNode::Split(_, left, right) => {
                let mut ids = left.leaves();
                ids.extend(right.leaves());
                ids
            }
        }
    }

    /// Vrátí true pokud strom obsahuje daný PaneId.
    pub fn contains(&self, id: PaneId) -> bool {
        self.leaves().contains(&id)
    }

    /// Nahradí Leaf(target) za Split(dir, Leaf(target), Leaf(new_id)).
    /// Vrátí true pokud byl target nalezen a nahrazen.
    pub fn split_leaf(&mut self, target: PaneId, dir: SplitDir, new_id: PaneId) -> bool {
        match self {
            PaneNode::Leaf(id) if *id == target => {
                let old = PaneNode::Leaf(target);
                let new = PaneNode::Leaf(new_id);
                *self = PaneNode::Split(dir, Box::new(old), Box::new(new));
                true
            }
            PaneNode::Leaf(_) => false,
            PaneNode::Split(_, left, right) => {
                left.split_leaf(target, dir, new_id) || right.split_leaf(target, dir, new_id)
            }
        }
    }

    /// Odstraní Leaf(target) a nahradí jeho rodičovský Split sourozencem.
    /// Vrátí true pokud byl target odstraněn.
    pub fn remove_leaf(&mut self, target: PaneId) -> bool {
        match self {
            PaneNode::Leaf(_) => false,
            PaneNode::Split(_, left, right) => {
                // Pokud levé dítě je cíl, nahradit self pravým dítětem
                if matches!(left.as_ref(), PaneNode::Leaf(id) if *id == target) {
                    *self = *right.clone();
                    return true;
                }
                // Pokud pravé dítě je cíl, nahradit self levým dítětem
                if matches!(right.as_ref(), PaneNode::Leaf(id) if *id == target) {
                    *self = *left.clone();
                    return true;
                }
                // Rekurzivně hledat
                left.remove_leaf(target) || right.remove_leaf(target)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_leaves_returns_self() {
        let node = PaneNode::Leaf(PaneId(1));
        assert_eq!(node.leaves(), vec![PaneId(1)]);
    }

    #[test]
    fn split_leaves_returns_both_children() {
        let node = PaneNode::Split(
            SplitDir::Horizontal,
            Box::new(PaneNode::Leaf(PaneId(1))),
            Box::new(PaneNode::Leaf(PaneId(2))),
        );
        assert_eq!(node.leaves(), vec![PaneId(1), PaneId(2)]);
    }

    #[test]
    fn contains_finds_existing_id() {
        let node = PaneNode::Leaf(PaneId(5));
        assert!(node.contains(PaneId(5)));
        assert!(!node.contains(PaneId(6)));
    }

    #[test]
    fn split_leaf_replaces_target_with_split() {
        let mut node = PaneNode::Leaf(PaneId(1));
        let result = node.split_leaf(PaneId(1), SplitDir::Horizontal, PaneId(2));
        assert!(result);
        let leaves = node.leaves();
        assert_eq!(leaves.len(), 2);
        assert!(leaves.contains(&PaneId(1)));
        assert!(leaves.contains(&PaneId(2)));
    }

    #[test]
    fn split_leaf_returns_false_for_unknown_id() {
        let mut node = PaneNode::Leaf(PaneId(1));
        let result = node.split_leaf(PaneId(99), SplitDir::Vertical, PaneId(2));
        assert!(!result);
    }

    #[test]
    fn remove_leaf_removes_child_and_collapses_parent() {
        let mut node = PaneNode::Split(
            SplitDir::Horizontal,
            Box::new(PaneNode::Leaf(PaneId(1))),
            Box::new(PaneNode::Leaf(PaneId(2))),
        );
        let result = node.remove_leaf(PaneId(1));
        assert!(result);
        assert_eq!(node.leaves(), vec![PaneId(2)]);
    }

    #[test]
    fn remove_leaf_returns_false_on_single_leaf() {
        let mut node = PaneNode::Leaf(PaneId(1));
        assert!(!node.remove_leaf(PaneId(1)));
    }

    #[test]
    fn nested_split_leaves_returns_all_ids() {
        // Split(H, Split(V, Leaf(1), Leaf(2)), Leaf(3))
        let node = PaneNode::Split(
            SplitDir::Horizontal,
            Box::new(PaneNode::Split(
                SplitDir::Vertical,
                Box::new(PaneNode::Leaf(PaneId(1))),
                Box::new(PaneNode::Leaf(PaneId(2))),
            )),
            Box::new(PaneNode::Leaf(PaneId(3))),
        );
        let leaves = node.leaves();
        assert_eq!(leaves.len(), 3);
        assert_eq!(leaves, vec![PaneId(1), PaneId(2), PaneId(3)]);
    }
}
