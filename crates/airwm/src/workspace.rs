// crates/airwm/src/workspace.rs

use airproto::types::WindowId;

/// Virtuální plocha (workspace/desktop).
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: u32,
    pub name: String,
    window_ids: Vec<WindowId>,
}

impl Workspace {
    fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            window_ids: Vec::new(),
        }
    }

    pub fn window_ids(&self) -> &[WindowId] {
        &self.window_ids
    }

    pub fn contains_window(&self, id: WindowId) -> bool {
        self.window_ids.contains(&id)
    }
}

/// Správce virtuálních ploch.
pub struct WorkspaceManager {
    workspaces: Vec<Workspace>,
    active_idx: usize,
    next_id: u32,
}

impl WorkspaceManager {
    /// Vytvoří manažer s jednou výchozí plochou "Desktop 1".
    pub fn new() -> Self {
        Self {
            workspaces: vec![Workspace::new(0, "Desktop 1")],
            active_idx: 0,
            next_id: 1,
        }
    }

    /// Přidá novou plochu a vrátí její ID.
    pub fn add_workspace(&mut self, name: impl Into<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.workspaces.push(Workspace::new(id, name));
        id
    }

    /// Přepne na plochu dle ID. Vrátí false pokud plocha neexistuje.
    pub fn switch_to(&mut self, id: u32) -> bool {
        if let Some(idx) = self.workspaces.iter().position(|w| w.id == id) {
            self.active_idx = idx;
            true
        } else {
            false
        }
    }

    pub fn active(&self) -> &Workspace {
        &self.workspaces[self.active_idx]
    }

    /// Přiřadí okno na konkrétní plochu (odstraní ho z ostatních ploch).
    pub fn assign_window(&mut self, window_id: WindowId, workspace_id: u32) {
        for ws in &mut self.workspaces {
            ws.window_ids.retain(|&id| id != window_id);
        }
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == workspace_id) {
            ws.window_ids.push(window_id);
        }
    }

    /// Odstraní okno ze všech ploch (při zavření).
    pub fn remove_window(&mut self, window_id: WindowId) {
        for ws in &mut self.workspaces {
            ws.window_ids.retain(|&id| id != window_id);
        }
    }

    /// Vrátí ID plochy, na které okno je, nebo None.
    pub fn workspace_for_window(&self, window_id: WindowId) -> Option<u32> {
        self.workspaces
            .iter()
            .find(|w| w.contains_window(window_id))
            .map(|w| w.id)
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_one_workspace_named_desktop_1() {
        let wm = WorkspaceManager::new();
        assert_eq!(wm.workspaces().len(), 1);
        assert_eq!(wm.active().name, "Desktop 1");
        assert_eq!(wm.active().id, 0);
    }

    #[test]
    fn add_workspace_returns_incrementing_id() {
        let mut wm = WorkspaceManager::new();
        let id1 = wm.add_workspace("Desktop 2");
        let id2 = wm.add_workspace("Desktop 3");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(wm.workspaces().len(), 3);
    }

    #[test]
    fn switch_to_changes_active_workspace() {
        let mut wm = WorkspaceManager::new();
        let id = wm.add_workspace("Desktop 2");
        let switched = wm.switch_to(id);
        assert!(switched);
        assert_eq!(wm.active().id, id);
    }

    #[test]
    fn switch_to_nonexistent_returns_false() {
        let mut wm = WorkspaceManager::new();
        assert!(!wm.switch_to(99));
        assert_eq!(wm.active().id, 0);
    }

    #[test]
    fn assign_window_tracks_workspace() {
        let mut wm = WorkspaceManager::new();
        let ws2 = wm.add_workspace("Desktop 2");
        wm.assign_window(WindowId(1), ws2);
        assert_eq!(wm.workspace_for_window(WindowId(1)), Some(ws2));
        assert!(!wm.workspaces()[0].contains_window(WindowId(1)));
    }

    #[test]
    fn remove_window_unassigns_from_all_workspaces() {
        let mut wm = WorkspaceManager::new();
        wm.assign_window(WindowId(1), 0);
        wm.remove_window(WindowId(1));
        assert_eq!(wm.workspace_for_window(WindowId(1)), None);
    }
}
