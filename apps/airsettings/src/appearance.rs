// apps/airsettings/src/appearance.rs

use airkit_theme::Color;

/// Režim barevného schématu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    /// Automaticky dle systémového nastavení.
    Auto,
}

/// Preset accent barvy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccentColor {
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Yellow,
    Green,
    Graphite,
}

impl AccentColor {
    /// Vrátí RGBA barvu pro daný preset.
    pub fn to_color(self) -> Color {
        match self {
            Self::Blue => Color::from_hex(0x007AFF),
            Self::Purple => Color::from_hex(0xBF5AF2),
            Self::Pink => Color::from_hex(0xFF2D55),
            Self::Red => Color::from_hex(0xFF3B30),
            Self::Orange => Color::from_hex(0xFF9500),
            Self::Yellow => Color::from_hex(0xFFCC00),
            Self::Green => Color::from_hex(0x34C759),
            Self::Graphite => Color::from_hex(0x8E8E93),
        }
    }
}

/// Nastavení vzhledu systému.
#[derive(Debug, Clone)]
pub struct AppearanceSettings {
    pub theme_mode: ThemeMode,
    pub accent: AccentColor,
    /// Měřítko písma, rozsah 0.8–2.0. Výchozí 1.0.
    pub font_scale: f32,
}

impl AppearanceSettings {
    pub fn new() -> Self {
        Self {
            theme_mode: ThemeMode::Light,
            accent: AccentColor::Blue,
            font_scale: 1.0,
        }
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
    }

    pub fn set_accent(&mut self, accent: AccentColor) {
        self.accent = accent;
    }

    /// Nastaví font_scale. Hodnota je ořezána na rozsah 0.8–2.0.
    pub fn set_font_scale(&mut self, scale: f32) {
        self.font_scale = scale.clamp(0.8, 2.0);
    }
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_mode_is_light() {
        let s = AppearanceSettings::new();
        assert_eq!(s.theme_mode, ThemeMode::Light);
    }

    #[test]
    fn default_accent_is_blue() {
        let s = AppearanceSettings::new();
        assert_eq!(s.accent, AccentColor::Blue);
    }

    #[test]
    fn default_font_scale_is_one() {
        let s = AppearanceSettings::new();
        assert!((s.font_scale - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_theme_mode_changes_mode() {
        let mut s = AppearanceSettings::new();
        s.set_theme_mode(ThemeMode::Dark);
        assert_eq!(s.theme_mode, ThemeMode::Dark);
    }

    #[test]
    fn set_accent_changes_accent() {
        let mut s = AppearanceSettings::new();
        s.set_accent(AccentColor::Green);
        assert_eq!(s.accent, AccentColor::Green);
    }

    #[test]
    fn set_font_scale_clamps_to_min() {
        let mut s = AppearanceSettings::new();
        s.set_font_scale(0.1);
        assert!((s.font_scale - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn set_font_scale_clamps_to_max() {
        let mut s = AppearanceSettings::new();
        s.set_font_scale(5.0);
        assert!((s.font_scale - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn accent_to_color_returns_distinct_colors_for_blue_and_red() {
        let blue = AccentColor::Blue.to_color();
        let red = AccentColor::Red.to_color();
        assert_ne!(blue, red);
    }
}
