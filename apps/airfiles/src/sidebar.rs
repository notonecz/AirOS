use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarSection {
    Favorites,
    Devices,
    Tags,
}

#[derive(Debug, Clone)]
pub struct SidebarItem {
    pub label: String,
    pub path: Option<PathBuf>,
    pub section: SidebarSection,
}

#[derive(Debug)]
pub struct SidebarState {
    items: Vec<SidebarItem>,
    active: Option<usize>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self { items: Vec::new(), active: None }
    }

    /// Přidá položku do sekce Oblíbené.
    pub fn add_favorite(&mut self, path: PathBuf, label: String) {
        self.items.push(SidebarItem {
            label,
            path: Some(path),
            section: SidebarSection::Favorites,
        });
    }

    /// Přidá položku do sekce Zařízení.
    pub fn add_device(&mut self, path: PathBuf, label: String) {
        self.items.push(SidebarItem {
            label,
            path: Some(path),
            section: SidebarSection::Devices,
        });
    }

    /// Přidá tag (bez cesty).
    pub fn add_tag(&mut self, label: String) {
        self.items.push(SidebarItem {
            label,
            path: None,
            section: SidebarSection::Tags,
        });
    }

    /// Odstraní první Favorites položku se shodnou cestou.
    /// Pokud byla odstraněna aktivní položka, výběr se vymaže.
    /// Pokud byla odstraněna položka s nižším indexem než aktivní, aktivní index se sníží o 1.
    pub fn remove_favorite(&mut self, path: &Path) {
        let removed_idx = self.items.iter().position(|i| {
            i.section == SidebarSection::Favorites && i.path.as_deref() == Some(path)
        });
        self.items.retain(|i| {
            !(i.section == SidebarSection::Favorites && i.path.as_deref() == Some(path))
        });
        if let Some(removed) = removed_idx {
            match self.active {
                Some(active) if active == removed => self.active = None,
                Some(active) if active > removed => self.active = Some(active - 1),
                _ => {}
            }
        }
    }

    /// Nastaví aktivní položku dle indexu. No-op pokud index neexistuje.
    pub fn set_active(&mut self, index: usize) {
        if index < self.items.len() {
            self.active = Some(index);
        }
    }

    pub fn active_index(&self) -> Option<usize> {
        self.active
    }

    /// Vrátí cestu aktivní položky. None pokud nic není vybráno nebo položka nemá cestu (tag).
    pub fn active_path(&self) -> Option<&Path> {
        self.active
            .and_then(|i| self.items.get(i))
            .and_then(|item| item.path.as_deref())
    }

    /// Vrátí všechny položky v dané sekci.
    pub fn items_in_section(&self, section: SidebarSection) -> Vec<&SidebarItem> {
        self.items.iter().filter(|i| i.section == section).collect()
    }

    pub fn items(&self) -> &[SidebarItem] {
        &self.items
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for SidebarState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sidebar_is_empty() {
        let s = SidebarState::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn add_favorite_adds_item_in_favorites_section() {
        let mut s = SidebarState::new();
        s.add_favorite(PathBuf::from("/Users/vo"), "Home".into());
        assert_eq!(s.len(), 1);
        let favs = s.items_in_section(SidebarSection::Favorites);
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].label, "Home");
    }

    #[test]
    fn add_device_adds_item_in_devices_section() {
        let mut s = SidebarState::new();
        s.add_device(PathBuf::from("/Volumes/USB"), "USB Drive".into());
        let devs = s.items_in_section(SidebarSection::Devices);
        assert_eq!(devs.len(), 1);
        assert_eq!(devs[0].label, "USB Drive");
    }

    #[test]
    fn add_tag_adds_item_with_no_path() {
        let mut s = SidebarState::new();
        s.add_tag("Work".into());
        let tags = s.items_in_section(SidebarSection::Tags);
        assert_eq!(tags.len(), 1);
        assert!(tags[0].path.is_none());
    }

    #[test]
    fn items_in_section_filters_correctly() {
        let mut s = SidebarState::new();
        s.add_favorite(PathBuf::from("/Users/vo"), "Home".into());
        s.add_device(PathBuf::from("/Volumes/USB"), "USB".into());
        s.add_tag("Work".into());
        assert_eq!(s.items_in_section(SidebarSection::Favorites).len(), 1);
        assert_eq!(s.items_in_section(SidebarSection::Devices).len(), 1);
        assert_eq!(s.items_in_section(SidebarSection::Tags).len(), 1);
    }

    #[test]
    fn set_active_marks_correct_item() {
        let mut s = SidebarState::new();
        s.add_favorite(PathBuf::from("/Users/vo"), "Home".into());
        s.add_device(PathBuf::from("/Volumes/USB"), "USB".into());
        s.set_active(1);
        assert_eq!(s.active_index(), Some(1));
    }

    #[test]
    fn set_active_out_of_bounds_is_noop() {
        let mut s = SidebarState::new();
        s.add_favorite(PathBuf::from("/Users/vo"), "Home".into());
        s.set_active(99);
        assert_eq!(s.active_index(), None);
    }

    #[test]
    fn active_path_returns_path_of_active_item() {
        let mut s = SidebarState::new();
        s.add_favorite(PathBuf::from("/Users/vo/Downloads"), "Downloads".into());
        s.set_active(0);
        assert_eq!(s.active_path(), Some(Path::new("/Users/vo/Downloads")));
    }

    #[test]
    fn active_path_is_none_for_tag() {
        let mut s = SidebarState::new();
        s.add_tag("Work".into());
        s.set_active(0);
        assert_eq!(s.active_path(), None);
    }

    #[test]
    fn remove_favorite_removes_item_and_clears_active() {
        let mut s = SidebarState::new();
        s.add_favorite(PathBuf::from("/Users/vo"), "Home".into());
        s.set_active(0);
        s.remove_favorite(Path::new("/Users/vo"));
        assert!(s.is_empty());
        assert_eq!(s.active_index(), None);
    }

    #[test]
    fn remove_nonexistent_favorite_is_noop() {
        let mut s = SidebarState::new();
        s.add_favorite(PathBuf::from("/Users/vo"), "Home".into());
        s.remove_favorite(Path::new("/nonexistent"));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn remove_other_favorite_does_not_clear_active_on_different_item() {
        let mut s = SidebarState::new();
        s.add_favorite(PathBuf::from("/Users/vo"), "Home".into());
        s.add_favorite(PathBuf::from("/Users/vo/Downloads"), "Downloads".into());
        s.set_active(1); // Downloads is active
        s.remove_favorite(Path::new("/Users/vo")); // remove Home, not the active item
        // Active should still be valid (now index 0 = Downloads)
        assert!(s.active_index().is_some());
    }
}
