// apps/airsetup/src/app.rs

use crate::appearance::AppearanceStep;
use crate::language::LanguageStep;
use crate::network::NetworkStep;
use crate::step::WizardStep;
use crate::user::UserStep;
use airbus::messages::MenuItem;

/// Kompletní stav průvodce prvním spuštěním.
#[derive(Debug)]
pub struct AirSetupApp {
    pub current_step: WizardStep,
    pub language: LanguageStep,
    pub user: UserStep,
    pub network: NetworkStep,
    pub appearance: AppearanceStep,
    /// True pokud uživatel dokončil celý průvodce (prošel krokem Done).
    pub completed: bool,
}

impl AirSetupApp {
    pub fn new() -> Self {
        Self {
            current_step: WizardStep::Welcome,
            language: LanguageStep::new(),
            user: UserStep::new(),
            network: NetworkStep::new(),
            appearance: AppearanceStep::new(),
            completed: false,
        }
    }

    /// Vrátí true pokud aktuální krok lze opustit vpřed.
    /// Welcome, Language, Appearance: vždy true.
    /// User: vyžaduje platná data.
    /// Network: vyžaduje výběr nebo skip.
    /// Done: false (nelze pokračovat dál).
    pub fn can_advance(&self) -> bool {
        match self.current_step {
            WizardStep::Welcome => true,
            WizardStep::Language => true,
            WizardStep::User => self.user.is_valid(),
            WizardStep::Network => self.network.is_valid(),
            WizardStep::Appearance => true,
            WizardStep::Done => false,
        }
    }

    /// Posune průvodce na další krok. Vrátí true pokud byl posun úspěšný.
    /// Vrátí false pokud aktuální krok není platný nebo jsme na Done.
    /// Pokud advance() přejde do Done, nastaví `completed = true`.
    pub fn advance(&mut self) -> bool {
        if !self.can_advance() {
            return false;
        }
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            if self.current_step == WizardStep::Done {
                self.completed = true;
            }
            true
        } else {
            false
        }
    }

    /// Vrátí průvodce o krok zpět. No-op pokud jsme na Welcome.
    pub fn go_back(&mut self) {
        if let Some(prev) = self.current_step.prev() {
            self.current_step = prev;
        }
    }

    /// Vrátí menu položky pro registraci přes AirBus.
    pub fn menu_items() -> Vec<MenuItem> {
        vec![
            MenuItem {
                id: "next".into(),
                label: "Next".into(),
                enabled: true,
            },
            MenuItem {
                id: "back".into(),
                label: "Back".into(),
                enabled: true,
            },
        ]
    }
}

impl Default for AirSetupApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app() -> AirSetupApp {
        AirSetupApp::new()
    }

    #[test]
    fn new_app_starts_at_welcome() {
        let app = make_app();
        assert_eq!(app.current_step, WizardStep::Welcome);
    }

    #[test]
    fn new_app_is_not_completed() {
        let app = make_app();
        assert!(!app.completed);
    }

    #[test]
    fn advance_from_welcome_goes_to_language() {
        let mut app = make_app();
        let result = app.advance();
        assert!(result);
        assert_eq!(app.current_step, WizardStep::Language);
    }

    #[test]
    fn go_back_from_language_returns_to_welcome() {
        let mut app = make_app();
        app.advance(); // → Language
        app.go_back();
        assert_eq!(app.current_step, WizardStep::Welcome);
    }

    #[test]
    fn go_back_at_welcome_is_noop() {
        let mut app = make_app();
        app.go_back();
        assert_eq!(app.current_step, WizardStep::Welcome);
    }

    #[test]
    fn advance_blocked_on_user_step_without_data() {
        let mut app = make_app();
        app.advance(); // → Language
        app.advance(); // → User
        let result = app.advance(); // blocked: user not valid
        assert!(!result);
        assert_eq!(app.current_step, WizardStep::User);
    }

    #[test]
    fn advance_allowed_on_user_step_with_valid_data() {
        let mut app = make_app();
        app.advance(); // → Language
        app.advance(); // → User
        app.user.set_username("vo");
        app.user.set_password("secret");
        let result = app.advance(); // → Network
        assert!(result);
        assert_eq!(app.current_step, WizardStep::Network);
    }

    #[test]
    fn advance_blocked_on_network_step_without_selection() {
        let mut app = make_app();
        app.advance(); // → Language
        app.advance(); // → User
        app.user.set_username("vo");
        app.user.set_password("pw");
        app.advance(); // → Network
        let result = app.advance(); // blocked: network not valid
        assert!(!result);
        assert_eq!(app.current_step, WizardStep::Network);
    }

    #[test]
    fn completing_all_steps_sets_completed() {
        let mut app = make_app();
        app.advance(); // → Language
        app.advance(); // → User
        app.user.set_username("vo");
        app.user.set_password("pw");
        app.advance(); // → Network
        app.network.skip();
        app.advance(); // → Appearance
        app.advance(); // → Done, completed = true
        assert!(app.completed);
        assert_eq!(app.current_step, WizardStep::Done);
    }

    #[test]
    fn advance_at_done_returns_false() {
        let mut app = make_app();
        app.advance(); // → Language
        app.advance(); // → User
        app.user.set_username("vo");
        app.user.set_password("pw");
        app.advance(); // → Network
        app.network.skip();
        app.advance(); // → Appearance
        app.advance(); // → Done
        let result = app.advance(); // no-op
        assert!(!result);
        assert_eq!(app.current_step, WizardStep::Done);
    }

    #[test]
    fn menu_items_returns_two_items() {
        let items = AirSetupApp::menu_items();
        assert_eq!(items.len(), 2);
    }
}
