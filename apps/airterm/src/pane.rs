use crate::grid::TermGrid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaneId(pub u64);

/// Jeden terminálový panel — obaluje TermGrid a má metadata.
#[derive(Debug)]
pub struct Pane {
    pub id: PaneId,
    pub grid: TermGrid,
    pub title: String,
}

impl Pane {
    pub fn new(id: PaneId, rows: usize, cols: usize) -> Self {
        Self {
            id,
            grid: TermGrid::new(rows, cols),
            title: String::new(),
        }
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Zapíše řetězec znak po znaku do gridu.
    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            if ch == '\n' {
                self.grid.newline();
            } else {
                self.grid.write_char(ch);
            }
        }
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.grid.resize(rows, cols);
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pane_has_correct_dimensions() {
        let p = Pane::new(PaneId(1), 24, 80);
        assert_eq!(p.grid.rows(), 24);
        assert_eq!(p.grid.cols(), 80);
    }

    #[test]
    fn new_pane_has_empty_title() {
        let p = Pane::new(PaneId(1), 24, 80);
        assert!(p.title.is_empty());
    }

    #[test]
    fn set_title_updates_title() {
        let mut p = Pane::new(PaneId(1), 24, 80);
        p.set_title("bash");
        assert_eq!(p.title, "bash");
    }

    #[test]
    fn write_str_stores_characters_in_grid() {
        let mut p = Pane::new(PaneId(1), 24, 80);
        p.write_str("Hi");
        assert_eq!(p.grid.cell_at(0, 0).unwrap().ch, 'H');
        assert_eq!(p.grid.cell_at(0, 1).unwrap().ch, 'i');
    }

    #[test]
    fn write_str_newline_moves_cursor() {
        let mut p = Pane::new(PaneId(1), 24, 80);
        p.write_str("A\nB");
        assert_eq!(p.grid.cell_at(0, 0).unwrap().ch, 'A');
        assert_eq!(p.grid.cell_at(1, 0).unwrap().ch, 'B');
    }

    #[test]
    fn resize_changes_grid_size() {
        let mut p = Pane::new(PaneId(1), 24, 80);
        p.resize(30, 120);
        assert_eq!(p.grid.rows(), 30);
        assert_eq!(p.grid.cols(), 120);
    }

    #[test]
    fn clear_resets_grid() {
        let mut p = Pane::new(PaneId(1), 5, 10);
        p.write_str("Hello");
        p.clear();
        assert_eq!(p.grid.cursor_row, 0);
        assert_eq!(p.grid.cursor_col, 0);
        assert_eq!(p.grid.cell_at(0, 0).unwrap().ch, ' ');
    }
}
