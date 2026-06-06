// crates/airkit-theme/src/tokens.rs

use crate::color::Color;

/// Complete set of design tokens for one theme.
#[derive(Debug, Clone)]
pub struct ThemeTokens {
    // --- Colors ---
    pub background: Color,
    pub surface: Color,
    pub primary: Color,
    pub on_primary: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub border: Color,
    pub chrome: Color,

    // --- Spacing (logical pixels) ---
    pub spacing_xs: f32, // 4
    pub spacing_sm: f32, // 8
    pub spacing_md: f32, // 16
    pub spacing_lg: f32, // 24
    pub spacing_xl: f32, // 32

    // --- Border radius ---
    pub radius_sm: f32, // 4
    pub radius_md: f32, // 8
    pub radius_lg: f32, // 12

    // --- Font sizes ---
    pub font_size_sm: f32, // 12
    pub font_size_md: f32, // 14
    pub font_size_lg: f32, // 16
    pub font_size_xl: f32, // 20
}

impl ThemeTokens {
    /// Light theme preset.
    pub fn light() -> Self {
        Self {
            background: Color::from_hex(0xF5F5F7),
            surface: Color::from_hex(0xFFFFFF),
            primary: Color::from_hex(0x0066CC),
            on_primary: Color::from_hex(0xFFFFFF),
            text: Color::from_hex(0x1D1D1F),
            text_secondary: Color::from_hex(0x6E6E73),
            border: Color::from_hex(0xD2D2D7),
            chrome: Color::from_hex(0xE8E8ED),
            spacing_xs: 4.0,
            spacing_sm: 8.0,
            spacing_md: 16.0,
            spacing_lg: 24.0,
            spacing_xl: 32.0,
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_lg: 12.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
        }
    }

    /// Dark theme preset.
    pub fn dark() -> Self {
        Self {
            background: Color::from_hex(0x1C1C1E),
            surface: Color::from_hex(0x2C2C2E),
            primary: Color::from_hex(0x0A84FF),
            on_primary: Color::from_hex(0xFFFFFF),
            text: Color::from_hex(0xF5F5F7),
            text_secondary: Color::from_hex(0x8E8E93),
            border: Color::from_hex(0x3A3A3C),
            chrome: Color::from_hex(0x242426),
            spacing_xs: 4.0,
            spacing_sm: 8.0,
            spacing_md: 16.0,
            spacing_lg: 24.0,
            spacing_xl: 32.0,
            radius_sm: 4.0,
            radius_md: 8.0,
            radius_lg: 12.0,
            font_size_sm: 12.0,
            font_size_md: 14.0,
            font_size_lg: 16.0,
            font_size_xl: 20.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_theme_has_light_background() {
        let t = ThemeTokens::light();
        assert!(t.background.r > 0.9);
        assert!(t.background.g > 0.9);
        assert!(t.background.b > 0.9);
    }

    #[test]
    fn dark_theme_has_dark_background() {
        let t = ThemeTokens::dark();
        assert!(t.background.r < 0.2);
        assert!(t.background.g < 0.2);
        assert!(t.background.b < 0.2);
    }

    #[test]
    fn spacing_values_are_ascending() {
        let t = ThemeTokens::light();
        assert!(t.spacing_xs < t.spacing_sm);
        assert!(t.spacing_sm < t.spacing_md);
        assert!(t.spacing_md < t.spacing_lg);
        assert!(t.spacing_lg < t.spacing_xl);
    }

    #[test]
    fn radius_values_are_ascending() {
        let t = ThemeTokens::light();
        assert!(t.radius_sm < t.radius_md);
        assert!(t.radius_md < t.radius_lg);
    }

    #[test]
    fn font_sizes_are_ascending() {
        let t = ThemeTokens::light();
        assert!(t.font_size_sm < t.font_size_md);
        assert!(t.font_size_md < t.font_size_lg);
        assert!(t.font_size_lg < t.font_size_xl);
    }

    #[test]
    fn light_and_dark_primary_colors_differ() {
        let light = ThemeTokens::light();
        let dark = ThemeTokens::dark();
        assert!(
            (light.primary.r - dark.primary.r).abs() > 0.01
                || (light.primary.g - dark.primary.g).abs() > 0.01
                || (light.primary.b - dark.primary.b).abs() > 0.01
        );
    }

    #[test]
    fn clone_produces_independent_copy() {
        let t1 = ThemeTokens::light();
        let t2 = t1.clone();
        assert!((t1.spacing_md - t2.spacing_md).abs() < f32::EPSILON);
    }
}
