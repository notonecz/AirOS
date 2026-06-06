/// Barevný obdélník v pixelových souřadnicích (origin = levý horní roh obrazovky).
#[derive(Debug, Clone, Copy)]
pub struct ColoredRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// RGBA barva v rozsahu 0.0..=1.0.
    pub color: [f32; 4],
}
