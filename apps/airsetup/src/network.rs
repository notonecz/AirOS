// apps/airsetup/src/network.rs

/// Krok výběru sítě. Uživatel může vybrat SSID nebo přeskočit.
#[derive(Debug)]
pub struct NetworkStep {
    available: Vec<String>,
    pub selected: Option<String>,
    pub skipped: bool,
}

impl NetworkStep {
    pub fn new() -> Self {
        Self {
            available: Vec::new(),
            selected: None,
            skipped: false,
        }
    }

    /// Nastaví seznam dostupných sítí (přijde z OS scanu; v pure-state cratu jen uchováme).
    pub fn set_available(&mut self, networks: Vec<String>) {
        self.available = networks;
    }

    pub fn available(&self) -> &[String] {
        &self.available
    }

    /// Vybere síť dle SSID. Zruší skip pokud byl nastaven.
    pub fn select(&mut self, ssid: impl Into<String>) {
        self.selected = Some(ssid.into());
        self.skipped = false;
    }

    /// Přeskočí výběr sítě. Zruší výběr SSID pokud byl nastaven.
    pub fn skip(&mut self) {
        self.skipped = true;
        self.selected = None;
    }

    /// Krok je platný pokud je vybráno SSID nebo je přeskočeno.
    pub fn is_valid(&self) -> bool {
        self.selected.is_some() || self.skipped
    }
}

impl Default for NetworkStep {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_step_is_invalid() {
        let s = NetworkStep::new();
        assert!(!s.is_valid());
    }

    #[test]
    fn select_makes_step_valid() {
        let mut s = NetworkStep::new();
        s.select("HomeNet");
        assert!(s.is_valid());
    }

    #[test]
    fn skip_makes_step_valid() {
        let mut s = NetworkStep::new();
        s.skip();
        assert!(s.is_valid());
    }

    #[test]
    fn select_stores_ssid() {
        let mut s = NetworkStep::new();
        s.select("HomeNet");
        assert_eq!(s.selected.as_deref(), Some("HomeNet"));
    }

    #[test]
    fn skip_clears_selected() {
        let mut s = NetworkStep::new();
        s.select("HomeNet");
        s.skip();
        assert!(s.selected.is_none());
        assert!(s.skipped);
    }

    #[test]
    fn select_after_skip_clears_skipped() {
        let mut s = NetworkStep::new();
        s.skip();
        s.select("Office");
        assert!(!s.skipped);
        assert_eq!(s.selected.as_deref(), Some("Office"));
    }

    #[test]
    fn set_available_stores_networks() {
        let mut s = NetworkStep::new();
        s.set_available(vec!["Net1".into(), "Net2".into()]);
        assert_eq!(s.available().len(), 2);
    }
}
