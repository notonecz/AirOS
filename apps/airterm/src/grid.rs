use crate::cell::Cell;

/// 2D buffer terminálových buněk s kurzorem.
/// Řádky jsou indexovány od 0 (nahoře), sloupce od 0 (vlevo).
#[derive(Debug, Clone)]
pub struct TermGrid {
    rows: usize,
    cols: usize,
    cells: Vec<Cell>,       // row-major: cells[row * cols + col]
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl TermGrid {
    /// Vytvoří grid vyplněný prázdnými buňkami.
    pub fn new(rows: usize, cols: usize) -> Self {
        assert!(rows > 0 && cols > 0, "grid dimensions must be positive");
        Self {
            rows,
            cols,
            cells: vec![Cell::blank(); rows * cols],
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Vrátí referenci na buňku. Vrátí None pokud jsou souřadnice mimo grid.
    pub fn cell_at(&self, row: usize, col: usize) -> Option<&Cell> {
        if row < self.rows && col < self.cols {
            Some(&self.cells[row * self.cols + col])
        } else {
            None
        }
    }

    /// Zapíše znak na pozici kurzoru a posune kurzor doprava.
    /// Na konci řádku automaticky přejde na nový řádek (wrap).
    pub fn write_char(&mut self, ch: char) {
        if self.cursor_row >= self.rows {
            return;
        }
        self.cells[self.cursor_row * self.cols + self.cursor_col] = Cell::new(ch);
        self.cursor_col += 1;
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
            // Pokud jsme za posledním řádkem, scrolluj.
            if self.cursor_row >= self.rows {
                self.scroll_up();
                self.cursor_row = self.rows - 1;
            }
        }
    }

    /// Posune kurzor na začátek nového řádku. Pokud je na posledním řádku, scrolluje.
    pub fn newline(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row + 1 < self.rows {
            self.cursor_row += 1;
        } else {
            self.scroll_up();
        }
    }

    /// Posune obsah gridu o jeden řádek nahoru. Nejnižší řádek se vyplní prázdnými buňkami.
    pub fn scroll_up(&mut self) {
        self.cells.copy_within(self.cols.., 0);
        let last_row_start = (self.rows - 1) * self.cols;
        for cell in &mut self.cells[last_row_start..] {
            *cell = Cell::blank();
        }
    }

    /// Vymaže celý grid a přesune kurzor na [0, 0].
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::blank();
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Změní velikost gridu. Obsah se ztratí (reset na prázdno).
    pub fn resize(&mut self, rows: usize, cols: usize) {
        assert!(rows > 0 && cols > 0, "grid dimensions must be positive");
        self.rows = rows;
        self.cols = cols;
        self.cells = vec![Cell::blank(); rows * cols];
        self.cursor_row = self.cursor_row.min(rows - 1);
        self.cursor_col = self.cursor_col.min(cols - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_is_filled_with_blanks() {
        let g = TermGrid::new(3, 10);
        for row in 0..3 {
            for col in 0..10 {
                assert_eq!(g.cell_at(row, col).unwrap().ch, ' ');
            }
        }
    }

    #[test]
    fn cell_at_out_of_bounds_returns_none() {
        let g = TermGrid::new(2, 5);
        assert!(g.cell_at(2, 0).is_none());
        assert!(g.cell_at(0, 5).is_none());
    }

    #[test]
    fn write_char_stores_char_and_advances_cursor() {
        let mut g = TermGrid::new(5, 10);
        g.write_char('A');
        assert_eq!(g.cell_at(0, 0).unwrap().ch, 'A');
        assert_eq!(g.cursor_col, 1);
        assert_eq!(g.cursor_row, 0);
    }

    #[test]
    fn write_char_wraps_at_end_of_row() {
        let mut g = TermGrid::new(5, 3);
        g.write_char('A');
        g.write_char('B');
        g.write_char('C'); // fills col 2, then wraps
        assert_eq!(g.cursor_col, 0);
        assert_eq!(g.cursor_row, 1);
    }

    #[test]
    fn newline_moves_cursor_to_next_row() {
        let mut g = TermGrid::new(5, 10);
        g.write_char('X');
        g.newline();
        assert_eq!(g.cursor_row, 1);
        assert_eq!(g.cursor_col, 0);
    }

    #[test]
    fn scroll_up_moves_content_and_blanks_last_row() {
        let mut g = TermGrid::new(2, 3);
        g.write_char('A');
        g.write_char('B');
        g.write_char('C');
        // Row 0: A B C, Row 1: blank
        g.scroll_up();
        // Row 0 should now be row 1's blanks, row 1 should be blank
        assert_eq!(g.cell_at(0, 0).unwrap().ch, ' ');
        assert_eq!(g.cell_at(1, 0).unwrap().ch, ' ');
    }

    #[test]
    fn clear_resets_all_cells_and_cursor() {
        let mut g = TermGrid::new(3, 5);
        g.write_char('Z');
        g.newline();
        g.clear();
        assert_eq!(g.cursor_row, 0);
        assert_eq!(g.cursor_col, 0);
        assert_eq!(g.cell_at(0, 0).unwrap().ch, ' ');
    }

    #[test]
    fn resize_changes_dimensions_and_clears() {
        let mut g = TermGrid::new(3, 5);
        g.write_char('Q');
        g.resize(10, 20);
        assert_eq!(g.rows(), 10);
        assert_eq!(g.cols(), 20);
        assert_eq!(g.cell_at(0, 0).unwrap().ch, ' ');
    }

    #[test]
    fn write_char_on_last_row_triggers_scroll() {
        let mut g = TermGrid::new(2, 2);
        // Fill row 0
        g.write_char('A');
        g.write_char('B');
        // Now on row 1
        g.write_char('C');
        g.write_char('D');
        // Now scroll triggered: cursor stays at last row
        assert_eq!(g.cursor_row, 1);
    }
}
