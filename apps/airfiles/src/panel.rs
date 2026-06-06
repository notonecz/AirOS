use crate::breadcrumb::BreadcrumbPath;
use crate::fs::FsEntry;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    NameAsc,
    NameDesc,
    SizeAsc,
    SizeDesc,
}

#[derive(Debug)]
pub struct PanelState {
    pub path: BreadcrumbPath,
    entries: Vec<FsEntry>,
    selected: Option<usize>,
    pub sort: SortOrder,
}

impl PanelState {
    pub fn new(path: &Path) -> Self {
        Self {
            path: BreadcrumbPath::from_path(path),
            entries: Vec::new(),
            selected: None,
            sort: SortOrder::NameAsc,
        }
    }

    /// Nahradí entries novým seznamem a aplikuje aktuální řazení.
    /// Výběr se vymaže.
    pub fn load_entries(&mut self, mut entries: Vec<FsEntry>) {
        Self::sort_entries(&mut entries, self.sort);
        self.entries = entries;
        self.selected = None;
    }

    /// Změní řazení a přeřadí existující entries. Výběr se vymaže.
    pub fn set_sort(&mut self, order: SortOrder) {
        self.sort = order;
        Self::sort_entries(&mut self.entries, order);
        self.selected = None;
    }

    /// Vybere entry na daném indexu. No-op pokud index neexistuje.
    pub fn select(&mut self, index: usize) {
        if index < self.entries.len() {
            self.selected = Some(index);
        }
    }

    /// Zruší výběr.
    pub fn select_none(&mut self) {
        self.selected = None;
    }

    pub fn selected_entry(&self) -> Option<&FsEntry> {
        self.selected.and_then(|i| self.entries.get(i))
    }

    pub fn entries(&self) -> &[FsEntry] {
        &self.entries
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Přejde do podadresáře. Přidá `name` do breadcrumb, vymaže entries a výběr.
    pub fn navigate_into(&mut self, name: String) {
        self.path.push(name);
        self.entries.clear();
        self.selected = None;
    }

    /// Přejde o úroveň výš. Pokud jsme na rootu, nic se nestane.
    /// Vymaže entries a výběr.
    pub fn navigate_up(&mut self) {
        self.path.pop();
        self.entries.clear();
        self.selected = None;
    }

    fn sort_entries(entries: &mut [FsEntry], order: SortOrder) {
        match order {
            SortOrder::NameAsc => entries.sort_by(|a, b| a.name.cmp(&b.name)),
            SortOrder::NameDesc => entries.sort_by(|a, b| b.name.cmp(&a.name)),
            SortOrder::SizeAsc => entries.sort_by(|a, b| {
                a.meta.size_bytes.unwrap_or(0).cmp(&b.meta.size_bytes.unwrap_or(0))
            }),
            SortOrder::SizeDesc => entries.sort_by(|a, b| {
                b.meta.size_bytes.unwrap_or(0).cmp(&a.meta.size_bytes.unwrap_or(0))
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::{EntryMeta, FileKind};
    use std::path::{Path, PathBuf};

    fn file(name: &str, size: u64) -> FsEntry {
        FsEntry::new(name, PathBuf::from(name), FileKind::File, EntryMeta::file(size))
    }

    fn make_panel() -> PanelState {
        PanelState::new(Path::new("/Users/vo"))
    }

    #[test]
    fn new_panel_has_empty_entries_and_no_selection() {
        let p = make_panel();
        assert!(p.entries().is_empty());
        assert!(p.selected_entry().is_none());
    }

    #[test]
    fn load_entries_stores_and_sorts_name_asc() {
        let mut p = make_panel();
        p.load_entries(vec![file("zebra.txt", 1), file("apple.txt", 2)]);
        assert_eq!(p.entries()[0].name, "apple.txt");
        assert_eq!(p.entries()[1].name, "zebra.txt");
    }

    #[test]
    fn load_entries_clears_selection() {
        let mut p = make_panel();
        p.load_entries(vec![file("a.txt", 1)]);
        p.select(0);
        p.load_entries(vec![file("b.txt", 2)]);
        assert!(p.selected_entry().is_none());
    }

    #[test]
    fn select_valid_index_sets_selection() {
        let mut p = make_panel();
        p.load_entries(vec![file("a.txt", 1), file("b.txt", 2)]);
        p.select(1);
        assert_eq!(p.selected_entry().unwrap().name, "b.txt");
    }

    #[test]
    fn select_out_of_bounds_is_noop() {
        let mut p = make_panel();
        p.load_entries(vec![file("a.txt", 1)]);
        p.select(99);
        assert!(p.selected_entry().is_none());
    }

    #[test]
    fn select_none_clears_selection() {
        let mut p = make_panel();
        p.load_entries(vec![file("a.txt", 1)]);
        p.select(0);
        p.select_none();
        assert!(p.selected_entry().is_none());
    }

    #[test]
    fn set_sort_name_desc_reverses_order() {
        let mut p = make_panel();
        p.load_entries(vec![file("apple.txt", 1), file("zebra.txt", 2)]);
        p.set_sort(SortOrder::NameDesc);
        assert_eq!(p.entries()[0].name, "zebra.txt");
        assert_eq!(p.entries()[1].name, "apple.txt");
    }

    #[test]
    fn set_sort_size_asc_sorts_by_size() {
        let mut p = make_panel();
        p.load_entries(vec![file("big.bin", 9000), file("small.txt", 10)]);
        p.set_sort(SortOrder::SizeAsc);
        assert_eq!(p.entries()[0].name, "small.txt");
        assert_eq!(p.entries()[1].name, "big.bin");
    }

    #[test]
    fn set_sort_clears_selection() {
        let mut p = make_panel();
        p.load_entries(vec![file("a.txt", 1)]);
        p.select(0);
        p.set_sort(SortOrder::NameDesc);
        assert!(p.selected_entry().is_none());
    }

    #[test]
    fn navigate_into_pushes_breadcrumb_and_clears_entries() {
        let mut p = make_panel();
        p.load_entries(vec![file("a.txt", 1)]);
        p.select(0);
        p.navigate_into("Documents".into());
        assert!(p.entries().is_empty());
        assert!(p.selected_entry().is_none());
        assert_eq!(p.path.segments().last().unwrap(), "Documents");
    }

    #[test]
    fn navigate_up_pops_breadcrumb_and_clears_entries() {
        let mut p = make_panel();
        p.navigate_into("Downloads".into());
        p.load_entries(vec![file("f.zip", 1000)]);
        p.navigate_up();
        assert!(p.entries().is_empty());
        assert_eq!(p.path.segments().last().unwrap(), "vo");
    }

    #[test]
    fn navigate_up_at_root_does_not_panic() {
        let mut p = PanelState::new(Path::new("/"));
        p.navigate_up(); // should be no-op
        assert_eq!(p.path.depth(), 1);
    }
}
