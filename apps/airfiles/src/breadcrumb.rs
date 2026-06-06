use std::path::{Path, PathBuf};

/// Reprezentuje aktuální cestu jako seznam segmentů pro breadcrumb navigaci.
/// Segmenty jsou komponenty cesty (např. ["/", "Users", "vo", "Documents"]).
#[derive(Debug, Clone, PartialEq)]
pub struct BreadcrumbPath {
    segments: Vec<String>,
}

impl BreadcrumbPath {
    /// Parsuje absolutní cestu na segmenty.
    pub fn from_path(path: &Path) -> Self {
        let segments = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        Self { segments }
    }

    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Vrátí novou BreadcrumbPath oříznutou na index (včetně).
    /// Pokud je index >= počet segmentů, vrátí kopii celé cesty.
    pub fn navigate_to_index(&self, index: usize) -> Self {
        let end = (index + 1).min(self.segments.len());
        Self {
            segments: self.segments[..end].to_vec(),
        }
    }

    /// Přidá segment (název adresáře) na konec.
    pub fn push(&mut self, name: String) {
        self.segments.push(name);
    }

    /// Odstraní poslední segment. Vrátí Some(name) pokud byl odstraněn,
    /// None pokud jsme na rootu (nelze jít výše).
    pub fn pop(&mut self) -> Option<String> {
        if self.segments.len() > 1 {
            self.segments.pop()
        } else {
            None
        }
    }

    /// Počet segmentů (hloubka cesty).
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Rekonstruuje PathBuf ze segmentů.
    pub fn to_path_buf(&self) -> PathBuf {
        self.segments.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path_parses_segments() {
        let p = BreadcrumbPath::from_path(Path::new("/Users/vo/Documents"));
        let segs = p.segments();
        assert_eq!(segs.last().unwrap(), "Documents");
        assert!(segs.contains(&"Users".to_string()));
    }

    #[test]
    fn depth_matches_component_count() {
        let p = BreadcrumbPath::from_path(Path::new("/Users/vo"));
        // "/" + "Users" + "vo" = 3
        assert_eq!(p.depth(), 3);
    }

    #[test]
    fn navigate_to_index_truncates() {
        let p = BreadcrumbPath::from_path(Path::new("/Users/vo/Documents"));
        // index=1 → keep ["/", "Users"]
        let trimmed = p.navigate_to_index(1);
        assert_eq!(trimmed.depth(), 2);
        assert_eq!(trimmed.segments().last().unwrap(), "Users");
    }

    #[test]
    fn navigate_to_index_beyond_length_returns_full() {
        let p = BreadcrumbPath::from_path(Path::new("/Users/vo"));
        let full = p.navigate_to_index(99);
        assert_eq!(full.depth(), p.depth());
    }

    #[test]
    fn push_appends_segment() {
        let mut p = BreadcrumbPath::from_path(Path::new("/Users/vo"));
        p.push("Downloads".into());
        assert_eq!(p.depth(), 4);
        assert_eq!(p.segments().last().unwrap(), "Downloads");
    }

    #[test]
    fn pop_removes_last_segment() {
        let mut p = BreadcrumbPath::from_path(Path::new("/Users/vo"));
        let removed = p.pop();
        assert_eq!(removed, Some("vo".to_string()));
        assert_eq!(p.depth(), 2);
    }

    #[test]
    fn pop_on_root_returns_none() {
        let mut p = BreadcrumbPath::from_path(Path::new("/"));
        let removed = p.pop();
        assert_eq!(removed, None);
        assert_eq!(p.depth(), 1);
    }

    #[test]
    fn to_path_buf_reconstructs_path() {
        let original = Path::new("/Users/vo/Documents");
        let p = BreadcrumbPath::from_path(original);
        assert_eq!(p.to_path_buf(), original);
    }
}
