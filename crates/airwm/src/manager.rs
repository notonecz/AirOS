// crates/airwm/src/manager.rs

use crate::snap::SnapZone;
use crate::window::ManagedWindow;
use airproto::types::{Point, Size, WindowId};
use std::collections::HashMap;

pub struct WindowManager {
    windows: HashMap<WindowId, ManagedWindow>,
    z_order: Vec<WindowId>,
    screen: Size,
}

impl WindowManager {
    pub fn new(screen: Size) -> Self {
        Self {
            windows: HashMap::new(),
            z_order: Vec::new(),
            screen,
        }
    }

    pub fn add_window(
        &mut self,
        id: WindowId,
        title: impl Into<String>,
        position: Point,
        size: Size,
    ) {
        self.windows
            .insert(id, ManagedWindow::new(id, title, position, size));
        if !self.z_order.contains(&id) {
            self.z_order.push(id);
        }
    }

    pub fn remove_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
        self.z_order.retain(|&w| w != id);
    }

    /// Přesune okno na vrchol z-stacku (focus).
    pub fn raise(&mut self, id: WindowId) {
        if self.windows.contains_key(&id) {
            self.z_order.retain(|&w| w != id);
            self.z_order.push(id);
        }
    }

    pub fn move_window(&mut self, id: WindowId, position: Point) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.move_to(position);
        }
    }

    pub fn resize_window(&mut self, id: WindowId, size: Size) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.resize_to(size);
        }
    }

    pub fn minimize_window(&mut self, id: WindowId) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.minimize();
        }
    }

    pub fn restore_window(&mut self, id: WindowId) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.restore();
        }
    }

    pub fn maximize_window(&mut self, id: WindowId) {
        let screen = self.screen;
        if let Some(w) = self.windows.get_mut(&id) {
            w.maximize(screen);
        }
    }

    pub fn snap_window(&mut self, id: WindowId, zone: SnapZone) {
        let screen = self.screen;
        if let Some(w) = self.windows.get_mut(&id) {
            w.snap_to(zone, screen);
        }
    }

    pub fn get(&self, id: WindowId) -> Option<&ManagedWindow> {
        self.windows.get(&id)
    }

    /// Vrátí WindowId v z-pořadí: první = nejníže, poslední = nahoře (focused).
    pub fn z_order(&self) -> &[WindowId] {
        &self.z_order
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Size {
        Size {
            width: 1920,
            height: 1080,
        }
    }

    fn pos(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    fn sz(w: u32, h: u32) -> Size {
        Size {
            width: w,
            height: h,
        }
    }

    #[test]
    fn add_window_increases_count() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        assert_eq!(wm.window_count(), 1);
        assert!(wm.get(WindowId(1)).is_some());
    }

    #[test]
    fn remove_window_decreases_count_and_clears_z_order() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        wm.remove_window(WindowId(1));
        assert_eq!(wm.window_count(), 0);
        assert!(!wm.z_order().contains(&WindowId(1)));
    }

    #[test]
    fn raise_moves_window_to_top_of_z_order() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "A", pos(0.0, 0.0), sz(400, 300));
        wm.add_window(WindowId(2), "B", pos(50.0, 50.0), sz(400, 300));
        wm.add_window(WindowId(3), "C", pos(100.0, 100.0), sz(400, 300));
        wm.raise(WindowId(1));
        let z = wm.z_order();
        assert_eq!(z[z.len() - 1], WindowId(1));
        assert_eq!(z.len(), 3);
    }

    #[test]
    fn move_window_changes_position() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        wm.move_window(WindowId(1), pos(200.0, 150.0));
        let w = wm.get(WindowId(1)).unwrap();
        assert!((w.position.x - 200.0).abs() < f32::EPSILON);
        assert!((w.position.y - 150.0).abs() < f32::EPSILON);
    }

    #[test]
    fn minimize_and_restore_window() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        wm.minimize_window(WindowId(1));
        assert!(!wm.get(WindowId(1)).unwrap().is_visible());
        wm.restore_window(WindowId(1));
        assert!(wm.get(WindowId(1)).unwrap().is_visible());
    }

    #[test]
    fn maximize_window_fills_screen() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(100.0, 100.0), sz(400, 300));
        wm.maximize_window(WindowId(1));
        let w = wm.get(WindowId(1)).unwrap();
        assert_eq!(w.size.width, 1920);
        assert_eq!(w.size.height, 1080);
    }

    #[test]
    fn snap_window_snaps_to_right_zone() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        wm.snap_window(WindowId(1), SnapZone::Right);
        let w = wm.get(WindowId(1)).unwrap();
        assert_eq!(w.size.width, 960);
        assert!((w.position.x - 960.0).abs() < f32::EPSILON);
    }

    #[test]
    fn z_order_reflects_insertion_order() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "A", pos(0.0, 0.0), sz(400, 300));
        wm.add_window(WindowId(2), "B", pos(50.0, 50.0), sz(400, 300));
        assert_eq!(wm.z_order(), &[WindowId(1), WindowId(2)]);
    }
}
