// apps/airterm/src/app.rs

use crate::tab::Tab;
use airbus::messages::MenuItem;

/// Konfigurace shellu.
#[derive(Debug, Clone)]
pub struct ShellConfig {
    pub shell: String,
    pub args: Vec<String>,
}

impl ShellConfig {
    /// Výchozí konfigurace: zsh bez argumentů.
    pub fn default_zsh() -> Self {
        Self {
            shell: "zsh".into(),
            args: Vec::new(),
        }
    }
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self::default_zsh()
    }
}

/// Kompletní stav AirTerm: taby + konfigurace shellu.
#[derive(Debug)]
pub struct AirTermApp {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_tab_id: u32,
    pub config: ShellConfig,
    default_rows: usize,
    default_cols: usize,
}

impl AirTermApp {
    pub fn new(rows: usize, cols: usize) -> Self {
        let first_tab = Tab::new(1, rows, cols);
        Self {
            tabs: vec![first_tab],
            active_tab: 0,
            next_tab_id: 2,
            config: ShellConfig::default_zsh(),
            default_rows: rows,
            default_cols: cols,
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Přidá nový tab a přepne na něj. Vrátí ID nového tabu.
    pub fn add_tab(&mut self) -> u32 {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        self.tabs.push(Tab::new(id, self.default_rows, self.default_cols));
        self.active_tab = self.tabs.len() - 1;
        id
    }

    /// Zavře tab dle ID. Pokud je to aktivní tab, přepne na sousední.
    /// Pokud je to jediný tab, nic se nestane.
    pub fn close_tab(&mut self, id: u32) {
        if self.tabs.len() <= 1 {
            return;
        }
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(idx);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    /// Přepne na tab dle ID. No-op pokud tab neexistuje.
    pub fn switch_to_tab(&mut self, id: u32) {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.active_tab = idx;
        }
    }

    /// Vrátí ID aktivního tabu.
    pub fn active_tab_id(&self) -> u32 {
        self.tabs[self.active_tab].id
    }

    /// Vrátí menu položky pro registraci přes AirBus.
    pub fn menu_items() -> Vec<MenuItem> {
        vec![
            MenuItem { id: "new_tab".into(), label: "New Tab".into(), enabled: true },
            MenuItem { id: "close_tab".into(), label: "Close Tab".into(), enabled: true },
            MenuItem { id: "split_horizontal".into(), label: "Split Horizontal".into(), enabled: true },
            MenuItem { id: "split_vertical".into(), label: "Split Vertical".into(), enabled: true },
            MenuItem { id: "clear".into(), label: "Clear".into(), enabled: true },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> AirTermApp {
        AirTermApp::new(24, 80)
    }

    #[test]
    fn new_app_has_one_tab() {
        let app = make_app();
        assert_eq!(app.tab_count(), 1);
    }

    #[test]
    fn new_app_active_tab_id_is_1() {
        let app = make_app();
        assert_eq!(app.active_tab_id(), 1);
    }

    #[test]
    fn add_tab_increases_count() {
        let mut app = make_app();
        app.add_tab();
        assert_eq!(app.tab_count(), 2);
    }

    #[test]
    fn add_tab_switches_to_new_tab() {
        let mut app = make_app();
        let new_id = app.add_tab();
        assert_eq!(app.active_tab_id(), new_id);
    }

    #[test]
    fn close_tab_removes_it() {
        let mut app = make_app();
        let new_id = app.add_tab();
        app.close_tab(new_id);
        assert_eq!(app.tab_count(), 1);
    }

    #[test]
    fn close_only_tab_is_noop() {
        let mut app = make_app();
        let id = app.active_tab_id();
        app.close_tab(id);
        assert_eq!(app.tab_count(), 1);
    }

    #[test]
    fn switch_to_tab_changes_active() {
        let mut app = make_app();
        let first_id = app.active_tab_id();
        app.add_tab();
        app.switch_to_tab(first_id);
        assert_eq!(app.active_tab_id(), first_id);
    }

    #[test]
    fn switch_to_nonexistent_tab_is_noop() {
        let mut app = make_app();
        app.switch_to_tab(99);
        assert_eq!(app.active_tab_id(), 1);
    }

    #[test]
    fn default_shell_is_zsh() {
        let app = make_app();
        assert_eq!(app.config.shell, "zsh");
    }

    #[test]
    fn menu_items_returns_five_items() {
        let items = AirTermApp::menu_items();
        assert_eq!(items.len(), 5);
    }

    #[test]
    fn menu_items_contains_new_tab() {
        let items = AirTermApp::menu_items();
        assert!(items.iter().any(|i| i.id == "new_tab"));
    }
}
