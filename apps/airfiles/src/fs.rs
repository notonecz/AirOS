use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryMeta {
    /// Velikost v bajtech. None pro adresáře.
    pub size_bytes: Option<u64>,
    /// Unix timestamp poslední změny. None pokud není k dispozici.
    pub modified_secs: Option<u64>,
}

impl EntryMeta {
    pub fn file(size: u64) -> Self {
        Self { size_bytes: Some(size), modified_secs: None }
    }

    pub fn dir() -> Self {
        Self { size_bytes: None, modified_secs: None }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: FileKind,
    pub meta: EntryMeta,
}

impl FsEntry {
    pub fn new(name: impl Into<String>, path: PathBuf, kind: FileKind, meta: EntryMeta) -> Self {
        Self { name: name.into(), path, kind, meta }
    }

    pub fn is_dir(&self) -> bool {
        self.kind == FileKind::Directory
    }

    pub fn is_file(&self) -> bool {
        self.kind == FileKind::File
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, size: u64) -> FsEntry {
        FsEntry::new(name, PathBuf::from(name), FileKind::File, EntryMeta::file(size))
    }

    fn dir(name: &str) -> FsEntry {
        FsEntry::new(name, PathBuf::from(name), FileKind::Directory, EntryMeta::dir())
    }

    #[test]
    fn file_is_not_dir() {
        let e = file("readme.txt", 1024);
        assert!(!e.is_dir());
        assert!(e.is_file());
    }

    #[test]
    fn dir_is_dir() {
        let e = dir("Documents");
        assert!(e.is_dir());
        assert!(!e.is_file());
    }

    #[test]
    fn symlink_is_neither_file_nor_dir() {
        let e = FsEntry::new(
            "link",
            PathBuf::from("link"),
            FileKind::Symlink,
            EntryMeta::file(0),
        );
        assert!(!e.is_dir());
        assert!(!e.is_file());
    }

    #[test]
    fn file_has_size() {
        let e = file("data.bin", 4096);
        assert_eq!(e.meta.size_bytes, Some(4096));
    }

    #[test]
    fn dir_has_no_size() {
        let e = dir("Downloads");
        assert_eq!(e.meta.size_bytes, None);
    }

    #[test]
    fn entry_name_stored_correctly() {
        let e = file("my_file.rs", 42);
        assert_eq!(e.name, "my_file.rs");
    }
}
