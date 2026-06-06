use crate::rect_pipeline::ColoredRect;

// Shell chrome barvy (macOS-inspired dark palette)
const COLOR_TOPBAR: [f32; 4] = [0.110, 0.110, 0.118, 1.0]; // #1C1C1E
const COLOR_DOCK_BG: [f32; 4] = [0.173, 0.173, 0.180, 0.92]; // #2C2C2E @92%
const COLOR_DOCK_ICON: [f32; 4] = [0.220, 0.220, 0.227, 1.0]; // #383838

const TOPBAR_H: f32 = 28.0;
const DOCK_W: f32 = 232.0;
const DOCK_H: f32 = 56.0;
const DOCK_MARGIN_BOTTOM: f32 = 8.0;
const DOCK_ICON_SIZE: f32 = 42.0;
const DOCK_ICON_GAP: f32 = 8.0;
const DOCK_ICON_SIDE_PAD: f32 = 20.0;

/// Vypočítá seznam obdélníků pro shell chrome.
///
/// Pořadí: topbar, dock bg, 4× dock icon placeholder (celkem 6).
/// Desktop fill (pozadí #141414) je řešen pomocí clear passus v `SurfaceRenderer::render_frame`.
pub fn compute_chrome(screen_w: u32, screen_h: u32) -> Vec<ColoredRect> {
    let sw = screen_w as f32;
    let sh = screen_h as f32;

    let dock_x = (sw - DOCK_W) / 2.0;
    let dock_y = sh - DOCK_H - DOCK_MARGIN_BOTTOM;
    let icon_y = dock_y + (DOCK_H - DOCK_ICON_SIZE) / 2.0;

    let mut rects = vec![
        // 0: topbar
        ColoredRect {
            x: 0.0,
            y: 0.0,
            w: sw,
            h: TOPBAR_H,
            color: COLOR_TOPBAR,
        },
        // 1: dock bg
        ColoredRect {
            x: dock_x,
            y: dock_y,
            w: DOCK_W,
            h: DOCK_H,
            color: COLOR_DOCK_BG,
        },
    ];

    // 2–5: dock icon placeholders
    for i in 0..4u32 {
        let icon_x = dock_x + DOCK_ICON_SIDE_PAD + i as f32 * (DOCK_ICON_SIZE + DOCK_ICON_GAP);
        rects.push(ColoredRect {
            x: icon_x,
            y: icon_y,
            w: DOCK_ICON_SIZE,
            h: DOCK_ICON_SIZE,
            color: COLOR_DOCK_ICON,
        });
    }

    rects
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_chrome_returns_six_rects() {
        let rects = compute_chrome(1920, 1080);
        assert_eq!(rects.len(), 6);
    }

    #[test]
    fn topbar_spans_full_width() {
        let rects = compute_chrome(1920, 1080);
        let topbar = &rects[0];
        assert_eq!(topbar.x, 0.0);
        assert_eq!(topbar.y, 0.0);
        assert_eq!(topbar.w, 1920.0);
        assert_eq!(topbar.h, 28.0);
    }

    #[test]
    fn dock_bg_is_horizontally_centered() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        let expected_x = (1920.0 - 232.0) / 2.0;
        assert!((dock.x - expected_x).abs() < 0.5);
        assert_eq!(dock.w, 232.0);
        assert_eq!(dock.h, 56.0);
    }

    #[test]
    fn dock_bg_is_near_bottom() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        let expected_y = 1080.0 - 56.0 - 8.0;
        assert!((dock.y - expected_y).abs() < 0.5);
    }

    #[test]
    fn four_icon_rects_are_within_dock_bg() {
        let rects = compute_chrome(1920, 1080);
        let dock = &rects[1];
        for icon in &rects[2..6] {
            assert!(icon.x >= dock.x);
            assert!(icon.x + icon.w <= dock.x + dock.w + 0.5);
            assert!(icon.y >= dock.y);
            assert_eq!(icon.w, 42.0);
            assert_eq!(icon.h, 42.0);
        }
    }

    #[test]
    fn colored_rect_fields_accessible() {
        use crate::rect_pipeline::ColoredRect;
        let r = ColoredRect {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
            color: [1.0, 0.0, 0.0, 1.0],
        };
        assert_eq!(r.x, 1.0);
        assert_eq!(r.color[0], 1.0);
    }
}
