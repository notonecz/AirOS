// crates/airproto/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_id_equality() {
        assert_eq!(WindowId(1), WindowId(1));
        assert_ne!(WindowId(1), WindowId(2));
    }

    #[test]
    fn size_fields() {
        let s = Size { width: 800, height: 600 };
        assert_eq!(s.width, 800);
        assert_eq!(s.height, 600);
    }

    #[test]
    fn point_fields() {
        let p = Point { x: 1.5, y: 2.5 };
        assert!((p.x - 1.5).abs() < f32::EPSILON);
        assert!((p.y - 2.5).abs() < f32::EPSILON);
    }
}
