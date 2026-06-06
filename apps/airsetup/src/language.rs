// apps/airsetup/src/language.rs

/// Dostupný jazyk systému.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Czech,
    Slovak,
}

impl Language {
    /// Vrátí všechny dostupné jazyky.
    pub fn all() -> Vec<Self> {
        vec![Self::English, Self::Czech, Self::Slovak]
    }

    /// Zobrazitelný název jazyka.
    pub fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Czech => "Čeština",
            Self::Slovak => "Slovenčina",
        }
    }

    /// IETF jazykový kód.
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Czech => "cs",
            Self::Slovak => "sk",
        }
    }
}

/// Krok výběru jazyka.
#[derive(Debug)]
pub struct LanguageStep {
    pub selected: Language,
}

impl LanguageStep {
    pub fn new() -> Self {
        Self {
            selected: Language::English,
        }
    }

    pub fn select(&mut self, lang: Language) {
        self.selected = lang;
    }
}

impl Default for LanguageStep {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_three_languages() {
        assert_eq!(Language::all().len(), 3);
    }

    #[test]
    fn all_languages_have_non_empty_labels() {
        for lang in Language::all() {
            assert!(!lang.label().is_empty());
        }
    }

    #[test]
    fn all_languages_have_distinct_codes() {
        let codes: Vec<_> = Language::all().iter().map(|l| l.code()).collect();
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(codes.len(), unique.len());
    }

    #[test]
    fn default_selected_is_english() {
        let s = LanguageStep::new();
        assert_eq!(s.selected, Language::English);
    }

    #[test]
    fn select_changes_language() {
        let mut s = LanguageStep::new();
        s.select(Language::Czech);
        assert_eq!(s.selected, Language::Czech);
    }
}
