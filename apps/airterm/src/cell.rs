/// Terminálová barva. Default/Basic odpovídá barvám ANSI 0–255, Rgb pro true color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TermColor {
    Default,
    Basic(u8),
    Rgb(u8, u8, u8),
}

/// Styl buňky (kombinace příznaků).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CellStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl CellStyle {
    pub const fn plain() -> Self {
        Self { bold: false, italic: false, underline: false }
    }

    pub const fn bold() -> Self {
        Self { bold: true, italic: false, underline: false }
    }
}

/// Jedna buňka v terminálovém gridu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: TermColor,
    pub bg: TermColor,
    pub style: CellStyle,
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            fg: TermColor::Default,
            bg: TermColor::Default,
            style: CellStyle::plain(),
        }
    }

    pub fn blank() -> Self {
        Self::new(' ')
    }

    pub fn with_style(mut self, style: CellStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_fg(mut self, fg: TermColor) -> Self {
        self.fg = fg;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_cell_is_space_with_default_colors() {
        let c = Cell::blank();
        assert_eq!(c.ch, ' ');
        assert_eq!(c.fg, TermColor::Default);
        assert_eq!(c.bg, TermColor::Default);
        assert!(!c.style.bold);
    }

    #[test]
    fn new_cell_stores_char() {
        let c = Cell::new('A');
        assert_eq!(c.ch, 'A');
    }

    #[test]
    fn with_style_sets_bold() {
        let c = Cell::new('X').with_style(CellStyle::bold());
        assert!(c.style.bold);
    }

    #[test]
    fn with_fg_sets_color() {
        let c = Cell::new('Z').with_fg(TermColor::Basic(196));
        assert_eq!(c.fg, TermColor::Basic(196));
    }

    #[test]
    fn term_color_rgb_equality() {
        assert_eq!(TermColor::Rgb(255, 0, 128), TermColor::Rgb(255, 0, 128));
        assert_ne!(TermColor::Rgb(255, 0, 0), TermColor::Rgb(0, 255, 0));
    }

    #[test]
    fn cell_style_default_is_plain() {
        let s = CellStyle::default();
        assert!(!s.bold);
        assert!(!s.italic);
        assert!(!s.underline);
    }
}
