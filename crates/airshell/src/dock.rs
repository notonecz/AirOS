// crates/airshell/src/dock.rs

#[derive(Debug, Clone, PartialEq)]
pub struct DockItem {
    pub id: String,
    pub label: String,
    pub is_running: bool,
    pub is_pinned: bool,
    pub badge: Option<u32>,
}

impl DockItem {
    fn new(id: impl Into<String>, label: impl Into<String>, pinned: bool, running: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            is_running: running,
            is_pinned: pinned,
            badge: None,
        }
    }
}

/// Stav docku: pinned aplikace + spuštěné aplikace.
#[derive(Debug, Default)]
pub struct DockState {
    items: Vec<DockItem>,
}

impl DockState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Přidá/označí položku jako pinned.
    pub fn pin(&mut self, id: impl Into<String>, label: impl Into<String>) {
        let id = id.into();
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.is_pinned = true;
        } else {
            self.items.push(DockItem::new(id, label, true, false));
        }
    }

    /// Odstraní pin. Pokud aplikace neběží, odstraní ji z docku úplně.
    pub fn unpin(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.is_pinned = false;
            if !item.is_running {
                self.items.retain(|i| i.id != id);
            }
        }
    }

    /// Nastaví running stav aplikace. Pokud running=true a položka neexistuje, přidá ji.
    /// Pokud running=false a položka není pinned, odstraní ji.
    pub fn set_running(&mut self, id: &str, label: &str, running: bool) {
        if running {
            if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                item.is_running = true;
            } else {
                self.items.push(DockItem::new(id, label, false, true));
            }
        } else if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.is_running = false;
            if !item.is_pinned {
                self.items.retain(|i| i.id != id);
            }
        }
    }

    /// Nastaví badge count. count=0 odstraní badge. No-op pokud item neexistuje.
    pub fn set_badge(&mut self, id: &str, count: u32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.badge = if count > 0 { Some(count) } else { None };
        }
    }

    pub fn get(&self, id: &str) -> Option<&DockItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn items(&self) -> &[DockItem] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_adds_item_to_dock() {
        let mut d = DockState::new();
        d.pin("finder", "Finder");
        let item = d.get("finder").unwrap();
        assert!(item.is_pinned);
        assert!(!item.is_running);
        assert_eq!(item.label, "Finder");
    }

    #[test]
    fn pin_existing_item_marks_it_pinned() {
        let mut d = DockState::new();
        d.set_running("finder", "Finder", true);
        d.pin("finder", "Finder");
        let item = d.get("finder").unwrap();
        assert!(item.is_pinned);
        assert!(item.is_running);
    }

    #[test]
    fn set_running_creates_unpinned_item() {
        let mut d = DockState::new();
        d.set_running("term", "Terminal", true);
        let item = d.get("term").unwrap();
        assert!(item.is_running);
        assert!(!item.is_pinned);
    }

    #[test]
    fn set_running_false_removes_unpinned_item() {
        let mut d = DockState::new();
        d.set_running("term", "Terminal", true);
        d.set_running("term", "Terminal", false);
        assert!(d.get("term").is_none());
    }

    #[test]
    fn set_running_false_keeps_pinned_item() {
        let mut d = DockState::new();
        d.pin("finder", "Finder");
        d.set_running("finder", "Finder", true);
        d.set_running("finder", "Finder", false);
        let item = d.get("finder").unwrap();
        assert!(item.is_pinned);
        assert!(!item.is_running);
    }

    #[test]
    fn set_badge_updates_badge_count() {
        let mut d = DockState::new();
        d.set_running("mail", "Mail", true);
        d.set_badge("mail", 5);
        assert_eq!(d.get("mail").unwrap().badge, Some(5));
    }

    #[test]
    fn set_badge_zero_clears_badge() {
        let mut d = DockState::new();
        d.set_running("mail", "Mail", true);
        d.set_badge("mail", 3);
        d.set_badge("mail", 0);
        assert_eq!(d.get("mail").unwrap().badge, None);
    }
}
