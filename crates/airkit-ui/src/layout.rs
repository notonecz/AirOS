// crates/airkit-ui/src/layout.rs

use crate::primitives::{Padding, Rect, UiSize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub width: UiSize,
    pub height: UiSize,
    pub direction: Direction,
    pub padding: Padding,
    pub gap: f32,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new() -> Self {
        Self {
            width: UiSize::Fill,
            height: UiSize::Fill,
            direction: Direction::Column,
            padding: Padding::all(0.0),
            gap: 0.0,
            children: Vec::new(),
        }
    }

    pub fn with_size(width: UiSize, height: UiSize) -> Self {
        Self { width, height, ..Self::new() }
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn add_child(mut self, child: LayoutNode) -> Self {
        self.children.push(child);
        self
    }

    /// Computes Rect for this node and recursively for its children.
    /// Returns Vec<Rect> — first element is this node, then children in order (DFS).
    pub fn layout(&self, available: Rect) -> Vec<Rect> {
        let my_rect = self.resolve_rect(available);
        let mut result = vec![my_rect];

        if self.children.is_empty() {
            return result;
        }

        let inner = my_rect.inset(self.padding);
        let child_rects = self.layout_children(inner);

        for (child, child_rect) in self.children.iter().zip(child_rects.iter()) {
            result.extend(child.layout(*child_rect));
        }

        result
    }

    fn resolve_rect(&self, available: Rect) -> Rect {
        let w = match self.width {
            UiSize::Fill => available.width,
            UiSize::Fixed(v) => v,
        };
        let h = match self.height {
            UiSize::Fill => available.height,
            UiSize::Fixed(v) => v,
        };
        Rect::new(available.x, available.y, w, h)
    }

    fn layout_children(&self, inner: Rect) -> Vec<Rect> {
        if self.children.is_empty() {
            return Vec::new();
        }

        let n = self.children.len() as f32;
        let total_gap = self.gap * (n - 1.0).max(0.0);

        match self.direction {
            Direction::Column => {
                let fixed_height: f32 = self.children.iter().map(|c| match c.height {
                    UiSize::Fixed(v) => v,
                    UiSize::Fill => 0.0,
                }).sum();
                let fill_count = self.children.iter().filter(|c| c.height == UiSize::Fill).count() as f32;
                let fill_height = if fill_count > 0.0 {
                    ((inner.height - fixed_height - total_gap) / fill_count).max(0.0)
                } else { 0.0 };

                let mut y = inner.y;
                self.children.iter().map(|c| {
                    let h = match c.height {
                        UiSize::Fixed(v) => v,
                        UiSize::Fill => fill_height,
                    };
                    let r = Rect::new(inner.x, y, inner.width, h);
                    y += h + self.gap;
                    r
                }).collect()
            }
            Direction::Row => {
                let fixed_width: f32 = self.children.iter().map(|c| match c.width {
                    UiSize::Fixed(v) => v,
                    UiSize::Fill => 0.0,
                }).sum();
                let fill_count = self.children.iter().filter(|c| c.width == UiSize::Fill).count() as f32;
                let fill_width = if fill_count > 0.0 {
                    ((inner.width - fixed_width - total_gap) / fill_count).max(0.0)
                } else { 0.0 };

                let mut x = inner.x;
                self.children.iter().map(|c| {
                    let w = match c.width {
                        UiSize::Fixed(v) => v,
                        UiSize::Fill => fill_width,
                    };
                    let r = Rect::new(x, inner.y, w, inner.height);
                    x += w + self.gap;
                    r
                }).collect()
            }
        }
    }
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available() -> Rect {
        Rect::from_size(400.0, 300.0)
    }

    #[test]
    fn fill_node_takes_all_available_space() {
        let node = LayoutNode::new();
        let rects = node.layout(available());
        assert!((rects[0].width - 400.0).abs() < f32::EPSILON);
        assert!((rects[0].height - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fixed_node_uses_fixed_size() {
        let node = LayoutNode::with_size(UiSize::Fixed(100.0), UiSize::Fixed(50.0));
        let rects = node.layout(available());
        assert!((rects[0].width - 100.0).abs() < f32::EPSILON);
        assert!((rects[0].height - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn column_two_fill_children_split_equally() {
        let node = LayoutNode::new()
            .with_direction(Direction::Column)
            .add_child(LayoutNode::new())
            .add_child(LayoutNode::new());

        let rects = node.layout(Rect::from_size(200.0, 100.0));
        // rects[0] = parent, rects[1] = child1, rects[2] = child2
        assert!((rects[1].height - 50.0).abs() < f32::EPSILON);
        assert!((rects[2].height - 50.0).abs() < f32::EPSILON);
        assert!((rects[2].y - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn column_gap_offsets_second_child() {
        let node = LayoutNode::new()
            .with_direction(Direction::Column)
            .with_gap(10.0)
            .add_child(LayoutNode::with_size(UiSize::Fill, UiSize::Fixed(40.0)))
            .add_child(LayoutNode::with_size(UiSize::Fill, UiSize::Fixed(40.0)));

        let rects = node.layout(Rect::from_size(200.0, 200.0));
        // child2.y = child1.y + child1.height + gap = 0 + 40 + 10 = 50
        assert!((rects[2].y - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn row_two_fill_children_split_equally() {
        let node = LayoutNode::new()
            .with_direction(Direction::Row)
            .add_child(LayoutNode::new())
            .add_child(LayoutNode::new());

        let rects = node.layout(Rect::from_size(200.0, 100.0));
        assert!((rects[1].width - 100.0).abs() < f32::EPSILON);
        assert!((rects[2].width - 100.0).abs() < f32::EPSILON);
        assert!((rects[2].x - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn padding_reduces_children_area() {
        let node = LayoutNode::new()
            .with_padding(Padding::all(20.0))
            .with_direction(Direction::Column)
            .add_child(LayoutNode::new());

        let rects = node.layout(Rect::from_size(200.0, 100.0));
        // child is inside padding: x=20, y=20, width=160, height=60
        assert!((rects[1].x - 20.0).abs() < f32::EPSILON);
        assert!((rects[1].y - 20.0).abs() < f32::EPSILON);
        assert!((rects[1].width - 160.0).abs() < f32::EPSILON);
        assert!((rects[1].height - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn no_children_returns_single_rect() {
        let node = LayoutNode::new();
        let rects = node.layout(available());
        assert_eq!(rects.len(), 1);
    }
}
