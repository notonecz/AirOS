// apps/airsettings/src/app.rs

use crate::appearance::AppearanceSettings;
use crate::category::{SettingsCategory, SidebarNav};
use crate::display::DisplaySettings;
use crate::sound::SoundSettings;
use crate::system::{AboutInfo, NetworkStatus, UserEntry, UsersSettings};
use airbus::messages::MenuItem;

/// Kompletní stav AirSettings — všechny sekce + navigace.
#[derive(Debug)]
pub struct AirSettingsApp {
    pub nav: SidebarNav,
    pub appearance: AppearanceSettings,
    pub display: DisplaySettings,
    pub sound: SoundSettings,
    pub network: NetworkStatus,
    pub users: UsersSettings,
    pub about: AboutInfo,
}

impl AirSettingsApp {
    pub fn new() -> Self {
        let default_user = UserEntry::new(1, "airos", "AirOS User", true);
        Self {
            nav: SidebarNav::new(),
            appearance: AppearanceSettings::new(),
            display: DisplaySettings::new(),
            sound: SoundSettings::new(),
            network: NetworkStatus::new(),
            users: UsersSettings::new(default_user),
            about: AboutInfo::default(),
        }
    }

    /// Přepne na danou sekci nastavení.
    pub fn navigate_to(&mut self, category: SettingsCategory) {
        self.nav.set_active(category);
    }

    /// Vrátí aktuálně aktivní kategorii.
    pub fn active_category(&self) -> SettingsCategory {
        self.nav.active()
    }

    /// Vrátí menu položky pro registraci přes AirBus.
    pub fn menu_items() -> Vec<MenuItem> {
        vec![
            MenuItem {
                id: "appearance".into(),
                label: "Appearance".into(),
                enabled: true,
            },
            MenuItem {
                id: "display".into(),
                label: "Display".into(),
                enabled: true,
            },
            MenuItem {
                id: "sound".into(),
                label: "Sound".into(),
                enabled: true,
            },
        ]
    }
}

impl Default for AirSettingsApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::appearance::ThemeMode;

    fn make_app() -> AirSettingsApp {
        AirSettingsApp::new()
    }

    #[test]
    fn new_app_starts_on_display_category() {
        let app = make_app();
        assert_eq!(app.active_category(), SettingsCategory::Display);
    }

    #[test]
    fn navigate_to_changes_active_category() {
        let mut app = make_app();
        app.navigate_to(SettingsCategory::Appearance);
        assert_eq!(app.active_category(), SettingsCategory::Appearance);
    }

    #[test]
    fn navigate_to_about_works() {
        let mut app = make_app();
        app.navigate_to(SettingsCategory::About);
        assert_eq!(app.active_category(), SettingsCategory::About);
    }

    #[test]
    fn default_appearance_theme_mode_is_light() {
        let app = make_app();
        assert_eq!(app.appearance.theme_mode, ThemeMode::Light);
    }

    #[test]
    fn default_sound_volume_is_0_5() {
        let app = make_app();
        assert!((app.sound.volume - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn default_network_is_not_connected() {
        let app = make_app();
        assert!(!app.network.connected);
    }

    #[test]
    fn default_users_has_one_user() {
        let app = make_app();
        assert_eq!(app.users.users().len(), 1);
    }

    #[test]
    fn default_about_os_name_is_airos() {
        let app = make_app();
        assert_eq!(app.about.os_name, "AirOS");
    }

    #[test]
    fn menu_items_returns_three_items() {
        let items = AirSettingsApp::menu_items();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn menu_items_contains_appearance() {
        let items = AirSettingsApp::menu_items();
        assert!(items.iter().any(|i| i.id == "appearance"));
    }
}
