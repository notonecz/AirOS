// apps/airterm/src/tab.rs

use crate::pane::{Pane, PaneId};
use crate::split::{PaneNode, SplitDir};
use std::collections::HashMap;

/// Jeden terminálový tab obsahující panes v split layoutu.
#[derive(Debug)]
pub struct Tab {
    pub id: u32,
    pub title: String,
    layout: PaneNode,
    panes: HashMap<PaneId, Pane>,
    active_pane: PaneId,
    next_pane_id: u64,
}

impl Tab {
    /// Vytvoří nový tab s jedním panem.
    pub fn new(id: u32, rows: usize, cols: usize) -> Self {
        let pane_id = PaneId(1);
        let pane = Pane::new(pane_id, rows, cols);
        let mut panes = HashMap::new();
        panes.insert(pane_id, pane);
        Self {
            id,
            title: String::new(),
            layout: PaneNode::Leaf(pane_id),
            panes,
            active_pane: pane_id,
            next_pane_id: 2,
        }
    }

    pub fn active_pane(&self) -> &Pane {
        self.panes.get(&self.active_pane).unwrap()
    }

    pub fn active_pane_mut(&mut self) -> &mut Pane {
        self.panes.get_mut(&self.active_pane).unwrap()
    }

    pub fn active_pane_id(&self) -> PaneId {
        self.active_pane
    }

    /// Vrátí pane dle ID. None pokud neexistuje.
    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        self.panes.get(&id)
    }

    /// Vrátí seznam PaneId v pořadí layoutu.
    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.layout.leaves()
    }

    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }

    /// Rozdělí aktivní pane. Nový pane je přidán jako druhé dítě splitu.
    /// Nastaví aktivní pane na nový pane.
    /// rows a cols jsou rozměry nového pane.
    pub fn split_active(&mut self, dir: SplitDir, rows: usize, cols: usize) {
        let new_id = PaneId(self.next_pane_id);
        self.next_pane_id += 1;
        self.layout.split_leaf(self.active_pane, dir, new_id);
        let new_pane = Pane::new(new_id, rows, cols);
        self.panes.insert(new_id, new_pane);
        self.active_pane = new_id;
    }

    /// Zavře pane dle ID. Pokud je to aktivní pane, přepne na jiný (první dostupný).
    /// Pokud je to jediný pane, nic se nestane (tab musí mít alespoň jeden pane).
    pub fn close_pane(&mut self, id: PaneId) {
        if self.panes.len() <= 1 {
            return;
        }
        self.layout.remove_leaf(id);
        self.panes.remove(&id);
        if self.active_pane == id {
            self.active_pane = *self.panes.keys().next().unwrap();
        }
    }

    /// Nastaví aktivní pane. No-op pokud ID neexistuje.
    pub fn set_active_pane(&mut self, id: PaneId) {
        if self.panes.contains_key(&id) {
            self.active_pane = id;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tab() -> Tab {
        Tab::new(1, 24, 80)
    }

    #[test]
    fn new_tab_has_one_pane() {
        let t = make_tab();
        assert_eq!(t.pane_count(), 1);
    }

    #[test]
    fn new_tab_active_pane_is_first_pane() {
        let t = make_tab();
        assert_eq!(t.active_pane_id(), PaneId(1));
    }

    #[test]
    fn split_active_adds_second_pane() {
        let mut t = make_tab();
        t.split_active(SplitDir::Horizontal, 24, 40);
        assert_eq!(t.pane_count(), 2);
    }

    #[test]
    fn split_active_makes_new_pane_active() {
        let mut t = make_tab();
        t.split_active(SplitDir::Horizontal, 24, 40);
        // Active pane should be the newly created one (PaneId(2))
        assert_eq!(t.active_pane_id(), PaneId(2));
    }

    #[test]
    fn pane_ids_reflects_layout_order() {
        let mut t = make_tab();
        t.split_active(SplitDir::Horizontal, 24, 40);
        let ids = t.pane_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&PaneId(1)));
        assert!(ids.contains(&PaneId(2)));
    }

    #[test]
    fn close_pane_reduces_count() {
        let mut t = make_tab();
        t.split_active(SplitDir::Horizontal, 24, 40);
        let second = t.active_pane_id();
        t.close_pane(second);
        assert_eq!(t.pane_count(), 1);
    }

    #[test]
    fn close_active_pane_switches_to_another() {
        let mut t = make_tab();
        t.split_active(SplitDir::Horizontal, 24, 40);
        let second = t.active_pane_id();
        t.close_pane(second);
        assert_ne!(t.active_pane_id(), second);
        assert!(t.pane(t.active_pane_id()).is_some());
    }

    #[test]
    fn close_only_pane_is_noop() {
        let mut t = make_tab();
        let only = t.active_pane_id();
        t.close_pane(only);
        assert_eq!(t.pane_count(), 1);
    }

    #[test]
    fn set_active_pane_changes_active() {
        let mut t = make_tab();
        t.split_active(SplitDir::Vertical, 12, 80);
        let first = PaneId(1);
        t.set_active_pane(first);
        assert_eq!(t.active_pane_id(), first);
    }

    #[test]
    fn set_active_pane_unknown_id_is_noop() {
        let mut t = make_tab();
        t.set_active_pane(PaneId(99));
        assert_eq!(t.active_pane_id(), PaneId(1));
    }
}
