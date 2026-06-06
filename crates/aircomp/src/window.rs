use airproto::types::Size;

#[derive(Debug, Clone)]
pub struct WindowState {
    pub size: Size,
    pub buffer: Vec<u8>,
}

impl WindowState {
    pub fn new(size: Size) -> Self {
        let pixel_count = (size.width * size.height * 4) as usize;
        Self {
            size,
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
        let state = WindowState::new(Size {
            width: 2,
            height: 2,
        });
        assert_eq!(state.buffer.len(), 16); // 2*2*4
        assert!(state.buffer.iter().all(|&b| b == 0));
    }

    #[test]
    fn resize_clears_buffer() {
        let mut state = WindowState::new(Size {
            width: 2,
            height: 2,
        });
        state.commit_buffer(vec![1u8; 16]);
        state.resize(Size {
            width: 4,
            height: 4,
        });
        assert_eq!(state.buffer.len(), 64); // 4*4*4
        assert!(state.buffer.iter().all(|&b| b == 0));
    }

    #[test]
    fn commit_buffer_updates_pixels() {
        let mut state = WindowState::new(Size {
            width: 1,
            height: 1,
        });
        let pixels = vec![255u8, 0, 128, 255];
        state.commit_buffer(pixels.clone());
        assert_eq!(state.buffer, pixels);
    }

    #[test]
    fn commit_buffer_wrong_size_is_ignored() {
        let mut state = WindowState::new(Size {
            width: 1,
            height: 1,
        });
        let original = state.buffer.clone();
        state.commit_buffer(vec![1u8; 999]);
        assert_eq!(state.buffer, original);
    }
}
