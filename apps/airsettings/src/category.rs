// apps/airsettings/src/category.rs

/// Kategorie v nastavení (položky sidebaru).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    Display,
    Sound,
    Network,
    Users,
    Appearance,
    About,
}

impl SettingsCategory {
    /// Vrátí všechny kategorie v pořadí sidebaru.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Display,
            Self::Sound,
            Self::Network,
            Self::Users,
            Self::Appearance,
            Self::About,
        ]
    }

    /// Zobrazitelný název kategorie.
    pub fn label(self) -> &'static str {
        match self {
            Self::Display => "Display",
            Self::Sound => "Sound",
            Self::Network => "Network",
            Self::Users => "Users",
            Self::Appearance => "Appearance",
            Self::About => "About",
        }
    }
}

/// Navigační stav sidebaru — drží aktivní kategorii.
#[derive(Debug)]
pub struct SidebarNav {
    active: SettingsCategory,
}

impl SidebarNav {
    pub fn new() -> Self {
        Self {
            active: SettingsCategory::Display,
        }
    }

    pub fn active(&self) -> SettingsCategory {
        self.active
    }

    pub fn set_active(&mut self, category: SettingsCategory) {
        self.active = category;
    }
}

impl Default for SidebarNav {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_six_categories() {
        assert_eq!(SettingsCategory::all().len(), 6);
    }

    #[test]
    fn all_categories_have_non_empty_labels() {
        for cat in SettingsCategory::all() {
            assert!(!cat.label().is_empty(), "label for {:?} is empty", cat);
        }
    }

    #[test]
    fn all_labels_are_distinct() {
        let labels: Vec<_> = SettingsCategory::all().iter().map(|c| c.label()).collect();
        let unique: std::collections::HashSet<_> = labels.iter().collect();
        assert_eq!(labels.len(), unique.len());
    }

    #[test]
    fn sidebar_nav_default_active_is_display() {
        let nav = SidebarNav::new();
        assert_eq!(nav.active(), SettingsCategory::Display);
    }

    #[test]
    fn set_active_changes_category() {
        let mut nav = SidebarNav::new();
        nav.set_active(SettingsCategory::Appearance);
        assert_eq!(nav.active(), SettingsCategory::Appearance);
    }

    #[test]
    fn set_active_can_switch_multiple_times() {
        let mut nav = SidebarNav::new();
        nav.set_active(SettingsCategory::Sound);
        nav.set_active(SettingsCategory::About);
        assert_eq!(nav.active(), SettingsCategory::About);
    }
}
