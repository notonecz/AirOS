use airproto::types::{Point, Size};

#[derive(Debug, Clone)]
pub struct WindowState {
    pub size: Size,
    pub position: Point,
    pub buffer: Vec<u8>,
}

impl WindowState {
    pub fn new(size: Size, position: Point) -> Self {
        let pixel_count = (size.width * size.height * 4) as usize;
        Self {
            size,
            position,
            buffer: vec![0u8; pixel_count],
        }
    }

    pub fn resize(&mut self, new_size: Size) {
        self.size = new_size;
        let pixel_count = (new_size.width * new_size.height * 4) as usize;
        self.buffer = vec![0u8; pixel_count];
    }

    pub fn commit_buffer(&mut self, data: Vec<u8>) {
        let expected = (self.size.width * self.size.height * 4) as usize;
        if data.len() == expected {
            self.buffer = data;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_window_has_zeroed_buffer() {
        let state = WindowState::new(
            Size { width: 2, height: 2 },
            Point { x: 0.0, y: 0.0 },
        );
        assert_eq!(state.buffer.len(), 16);
        assert!(state.buffer.iter().all(|&b| b == 0));
    }

    #[test]
    fn resize_clears_buffer() {
        let mut state = WindowState::new(
            Size { width: 2, height: 2 },
            Point { x: 0.0, y: 0.0 },
        );
        state.commit_buffer(vec![1u8; 16]);
        state.resize(Size { width: 4, height: 4 });
        assert_eq!(state.buffer.len(), 64);
        assert!(state.buffer.iter().all(|&b| b == 0));
    }

    #[test]
    fn commit_buffer_updates_pixels() {
        let mut state = WindowState::new(
            Size { width: 1, height: 1 },
            Point { x: 0.0, y: 0.0 },
        );
        let pixels = vec![255u8, 0, 128, 255];
        state.commit_buffer(pixels.clone());
        assert_eq!(state.buffer, pixels);
    }

    #[test]
    fn commit_buffer_wrong_size_is_ignored() {
        let mut state = WindowState::new(
            Size { width: 1, height: 1 },
            Point { x: 0.0, y: 0.0 },
        );
        let original = state.buffer.clone();
        state.commit_buffer(vec![1u8; 999]);
        assert_eq!(state.buffer, original);
    }

    #[test]
    fn window_state_stores_position() {
        let state = WindowState::new(
            Size { width: 640, height: 480 },
            Point { x: 100.0, y: 200.0 },
        );
        assert!((state.position.x - 100.0).abs() < f32::EPSILON);
        assert!((state.position.y - 200.0).abs() < f32::EPSILON);
    }
}
