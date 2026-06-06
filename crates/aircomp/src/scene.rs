use crate::window::WindowState;
use airproto::types::{Size, WindowId};
use std::collections::HashMap;

pub struct WindowScene {
    windows: HashMap<WindowId, WindowState>,
    z_order: Vec<WindowId>,
}

impl Default for WindowScene {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowScene {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            z_order: Vec::new(),
        }
    }

    pub fn add_window(&mut self, id: WindowId, size: Size) {
        self.windows.insert(id, WindowState::new(size));
        if !self.z_order.contains(&id) {
            self.z_order.push(id);
        }
    }

    pub fn remove_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
        self.z_order.retain(|&w| w != id);
    }

    pub fn get(&self, id: WindowId) -> Option<&WindowState> {
        self.windows.get(&id)
    }

    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut WindowState> {
        self.windows.get_mut(&id)
    }

    /// Vrátí okna v z-pořadí: první = nejníže, poslední = nahoře (focused).
    pub fn z_order(&self) -> &[WindowId] {
        &self.z_order
    }

    /// Přesune okno na vrchol z-stacku (focused).
    pub fn raise_to_front(&mut self, id: WindowId) {
        self.z_order.retain(|&w| w != id);
        self.z_order.push(id);
    }

    pub fn resize_window(&mut self, id: WindowId, new_size: Size) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.resize(new_size);
        }
    }

    pub fn commit_buffer(&mut self, id: WindowId, data: Vec<u8>) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.commit_buffer(data);
        }
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size(w: u32, h: u32) -> Size {
        Size {
            width: w,
            height: h,
        }
    }

    #[test]
    fn add_and_get_window() {
        let mut scene = WindowScene::new();
        scene.add_window(WindowId(1), size(800, 600));
        assert!(scene.get(WindowId(1)).is_some());
        assert_eq!(scene.window_count(), 1);
    }

    #[test]
    fn remove_window() {
        let mut scene = WindowScene::new();
        scene.add_window(WindowId(1), size(800, 600));
        scene.remove_window(WindowId(1));
        assert!(scene.get(WindowId(1)).is_none());
        assert_eq!(scene.window_count(), 0);
    }

    #[test]
    fn remove_nonexistent_window_is_noop() {
        let mut scene = WindowScene::new();
        scene.remove_window(WindowId(99));
        assert_eq!(scene.window_count(), 0);
    }

    #[test]
    fn z_order_reflects_insertion() {
        let mut scene = WindowScene::new();
        scene.add_window(WindowId(1), size(100, 100));
        scene.add_window(WindowId(2), size(100, 100));
        assert_eq!(scene.z_order(), &[WindowId(1), WindowId(2)]);
    }

    #[test]
    fn raise_to_front_moves_window_to_top() {
        let mut scene = WindowScene::new();
        scene.add_window(WindowId(1), size(100, 100));
        scene.add_window(WindowId(2), size(100, 100));
        scene.add_window(WindowId(3), size(100, 100));
        scene.raise_to_front(WindowId(1));
        let z = scene.z_order();
        assert_eq!(z[z.len() - 1], WindowId(1));
        assert_eq!(z.len(), 3);
    }

    #[test]
    fn remove_window_also_removes_from_z_order() {
        let mut scene = WindowScene::new();
        scene.add_window(WindowId(1), size(100, 100));
        scene.add_window(WindowId(2), size(100, 100));
        scene.remove_window(WindowId(1));
        assert_eq!(scene.z_order(), &[WindowId(2)]);
    }

    #[test]
    fn commit_buffer_updates_window_pixels() {
        let mut scene = WindowScene::new();
        scene.add_window(WindowId(1), size(1, 1));
        let pixels = vec![255u8, 0, 0, 255];
        scene.commit_buffer(WindowId(1), pixels.clone());
        assert_eq!(scene.get(WindowId(1)).unwrap().buffer, pixels);
    }

    #[test]
    fn resize_window_updates_size() {
        let mut scene = WindowScene::new();
        scene.add_window(WindowId(1), size(100, 100));
        scene.resize_window(WindowId(1), size(200, 150));
        let w = scene.get(WindowId(1)).unwrap();
        assert_eq!(w.size.width, 200);
        assert_eq!(w.size.height, 150);
    }
}
