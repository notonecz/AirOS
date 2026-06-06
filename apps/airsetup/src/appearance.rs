// apps/airsetup/src/appearance.rs

/// Výběr barevného schématu při prvním spuštění.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SetupTheme {
    Light,
    Dark,
}

/// Krok výběru vzhledu.
#[derive(Debug)]
pub struct AppearanceStep {
    pub theme: SetupTheme,
}

impl AppearanceStep {
    pub fn new() -> Self {
        Self {
            theme: SetupTheme::Light,
        }
    }

    pub fn set_theme(&mut self, theme: SetupTheme) {
        self.theme = theme;
    }
}

impl Default for AppearanceStep {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_is_light() {
        let s = AppearanceStep::new();
        assert_eq!(s.theme, SetupTheme::Light);
    }

    #[test]
    fn set_theme_changes_theme() {
        let mut s = AppearanceStep::new();
        s.set_theme(SetupTheme::Dark);
        assert_eq!(s.theme, SetupTheme::Dark);
    }

    #[test]
    fn light_and_dark_are_distinct() {
        assert_ne!(SetupTheme::Light, SetupTheme::Dark);
    }
}
