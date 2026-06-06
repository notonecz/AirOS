// crates/airkit-ui/src/label.rs

use airkit_theme::{Color, ThemeTokens};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub text: String,
    pub align: TextAlign,
    pub font_size: f32,
    pub secondary: bool,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            align: TextAlign::Left,
            font_size: 14.0,
            secondary: false,
        }
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn as_secondary(mut self) -> Self {
        self.secondary = true;
        self
    }

    /// Text color from theme.
    pub fn color(&self, tokens: &ThemeTokens) -> Color {
        if self.secondary {
            tokens.text_secondary
        } else {
            tokens.text
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_label_defaults() {
        let l = Label::new("Hello");
        assert_eq!(l.text, "Hello");
        assert_eq!(l.align, TextAlign::Left);
        assert!((l.font_size - 14.0).abs() < f32::EPSILON);
        assert!(!l.secondary);
    }

    #[test]
    fn builder_chain() {
        let l = Label::new("Hi")
            .with_align(TextAlign::Center)
            .with_font_size(20.0)
            .as_secondary();
        assert_eq!(l.align, TextAlign::Center);
        assert!((l.font_size - 20.0).abs() < f32::EPSILON);
        assert!(l.secondary);
    }

    #[test]
    fn primary_label_uses_text_color() {
        let tokens = ThemeTokens::light();
        let l = Label::new("Title");
        let color = l.color(&tokens);
        assert!((color.r - tokens.text.r).abs() < f32::EPSILON);
    }

    #[test]
    fn secondary_label_uses_text_secondary_color() {
        let tokens = ThemeTokens::light();
        let l = Label::new("Subtitle").as_secondary();
        let color = l.color(&tokens);
        assert!((color.r - tokens.text_secondary.r).abs() < f32::EPSILON);
    }

    #[test]
    fn primary_and_secondary_colors_differ() {
        let tokens = ThemeTokens::light();
        let primary = Label::new("A").color(&tokens);
        let secondary = Label::new("B").as_secondary().color(&tokens);
        assert!(
            (primary.r - secondary.r).abs() > 0.01
                || (primary.g - secondary.g).abs() > 0.01
                || (primary.b - secondary.b).abs() > 0.01
        );
    }
}
