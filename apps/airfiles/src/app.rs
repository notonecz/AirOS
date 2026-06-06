// apps/airfiles/src/app.rs

use crate::panel::PanelState;
use crate::sidebar::SidebarState;
use airbus::messages::MenuItem;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Left,
    Right,
}

/// Kompletní stav AirFiles: dva panely, sidebar, aktivní panel.
#[derive(Debug)]
pub struct AirFilesApp {
    pub left: PanelState,
    pub right: PanelState,
    pub sidebar: SidebarState,
    pub active_panel: ActivePanel,
    pub dual_panel: bool,
}

impl AirFilesApp {
    /// Vytvoří nový stav se dvěma panely otevřenými na `home`.
    pub fn new(home: PathBuf) -> Self {
        Self {
            left: PanelState::new(&home),
            right: PanelState::new(&home),
            sidebar: SidebarState::new(),
            active_panel: ActivePanel::Left,
            dual_panel: false,
        }
    }

    /// Přepne mezi single-panel a dual-panel módem.
    pub fn toggle_dual_panel(&mut self) {
        self.dual_panel = !self.dual_panel;
    }

    /// Vrátí referenci na aktivní panel.
    pub fn active_panel(&self) -> &PanelState {
        match self.active_panel {
            ActivePanel::Left => &self.left,
            ActivePanel::Right => &self.right,
        }
    }

    /// Vrátí mutabilní referenci na aktivní panel.
    pub fn active_panel_mut(&mut self) -> &mut PanelState {
        match self.active_panel {
            ActivePanel::Left => &mut self.left,
            ActivePanel::Right => &mut self.right,
        }
    }

    /// Přepne aktivní panel (Left ↔ Right). Relevantní v dual-panel módu.
    pub fn switch_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Left => ActivePanel::Right,
            ActivePanel::Right => ActivePanel::Left,
        };
    }

    /// Přejde do adresáře z kliknutí v sidebaru.
    /// Naviguje aktivní panel na danou cestu (reset breadcrumb).
    pub fn navigate_sidebar_path(&mut self, path: &Path) {
        let panel = self.active_panel_mut();
        *panel = PanelState::new(path);
    }

    /// Vrátí menu položky pro registraci přes AirBus.
    pub fn menu_items() -> Vec<MenuItem> {
        vec![
            MenuItem {
                id: "new_folder".into(),
                label: "New Folder".into(),
                enabled: true,
            },
            MenuItem {
                id: "copy".into(),
                label: "Copy".into(),
                enabled: true,
            },
            MenuItem {
                id: "paste".into(),
                label: "Paste".into(),
                enabled: true,
            },
            MenuItem {
                id: "delete".into(),
                label: "Delete".into(),
                enabled: true,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::{EntryMeta, FileKind, FsEntry};

    fn home() -> PathBuf {
        PathBuf::from("/Users/vo")
    }

    fn make_app() -> AirFilesApp {
        AirFilesApp::new(home())
    }

    fn file(name: &str) -> FsEntry {
        FsEntry::new(
            name,
            PathBuf::from(name),
            FileKind::File,
            EntryMeta::file(1),
        )
    }

    #[test]
    fn new_app_starts_in_single_panel_mode() {
        let app = make_app();
        assert!(!app.dual_panel);
    }

    #[test]
    fn new_app_active_panel_is_left() {
        let app = make_app();
        assert_eq!(app.active_panel, ActivePanel::Left);
    }

    #[test]
    fn toggle_dual_panel_enables_dual_mode() {
        let mut app = make_app();
        app.toggle_dual_panel();
        assert!(app.dual_panel);
    }

    #[test]
    fn toggle_dual_panel_twice_returns_to_single() {
        let mut app = make_app();
        app.toggle_dual_panel();
        app.toggle_dual_panel();
        assert!(!app.dual_panel);
    }

    #[test]
    fn switch_panel_changes_active_to_right() {
        let mut app = make_app();
        app.switch_panel();
        assert_eq!(app.active_panel, ActivePanel::Right);
    }

    #[test]
    fn switch_panel_twice_returns_to_left() {
        let mut app = make_app();
        app.switch_panel();
        app.switch_panel();
        assert_eq!(app.active_panel, ActivePanel::Left);
    }

    #[test]
    fn active_panel_mut_modifies_left_by_default() {
        let mut app = make_app();
        app.active_panel_mut()
            .load_entries(vec![file("readme.txt")]);
        assert_eq!(app.left.entries().len(), 1);
        assert!(app.right.entries().is_empty());
    }

    #[test]
    fn active_panel_mut_modifies_right_after_switch() {
        let mut app = make_app();
        app.switch_panel();
        app.active_panel_mut()
            .load_entries(vec![file("readme.txt")]);
        assert!(app.left.entries().is_empty());
        assert_eq!(app.right.entries().len(), 1);
    }

    #[test]
    fn navigate_sidebar_path_resets_active_panel() {
        let mut app = make_app();
        app.active_panel_mut().navigate_into("Documents".into());
        app.navigate_sidebar_path(Path::new("/Volumes/USB"));
        assert_eq!(
            app.active_panel().path.to_path_buf(),
            PathBuf::from("/Volumes/USB")
        );
    }

    #[test]
    fn menu_items_returns_four_items() {
        let items = AirFilesApp::menu_items();
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn menu_items_contains_new_folder() {
        let items = AirFilesApp::menu_items();
        assert!(items.iter().any(|i| i.id == "new_folder"));
    }
}
