// crates/airwm/src/window.rs

use airproto::types::{Point, Size, WindowId};
use crate::snap::{snap_rect, SnapZone};

/// Stav okna.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Snapped(SnapZone),
}

/// Spravované okno v AirWM.
#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub id: WindowId,
    pub title: String,
    pub position: Point,
    pub size: Size,
    pub state: WindowState,
}

impl ManagedWindow {
    pub fn new(id: WindowId, title: impl Into<String>, position: Point, size: Size) -> Self {
        Self {
            id,
            title: title.into(),
            position,
            size,
            state: WindowState::Normal,
        }
    }

    /// Přesune okno. Ignoruje se pokud není Normal.
    pub fn move_to(&mut self, position: Point) {
        if self.state == WindowState::Normal {
            self.position = position;
        }
    }

    /// Změní velikost. Ignoruje se pokud není Normal.
    pub fn resize_to(&mut self, size: Size) {
        if self.state == WindowState::Normal {
            self.size = size;
        }
    }

    pub fn minimize(&mut self) {
        self.state = WindowState::Minimized;
    }

    pub fn restore(&mut self) {
        self.state = WindowState::Normal;
    }

    pub fn maximize(&mut self, screen: Size) {
        self.state = WindowState::Maximized;
        self.position = Point { x: 0.0, y: 0.0 };
        self.size = screen;
    }

    /// Přichytí okno na danou snap zónu.
    pub fn snap_to(&mut self, zone: SnapZone, screen: Size) {
        let (x, y, w, h) = snap_rect(zone, screen.width as f32, screen.height as f32);
        self.state = WindowState::Snapped(zone);
        self.position = Point { x, y };
        self.size = Size {
            width: w as u32,
            height: h as u32,
        };
    }

    /// Vrátí false pokud je okno minimalizované.
    pub fn is_visible(&self) -> bool {
        self.state != WindowState::Minimized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_window() -> ManagedWindow {
        ManagedWindow::new(
            WindowId(1),
            "Test",
            Point { x: 100.0, y: 100.0 },
            Size { width: 800, height: 600 },
        )
    }

    fn screen() -> Size {
        Size { width: 1920, height: 1080 }
    }

    #[test]
    fn new_window_is_normal_and_visible() {
        let w = make_window();
        assert_eq!(w.state, WindowState::Normal);
        assert!(w.is_visible());
    }

    #[test]
    fn minimize_hides_window() {
        let mut w = make_window();
        w.minimize();
        assert_eq!(w.state, WindowState::Minimized);
        assert!(!w.is_visible());
    }

    #[test]
    fn restore_from_minimize() {
        let mut w = make_window();
        w.minimize();
        w.restore();
        assert_eq!(w.state, WindowState::Normal);
        assert!(w.is_visible());
    }

    #[test]
    fn maximize_fills_screen() {
        let mut w = make_window();
        w.maximize(screen());
        assert_eq!(w.state, WindowState::Maximized);
        assert_eq!(w.size.width, 1920);
        assert_eq!(w.size.height, 1080);
        assert!((w.position.x - 0.0).abs() < f32::EPSILON);
        assert!((w.position.y - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn snap_to_left_sets_half_width() {
        let mut w = make_window();
        w.snap_to(SnapZone::Left, screen());
        assert_eq!(w.state, WindowState::Snapped(SnapZone::Left));
        assert_eq!(w.size.width, 960);
        assert_eq!(w.size.height, 1080);
        assert!((w.position.x - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn move_to_only_works_in_normal_state() {
        let mut w = make_window();
        w.maximize(screen());
        w.move_to(Point { x: 50.0, y: 50.0 });
        // Position must not change — window is maximized
        assert!((w.position.x - 0.0).abs() < f32::EPSILON);
    }
}
