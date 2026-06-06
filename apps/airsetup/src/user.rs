// apps/airsetup/src/user.rs

/// Krok vytvoření prvního uživatelského účtu.
#[derive(Debug, Clone)]
pub struct UserStep {
    pub username: String,
    pub display_name: String,
    /// Heslo v plain-textu — reálné hashování přijde při zápisu do systému.
    pub password: String,
}

impl UserStep {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            display_name: String::new(),
            password: String::new(),
        }
    }

    pub fn set_username(&mut self, s: impl Into<String>) {
        self.username = s.into();
    }

    pub fn set_display_name(&mut self, s: impl Into<String>) {
        self.display_name = s.into();
    }

    pub fn set_password(&mut self, s: impl Into<String>) {
        self.password = s.into();
    }

    /// Krok je platný pokud username i password jsou neprázdné.
    pub fn is_valid(&self) -> bool {
        !self.username.is_empty() && !self.password.is_empty()
    }
}

impl Default for UserStep {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_step_is_invalid() {
        let s = UserStep::new();
        assert!(!s.is_valid());
    }

    #[test]
    fn only_username_is_not_valid() {
        let mut s = UserStep::new();
        s.set_username("vo");
        assert!(!s.is_valid());
    }

    #[test]
    fn only_password_is_not_valid() {
        let mut s = UserStep::new();
        s.set_password("secret");
        assert!(!s.is_valid());
    }

    #[test]
    fn username_and_password_is_valid() {
        let mut s = UserStep::new();
        s.set_username("vo");
        s.set_password("secret");
        assert!(s.is_valid());
    }

    #[test]
    fn set_display_name_stores_value() {
        let mut s = UserStep::new();
        s.set_display_name("Vojtech");
        assert_eq!(s.display_name, "Vojtech");
    }

    #[test]
    fn display_name_not_required_for_validity() {
        let mut s = UserStep::new();
        s.set_username("vo");
        s.set_password("pw");
        // display_name is empty but step is still valid
        assert!(s.is_valid());
    }
}
