// apps/airsetup/src/step.rs

/// Kroky průvodce prvním spuštěním v pořadí.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    Welcome,
    Language,
    User,
    Network,
    Appearance,
    Done,
}

impl WizardStep {
    /// Vrátí všechny kroky v pořadí průvodce.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Welcome,
            Self::Language,
            Self::User,
            Self::Network,
            Self::Appearance,
            Self::Done,
        ]
    }

    /// Vrátí následující krok. None pokud jsme na Done.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::Language),
            Self::Language => Some(Self::User),
            Self::User => Some(Self::Network),
            Self::Network => Some(Self::Appearance),
            Self::Appearance => Some(Self::Done),
            Self::Done => None,
        }
    }

    /// Vrátí předchozí krok. None pokud jsme na Welcome.
    pub fn prev(self) -> Option<Self> {
        match self {
            Self::Welcome => None,
            Self::Language => Some(Self::Welcome),
            Self::User => Some(Self::Language),
            Self::Network => Some(Self::User),
            Self::Appearance => Some(Self::Network),
            Self::Done => Some(Self::Appearance),
        }
    }

    /// Vrátí true pokud je to poslední krok (Done).
    pub fn is_last(self) -> bool {
        self == Self::Done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_returns_six_steps() {
        assert_eq!(WizardStep::all().len(), 6);
    }

    #[test]
    fn welcome_next_is_language() {
        assert_eq!(WizardStep::Welcome.next(), Some(WizardStep::Language));
    }

    #[test]
    fn done_next_is_none() {
        assert_eq!(WizardStep::Done.next(), None);
    }

    #[test]
    fn language_prev_is_welcome() {
        assert_eq!(WizardStep::Language.prev(), Some(WizardStep::Welcome));
    }

    #[test]
    fn welcome_prev_is_none() {
        assert_eq!(WizardStep::Welcome.prev(), None);
    }

    #[test]
    fn done_is_last() {
        assert!(WizardStep::Done.is_last());
    }

    #[test]
    fn welcome_is_not_last() {
        assert!(!WizardStep::Welcome.is_last());
    }

    #[test]
    fn next_then_prev_returns_original() {
        let step = WizardStep::User;
        assert_eq!(step.next().unwrap().prev(), Some(step));
    }
}
