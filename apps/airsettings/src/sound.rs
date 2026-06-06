// apps/airsettings/src/sound.rs

/// Nastavení zvuku.
#[derive(Debug, Clone)]
pub struct SoundSettings {
    /// Hlasitost, rozsah 0.0–1.0. Výchozí 0.5.
    pub volume: f32,
    /// Zda je zvuk ztlumený.
    pub muted: bool,
    /// Název výstupního zařízení.
    pub output_device: String,
}

impl SoundSettings {
    pub fn new() -> Self {
        Self {
            volume: 0.5,
            muted: false,
            output_device: "Built-in Speakers".into(),
        }
    }

    /// Nastaví hlasitost. Hodnota je ořezána na rozsah 0.0–1.0.
    pub fn set_volume(&mut self, value: f32) {
        self.volume = value.clamp(0.0, 1.0);
    }

    /// Přepne muted/unmuted.
    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn set_output_device(&mut self, device: impl Into<String>) {
        self.output_device = device.into();
    }

    /// Vrátí efektivní hlasitost — 0.0 pokud je muted, jinak volume.
    pub fn effective_volume(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume
        }
    }
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_volume_is_0_5() {
        let s = SoundSettings::new();
        assert!((s.volume - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn default_muted_is_false() {
        let s = SoundSettings::new();
        assert!(!s.muted);
    }

    #[test]
    fn default_output_device_is_non_empty() {
        let s = SoundSettings::new();
        assert!(!s.output_device.is_empty());
    }

    #[test]
    fn set_volume_clamps_to_zero() {
        let mut s = SoundSettings::new();
        s.set_volume(-0.5);
        assert!((s.volume - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_volume_clamps_to_one() {
        let mut s = SoundSettings::new();
        s.set_volume(1.5);
        assert!((s.volume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn toggle_mute_flips_muted() {
        let mut s = SoundSettings::new();
        s.toggle_mute();
        assert!(s.muted);
        s.toggle_mute();
        assert!(!s.muted);
    }

    #[test]
    fn effective_volume_is_zero_when_muted() {
        let mut s = SoundSettings::new();
        s.toggle_mute();
        assert!((s.effective_volume() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn effective_volume_returns_volume_when_not_muted() {
        let mut s = SoundSettings::new();
        s.set_volume(0.7);
        assert!((s.effective_volume() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn set_output_device_changes_device() {
        let mut s = SoundSettings::new();
        s.set_output_device("USB Headphones");
        assert_eq!(s.output_device, "USB Headphones");
    }
}
