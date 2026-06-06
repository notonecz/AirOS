// crates/airkit-ui/src/primitives.rs

/// Rectangle in 2D space (logical pixels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_size(width: f32, height: f32) -> Self {
        Self { x: 0.0, y: 0.0, width, height }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Shrinks rect by padding from all sides.
    pub fn inset(&self, padding: Padding) -> Self {
        Self {
            x: self.x + padding.left,
            y: self.y + padding.top,
            width: (self.width - padding.left - padding.right).max(0.0),
            height: (self.height - padding.top - padding.bottom).max(0.0),
        }
    }

    /// Returns true if point is inside rect.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }
}

/// Padding from four sides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub const fn all(v: f32) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }

    pub const fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }
}

/// Element size: fixed value or "fill available space".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UiSize {
    Fixed(f32),
    Fill,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_right_and_bottom() {
        let r = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!((r.right() - 110.0).abs() < f32::EPSILON);
        assert!((r.bottom() - 70.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rect_inset_reduces_size() {
        let r = Rect::from_size(100.0, 80.0);
        let p = Padding::all(10.0);
        let inner = r.inset(p);
        assert!((inner.x - 10.0).abs() < f32::EPSILON);
        assert!((inner.y - 10.0).abs() < f32::EPSILON);
        assert!((inner.width - 80.0).abs() < f32::EPSILON);
        assert!((inner.height - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rect_inset_clamps_to_zero() {
        let r = Rect::from_size(10.0, 10.0);
        let p = Padding::all(20.0);
        let inner = r.inset(p);
        assert!((inner.width - 0.0).abs() < f32::EPSILON);
        assert!((inner.height - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rect_contains_point() {
        let r = Rect::new(10.0, 10.0, 50.0, 50.0);
        assert!(r.contains(30.0, 30.0));
        assert!(!r.contains(5.0, 30.0));
        assert!(!r.contains(30.0, 65.0));
    }

    #[test]
    fn padding_all_sets_all_sides() {
        let p = Padding::all(8.0);
        assert!((p.top - 8.0).abs() < f32::EPSILON);
        assert!((p.right - 8.0).abs() < f32::EPSILON);
        assert!((p.bottom - 8.0).abs() < f32::EPSILON);
        assert!((p.left - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn padding_symmetric() {
        let p = Padding::symmetric(4.0, 8.0);
        assert!((p.top - 4.0).abs() < f32::EPSILON);
        assert!((p.right - 8.0).abs() < f32::EPSILON);
        assert!((p.bottom - 4.0).abs() < f32::EPSILON);
        assert!((p.left - 8.0).abs() < f32::EPSILON);
    }
}
