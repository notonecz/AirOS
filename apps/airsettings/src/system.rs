// apps/airsettings/src/system.rs

// ── Network ──────────────────────────────────────────────────────────────────

/// Stav síťového připojení (read-only snapshot; daemon ho aktualizuje).
#[derive(Debug, Clone)]
pub struct NetworkStatus {
    pub connected: bool,
    /// SSID Wi-Fi sítě, None pokud jsme na drátu nebo odpojeni.
    pub ssid: Option<String>,
    /// Název síťového rozhraní, např. "en0".
    pub interface: String,
}

impl NetworkStatus {
    pub fn new() -> Self {
        Self {
            connected: false,
            ssid: None,
            interface: "en0".into(),
        }
    }

    /// Aktualizuje stav připojení. ssid je None pro drátové/odpojené.
    pub fn set_connected(&mut self, connected: bool, ssid: Option<String>) {
        self.connected = connected;
        self.ssid = ssid;
    }
}

impl Default for NetworkStatus {
    fn default() -> Self {
        Self::new()
    }
}

// ── Users ─────────────────────────────────────────────────────────────────────

/// Jeden uživatelský účet.
#[derive(Debug, Clone, PartialEq)]
pub struct UserEntry {
    pub id: u32,
    pub username: String,
    pub display_name: String,
    pub is_admin: bool,
}

impl UserEntry {
    pub fn new(
        id: u32,
        username: impl Into<String>,
        display_name: impl Into<String>,
        is_admin: bool,
    ) -> Self {
        Self {
            id,
            username: username.into(),
            display_name: display_name.into(),
            is_admin,
        }
    }
}

/// Správa uživatelských účtů.
#[derive(Debug)]
pub struct UsersSettings {
    users: Vec<UserEntry>,
    active_user_id: u32,
}

impl UsersSettings {
    /// Vytvoří s jedním počátečním uživatelem.
    pub fn new(initial_user: UserEntry) -> Self {
        let id = initial_user.id;
        Self {
            users: vec![initial_user],
            active_user_id: id,
        }
    }

    pub fn users(&self) -> &[UserEntry] {
        &self.users
    }

    pub fn active_user(&self) -> Option<&UserEntry> {
        self.users.iter().find(|u| u.id == self.active_user_id)
    }

    pub fn add_user(&mut self, user: UserEntry) {
        self.users.push(user);
    }

    /// Odstraní uživatele dle ID. No-op pokud je to jediný uživatel nebo ID neexistuje.
    /// Pokud byl odstraněn aktivní uživatel, přepne na prvního dostupného.
    pub fn remove_user(&mut self, id: u32) {
        if self.users.len() <= 1 {
            return;
        }
        self.users.retain(|u| u.id != id);
        if self.active_user_id == id {
            self.active_user_id = self.users[0].id;
        }
    }

    /// Nastaví aktivního uživatele. No-op pokud ID neexistuje.
    pub fn set_active_user(&mut self, id: u32) {
        if self.users.iter().any(|u| u.id == id) {
            self.active_user_id = id;
        }
    }
}

// ── About ─────────────────────────────────────────────────────────────────────

/// Informace o systému (zobrazeno v sekci About).
#[derive(Debug, Clone)]
pub struct AboutInfo {
    pub os_name: String,
    pub version: String,
    pub build: String,
    pub hostname: String,
    pub arch: String,
}

impl AboutInfo {
    pub fn new(
        os_name: impl Into<String>,
        version: impl Into<String>,
        build: impl Into<String>,
        hostname: impl Into<String>,
        arch: impl Into<String>,
    ) -> Self {
        Self {
            os_name: os_name.into(),
            version: version.into(),
            build: build.into(),
            hostname: hostname.into(),
            arch: arch.into(),
        }
    }
}

impl Default for AboutInfo {
    fn default() -> Self {
        Self::new("AirOS", "0.1.0", "dev", "airos", "aarch64")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Network tests ──

    #[test]
    fn network_default_is_not_connected() {
        let n = NetworkStatus::new();
        assert!(!n.connected);
        assert!(n.ssid.is_none());
    }

    #[test]
    fn network_set_connected_updates_state() {
        let mut n = NetworkStatus::new();
        n.set_connected(true, Some("HomeNet".into()));
        assert!(n.connected);
        assert_eq!(n.ssid.as_deref(), Some("HomeNet"));
    }

    #[test]
    fn network_set_connected_false_clears_ssid() {
        let mut n = NetworkStatus::new();
        n.set_connected(true, Some("Net".into()));
        n.set_connected(false, None);
        assert!(!n.connected);
        assert!(n.ssid.is_none());
    }

    // ── Users tests ──

    #[test]
    fn users_new_has_one_user() {
        let u = UserEntry::new(1, "root", "Root User", true);
        let us = UsersSettings::new(u);
        assert_eq!(us.users().len(), 1);
    }

    #[test]
    fn users_active_user_returns_initial() {
        let u = UserEntry::new(42, "vo", "Vojtech", false);
        let us = UsersSettings::new(u.clone());
        assert_eq!(us.active_user().unwrap().id, 42);
    }

    #[test]
    fn users_add_user_increases_count() {
        let u = UserEntry::new(1, "root", "Root", true);
        let mut us = UsersSettings::new(u);
        us.add_user(UserEntry::new(2, "guest", "Guest", false));
        assert_eq!(us.users().len(), 2);
    }

    #[test]
    fn users_remove_only_user_is_noop() {
        let u = UserEntry::new(1, "root", "Root", true);
        let mut us = UsersSettings::new(u);
        us.remove_user(1);
        assert_eq!(us.users().len(), 1);
    }

    #[test]
    fn users_remove_non_active_user_works() {
        let u1 = UserEntry::new(1, "root", "Root", true);
        let u2 = UserEntry::new(2, "guest", "Guest", false);
        let mut us = UsersSettings::new(u1);
        us.add_user(u2);
        us.remove_user(2);
        assert_eq!(us.users().len(), 1);
        assert_eq!(us.active_user().unwrap().id, 1);
    }

    #[test]
    fn users_set_active_user_changes_active() {
        let u1 = UserEntry::new(1, "root", "Root", true);
        let u2 = UserEntry::new(2, "guest", "Guest", false);
        let mut us = UsersSettings::new(u1);
        us.add_user(u2);
        us.set_active_user(2);
        assert_eq!(us.active_user().unwrap().id, 2);
    }

    #[test]
    fn users_set_active_user_unknown_id_is_noop() {
        let u = UserEntry::new(1, "root", "Root", true);
        let mut us = UsersSettings::new(u);
        us.set_active_user(99);
        assert_eq!(us.active_user().unwrap().id, 1);
    }

    // ── About tests ──

    #[test]
    fn about_default_has_non_empty_os_name() {
        let a = AboutInfo::default();
        assert!(!a.os_name.is_empty());
    }

    #[test]
    fn about_new_stores_all_fields() {
        let a = AboutInfo::new("AirOS", "1.0.0", "build42", "myhost", "x86_64");
        assert_eq!(a.os_name, "AirOS");
        assert_eq!(a.version, "1.0.0");
        assert_eq!(a.arch, "x86_64");
    }
}
