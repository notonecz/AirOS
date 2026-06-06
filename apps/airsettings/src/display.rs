// apps/airsettings/src/display.rs

/// Nastavení displeje.
#[derive(Debug, Clone)]
pub struct DisplaySettings {
    /// Jas displeje, rozsah 0.0–1.0. Výchozí 0.8.
    pub brightness: f32,
    /// Název aktuálního rozlišení, např. "2560x1600".
    pub resolution: String,
    /// Obnovovací frekvence v Hz, např. 60 nebo 120.
    pub refresh_hz: u32,
}

impl DisplaySettings {
    pub fn new() -> Self {
        Self {
            brightness: 0.8,
            resolution: "2560x1600".into(),
            refresh_hz: 60,
        }
    }

    /// Nastaví jas. Hodnota je ořezána na rozsah 0.0–1.0.
    pub fn set_brightness(&mut self, value: f32) {
        self.brightness = value.clamp(0.0, 1.0);
    }

    pub fn set_resolution(&mut self, resolution: impl Into<String>) {
        self.resolution = resolution.into();
    }

    pub fn set_refresh_hz(&mut self, hz: u32) {
        self.refresh_hz = hz;
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_brightness_is_0_8() {
        let d = DisplaySettings::new();
        assert!((d.brightness - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn default_resolution_is_non_empty() {
        let d = DisplaySettings::new();
        assert!(!d.resolution.is_empty());
    }

    #[test]
    fn default_refresh_hz_is_60() {
        let d = DisplaySettings::new();
        assert_eq!(d.refresh_hz, 60);
    }

    #[test]
    fn set_brightness_clamps_to_zero() {
        let mut d = DisplaySettings::new();
        d.set_brightness(-1.0);
        assert!((d.brightness - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_brightness_clamps_to_one() {
        let mut d = DisplaySettings::new();
        d.set_brightness(2.0);
        assert!((d.brightness - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_resolution_changes_value() {
        let mut d = DisplaySettings::new();
        d.set_resolution("1920x1080");
        assert_eq!(d.resolution, "1920x1080");
    }

    #[test]
    fn set_refresh_hz_changes_value() {
        let mut d = DisplaySettings::new();
        d.set_refresh_hz(120);
        assert_eq!(d.refresh_hz, 120);
    }
}
