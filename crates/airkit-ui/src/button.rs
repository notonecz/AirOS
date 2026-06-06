// crates/airkit-ui/src/button.rs

use airkit_theme::{Color, ThemeTokens};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct Button {
    pub label: String,
    pub state: ButtonState,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            state: ButtonState::Normal,
        }
    }

    pub fn on_hover(&mut self) {
        if self.state != ButtonState::Disabled {
            self.state = ButtonState::Hovered;
        }
    }

    pub fn on_leave(&mut self) {
        if self.state == ButtonState::Hovered {
            self.state = ButtonState::Normal;
        }
    }

    pub fn on_press(&mut self) {
        if self.state != ButtonState::Disabled {
            self.state = ButtonState::Pressed;
        }
    }

    pub fn on_release(&mut self) {
        if self.state == ButtonState::Pressed {
            self.state = ButtonState::Normal;
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        if disabled {
            self.state = ButtonState::Disabled;
        } else if self.state == ButtonState::Disabled {
            self.state = ButtonState::Normal;
        }
    }

    /// Background color based on current state and theme.
    pub fn background_color(&self, tokens: &ThemeTokens) -> Color {
        match self.state {
            ButtonState::Normal => tokens.primary,
            ButtonState::Hovered => tokens.primary.lerp(Color::rgb(1.0, 1.0, 1.0), 0.15),
            ButtonState::Pressed => tokens.primary.lerp(Color::rgb(0.0, 0.0, 0.0), 0.15),
            ButtonState::Disabled => tokens.border,
        }
    }

    /// Text color based on current state and theme.
    pub fn text_color(&self, tokens: &ThemeTokens) -> Color {
        match self.state {
            ButtonState::Disabled => tokens.text_secondary,
            _ => tokens.on_primary,
        }
    }

    pub fn is_interactive(&self) -> bool {
        self.state != ButtonState::Disabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_button_is_normal() {
        let b = Button::new("OK");
        assert_eq!(b.state, ButtonState::Normal);
        assert_eq!(b.label, "OK");
    }

    #[test]
    fn hover_changes_state() {
        let mut b = Button::new("OK");
        b.on_hover();
        assert_eq!(b.state, ButtonState::Hovered);
    }

    #[test]
    fn leave_returns_to_normal() {
        let mut b = Button::new("OK");
        b.on_hover();
        b.on_leave();
        assert_eq!(b.state, ButtonState::Normal);
    }

    #[test]
    fn press_and_release() {
        let mut b = Button::new("OK");
        b.on_press();
        assert_eq!(b.state, ButtonState::Pressed);
        b.on_release();
        assert_eq!(b.state, ButtonState::Normal);
    }

    #[test]
    fn disabled_ignores_hover_and_press() {
        let mut b = Button::new("OK");
        b.set_disabled(true);
        b.on_hover();
        assert_eq!(b.state, ButtonState::Disabled);
        b.on_press();
        assert_eq!(b.state, ButtonState::Disabled);
    }

    #[test]
    fn enable_after_disable_returns_to_normal() {
        let mut b = Button::new("OK");
        b.set_disabled(true);
        b.set_disabled(false);
        assert_eq!(b.state, ButtonState::Normal);
    }

    #[test]
    fn background_color_differs_per_state() {
        let tokens = ThemeTokens::light();
        let mut b = Button::new("OK");
        let normal_bg = b.background_color(&tokens);
        b.on_hover();
        let hover_bg = b.background_color(&tokens);
        assert!(
            (normal_bg.r - hover_bg.r).abs() > 0.001
                || (normal_bg.g - hover_bg.g).abs() > 0.001
                || (normal_bg.b - hover_bg.b).abs() > 0.001
        );
    }

    #[test]
    fn disabled_not_interactive() {
        let mut b = Button::new("OK");
        assert!(b.is_interactive());
        b.set_disabled(true);
        assert!(!b.is_interactive());
    }
}
