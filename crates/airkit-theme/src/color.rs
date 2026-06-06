// crates/airkit-theme/src/color.rs

/// RGBA color with values 0.0–1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// Creates color from 0xRRGGBB hex value (alpha = 1.0).
    pub fn from_hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;
        Self::rgb(r, g, b)
    }

    /// Returns color with modified alpha.
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Linearly interpolates between two colors (t = 0.0 → self, t = 1.0 → other).
    pub fn lerp(self, other: Color, t: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_sets_alpha_to_one() {
        let c = Color::rgb(1.0, 0.0, 0.5);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rgba_stores_all_channels() {
        let c = Color::rgba(0.1, 0.2, 0.3, 0.4);
        assert!((c.r - 0.1).abs() < 0.001);
        assert!((c.g - 0.2).abs() < 0.001);
        assert!((c.b - 0.3).abs() < 0.001);
        assert!((c.a - 0.4).abs() < 0.001);
    }

    #[test]
    fn from_hex_parses_rrggbb() {
        let c = Color::from_hex(0xFF8000);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - (128.0 / 255.0)).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
        assert!((c.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn with_alpha_changes_only_alpha() {
        let c = Color::rgb(1.0, 0.5, 0.0).with_alpha(0.5);
        assert!((c.r - 1.0).abs() < f32::EPSILON);
        assert!((c.a - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn lerp_at_zero_returns_self() {
        let a = Color::rgb(1.0, 0.0, 0.0);
        let b = Color::rgb(0.0, 1.0, 0.0);
        let result = a.lerp(b, 0.0);
        assert!((result.r - 1.0).abs() < f32::EPSILON);
        assert!((result.g - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lerp_at_one_returns_other() {
        let a = Color::rgb(1.0, 0.0, 0.0);
        let b = Color::rgb(0.0, 1.0, 0.0);
        let result = a.lerp(b, 1.0);
        assert!((result.r - 0.0).abs() < f32::EPSILON);
        assert!((result.g - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lerp_at_half_averages_channels() {
        let a = Color::rgb(0.0, 0.0, 0.0);
        let b = Color::rgb(1.0, 1.0, 1.0);
        let result = a.lerp(b, 0.5);
        assert!((result.r - 0.5).abs() < 0.001);
    }
}
