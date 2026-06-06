// crates/airwm/src/snap.rs

/// Zóna pro snap okna.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapZone {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Vrátí SnapZone pokud je kurzor v oblasti snap triggeru (threshold od hrany/rohu).
/// Rohy mají prioritu před hranami.
pub fn compute_snap(
    cursor_x: f32,
    cursor_y: f32,
    screen_w: f32,
    screen_h: f32,
    threshold: f32,
) -> Option<SnapZone> {
    let near_left = cursor_x < threshold;
    let near_right = cursor_x > screen_w - threshold;
    let near_top = cursor_y < threshold;
    let near_bottom = cursor_y > screen_h - threshold;

    match (near_left, near_right, near_top, near_bottom) {
        (true, _, true, _) => Some(SnapZone::TopLeft),
        (true, _, _, true) => Some(SnapZone::BottomLeft),
        (_, true, true, _) => Some(SnapZone::TopRight),
        (_, true, _, true) => Some(SnapZone::BottomRight),
        (true, _, _, _) => Some(SnapZone::Left),
        (_, true, _, _) => Some(SnapZone::Right),
        _ => None,
    }
}

/// Vrátí cílový rect (x, y, width, height) pro danou snap zónu a velikost obrazovky.
pub fn snap_rect(zone: SnapZone, screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
    let hw = screen_w / 2.0;
    let hh = screen_h / 2.0;
    match zone {
        SnapZone::Left => (0.0, 0.0, hw, screen_h),
        SnapZone::Right => (hw, 0.0, hw, screen_h),
        SnapZone::TopLeft => (0.0, 0.0, hw, hh),
        SnapZone::TopRight => (hw, 0.0, hw, hh),
        SnapZone::BottomLeft => (0.0, hh, hw, hh),
        SnapZone::BottomRight => (hw, hh, hw, hh),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 1920.0;
    const H: f32 = 1080.0;
    const T: f32 = 20.0;

    #[test]
    fn compute_snap_near_left_edge() {
        let zone = compute_snap(5.0, 540.0, W, H, T);
        assert_eq!(zone, Some(SnapZone::Left));
    }

    #[test]
    fn compute_snap_near_right_edge() {
        let zone = compute_snap(1915.0, 540.0, W, H, T);
        assert_eq!(zone, Some(SnapZone::Right));
    }

    #[test]
    fn compute_snap_top_left_corner_takes_priority() {
        let zone = compute_snap(5.0, 5.0, W, H, T);
        assert_eq!(zone, Some(SnapZone::TopLeft));
    }

    #[test]
    fn compute_snap_bottom_right_corner() {
        let zone = compute_snap(1915.0, 1075.0, W, H, T);
        assert_eq!(zone, Some(SnapZone::BottomRight));
    }

    #[test]
    fn compute_snap_middle_returns_none() {
        let zone = compute_snap(960.0, 540.0, W, H, T);
        assert!(zone.is_none());
    }

    #[test]
    fn snap_rect_left_half_width() {
        let (x, y, w, h) = snap_rect(SnapZone::Left, 1920.0, 1080.0);
        assert!((x - 0.0).abs() < f32::EPSILON);
        assert!((y - 0.0).abs() < f32::EPSILON);
        assert!((w - 960.0).abs() < f32::EPSILON);
        assert!((h - 1080.0).abs() < f32::EPSILON);
    }

    #[test]
    fn snap_rect_top_right_quarter() {
        let (x, y, w, h) = snap_rect(SnapZone::TopRight, 1920.0, 1080.0);
        assert!((x - 960.0).abs() < f32::EPSILON);
        assert!((y - 0.0).abs() < f32::EPSILON);
        assert!((w - 960.0).abs() < f32::EPSILON);
        assert!((h - 540.0).abs() < f32::EPSILON);
    }
}
