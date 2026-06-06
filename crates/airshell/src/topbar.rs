// crates/airshell/src/topbar.rs

use crate::dock::DockState;
use crate::notifications::NotificationQueue;
use airbus::messages::MenuItem;
use std::collections::HashMap;

/// Stav top baru: registrovaná menu aplikací, aktivní aplikace.
#[derive(Debug, Default)]
pub struct TopBarState {
    registered_menus: HashMap<String, Vec<MenuItem>>,
    active_app_id: Option<String>,
}

impl TopBarState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Zaregistruje menu pro danou aplikaci.
    pub fn register_menu(&mut self, app_id: String, items: Vec<MenuItem>) {
        self.registered_menus.insert(app_id, items);
    }

    /// Odstraní menu aplikace (při ukončení).
    pub fn unregister_menu(&mut self, app_id: &str) {
        self.registered_menus.remove(app_id);
    }

    /// Nastaví aktivní aplikaci (focus).
    pub fn set_active_app(&mut self, app_id: Option<String>) {
        self.active_app_id = app_id;
    }

    /// Vrátí menu položky pro aktivní aplikaci.
    pub fn active_menu(&self) -> Option<&[MenuItem]> {
        self.active_app_id
            .as_ref()
            .and_then(|id| self.registered_menus.get(id))
            .map(Vec::as_slice)
    }
}

/// Kompletní stav AirShell: dock + notifikace + top bar.
pub struct ShellState {
    pub dock: DockState,
    pub notifications: NotificationQueue,
    pub topbar: TopBarState,
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            dock: DockState::new(),
            notifications: NotificationQueue::new(20),
            topbar: TopBarState::new(),
        }
    }
}

impl Default for ShellState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_menu() -> Vec<MenuItem> {
        vec![
            MenuItem {
                id: "new".into(),
                label: "New".into(),
                enabled: true,
            },
            MenuItem {
                id: "quit".into(),
                label: "Quit".into(),
                enabled: true,
            },
        ]
    }

    #[test]
    fn register_menu_stores_items() {
        let mut t = TopBarState::new();
        t.register_menu("finder".into(), make_menu());
        assert!(t.registered_menus.contains_key("finder"));
    }

    #[test]
    fn active_menu_none_when_no_active_app() {
        let t = TopBarState::new();
        assert!(t.active_menu().is_none());
    }

    #[test]
    fn active_menu_returns_items_for_active_app() {
        let mut t = TopBarState::new();
        t.register_menu("finder".into(), make_menu());
        t.set_active_app(Some("finder".into()));
        let menu = t.active_menu().unwrap();
        assert_eq!(menu.len(), 2);
        assert_eq!(menu[0].id, "new");
    }

    #[test]
    fn unregister_menu_removes_it() {
        let mut t = TopBarState::new();
        t.register_menu("finder".into(), make_menu());
        t.unregister_menu("finder");
        assert!(!t.registered_menus.contains_key("finder"));
    }

    #[test]
    fn shell_state_new_dock_and_queue_are_empty() {
        let s = ShellState::new();
        assert_eq!(s.dock.items().len(), 0);
        assert!(s.notifications.is_empty());
    }
}
