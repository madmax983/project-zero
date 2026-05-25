// src/shared/rendering.rs

use ratatui::style::Color;
use std::collections::BTreeMap;

/// Defines the z-order for rendering.
/// Lower layers render first (back to front).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum RenderLayer {
    /// Terrain tiles (grass, dirt, rock, water).
    Terrain = 0,

    /// Resources on tiles (trees, ore deposits - future).
    Resources = 1,

    /// Buildings (housing, farms, mines).
    Buildings = 2,

    /// Pops and other entities.
    Entities = 3,

    /// Cursor, selection highlights, designations.
    Cursor = 4,

    /// Modal overlays (chronicle, help, dialogs) - blocks everything below.
    Overlay = 5,
}

/// Represents a single renderable item.
#[derive(Clone, Debug)]
pub enum RenderItem {
    /// A single character at (x, y).
    Char(char, Color, i32, i32),

    /// A filled rectangle (for backgrounds, modals).
    Rect(Color, i32, i32, u16, u16), // color, x, y, width, height
}

impl RenderItem {
    #[must_use]
    pub fn x(&self) -> i32 {
        match self {
            Self::Char(_, _, x, _) | Self::Rect(_, x, _, _, _) => *x,
        }
    }

    #[must_use]
    pub fn y(&self) -> i32 {
        match self {
            Self::Char(_, _, _, y) | Self::Rect(_, _, y, _, _) => *y,
        }
    }

    #[must_use]
    pub fn char(&self) -> char {
        match self {
            Self::Char(c, _, _, _) => *c,
            Self::Rect(_, _, _, _, _) => ' ',
        }
    }

    #[must_use]
    pub fn color(&self) -> Color {
        match self {
            Self::Char(_, color, _, _) | Self::Rect(color, _, _, _, _) => *color,
        }
    }
}

/// Accumulates render items across layers, then renders in z-order.
#[derive(Default)]
pub struct RenderBuffer {
    layers: BTreeMap<RenderLayer, Vec<RenderItem>>,
}

impl RenderBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a render item to a specific layer.
    pub fn add(&mut self, layer: RenderLayer, item: RenderItem) {
        self.layers.entry(layer).or_default().push(item);
    }

    /// Clear all render items.
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Get all items sorted by layer (back to front).
    #[must_use]
    pub fn sorted_items(&self) -> Vec<(RenderLayer, &RenderItem)> {
        let mut items = Vec::new();
        for (layer, layer_items) in &self.layers {
            for item in layer_items {
                items.push((*layer, item));
            }
        }
        items
    }

    /// Get the topmost (highest z-order) item at a position.
    #[must_use]
    pub fn top_item_at(&self, x: i32, y: i32) -> Option<&RenderItem> {
        // Iterate layers in reverse (front to back)
        for layer_items in self.layers.values().rev() {
            for item in layer_items.iter().rev() {
                match item {
                    RenderItem::Char(_, _, char_x, char_y) => {
                        if *char_x == x && *char_y == y {
                            return Some(item);
                        }
                    }
                    RenderItem::Rect(_, rect_x, rect_y, w, h) => {
                        if x >= *rect_x
                            && x < *rect_x + (*w as i32)
                            && y >= *rect_y
                            && y < *rect_y + (*h as i32)
                        {
                            return Some(item);
                        }
                    }
                }
            }
        }
        None
    }

    #[must_use]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_layer_ordering() {
        // Verify layers render in correct z-order (back to front)
        assert!((RenderLayer::Terrain as u8) < RenderLayer::Resources as u8);
        assert!((RenderLayer::Resources as u8) < RenderLayer::Buildings as u8);
        assert!((RenderLayer::Buildings as u8) < RenderLayer::Entities as u8);
        assert!((RenderLayer::Entities as u8) < RenderLayer::Cursor as u8);
        assert!((RenderLayer::Cursor as u8) < RenderLayer::Overlay as u8);
    }

    #[test]
    fn test_render_buffer_layer_collection() {
        let mut buffer = RenderBuffer::new();

        // Add items to different layers
        buffer.add(
            RenderLayer::Terrain,
            RenderItem::Char('.', ratatui::style::Color::Green, 5, 5),
        );
        buffer.add(
            RenderLayer::Entities,
            RenderItem::Char('☺', ratatui::style::Color::Yellow, 5, 5),
        );
        buffer.add(
            RenderLayer::Cursor,
            RenderItem::Char('X', ratatui::style::Color::Red, 5, 5),
        );

        // Should have 3 layers with items
        assert_eq!(buffer.layer_count(), 3);
    }

    #[test]
    fn test_render_buffer_sorts_by_layer() {
        let mut buffer = RenderBuffer::new();

        // Add in random order
        buffer.add(
            RenderLayer::Cursor,
            RenderItem::Char('C', ratatui::style::Color::Red, 0, 0),
        );
        buffer.add(
            RenderLayer::Terrain,
            RenderItem::Char('T', ratatui::style::Color::Green, 0, 0),
        );
        buffer.add(
            RenderLayer::Entities,
            RenderItem::Char('E', ratatui::style::Color::Yellow, 0, 0),
        );

        let sorted = buffer.sorted_items();

        // Should be sorted: Terrain, Entities, Cursor
        assert_eq!(sorted[0].1.char(), 'T');
        assert_eq!(sorted[1].1.char(), 'E');
        assert_eq!(sorted[2].1.char(), 'C');
    }

    #[test]
    fn test_render_item_at_position() {
        let mut buffer = RenderBuffer::new();

        buffer.add(
            RenderLayer::Terrain,
            RenderItem::Char('.', ratatui::style::Color::Green, 10, 5),
        );
        buffer.add(
            RenderLayer::Entities,
            RenderItem::Char('☺', ratatui::style::Color::Yellow, 10, 5),
        );

        // Top layer at position should be entity (higher z)
        let top = buffer.top_item_at(10, 5);
        assert!(top.is_some());
        assert_eq!(top.unwrap().char(), '☺');
    }

    #[test]
    fn test_render_item_empty_position() {
        let buffer = RenderBuffer::new();

        let top = buffer.top_item_at(100, 100);
        assert!(top.is_none());
    }

    #[test]
    fn test_render_buffer_clear() {
        let mut buffer = RenderBuffer::new();

        buffer.add(
            RenderLayer::Terrain,
            RenderItem::Char('.', ratatui::style::Color::Green, 0, 0),
        );
        assert_eq!(buffer.layer_count(), 1);

        buffer.clear();
        assert_eq!(buffer.layer_count(), 0);
    }

    #[test]
    fn test_overlay_blocks_lower_layers() {
        let mut buffer = RenderBuffer::new();

        // Add items on all layers
        buffer.add(
            RenderLayer::Terrain,
            RenderItem::Char('.', ratatui::style::Color::Green, 5, 5),
        );
        buffer.add(
            RenderLayer::Entities,
            RenderItem::Char('☺', ratatui::style::Color::Yellow, 5, 5),
        );
        buffer.add(
            RenderLayer::Overlay,
            RenderItem::Rect(ratatui::style::Color::DarkGray, 0, 0, 20, 20),
        );

        // Overlay should be on top
        let layers = buffer.sorted_items();
        assert_eq!(layers.last().unwrap().0, RenderLayer::Overlay);
    }

    #[test]
    fn test_render_item_variants() {
        let char_item = RenderItem::Char('A', ratatui::style::Color::Red, 0, 0);
        let rect_item = RenderItem::Rect(ratatui::style::Color::Blue, 5, 5, 10, 10);

        assert_eq!(char_item.char(), 'A');
        assert_eq!(char_item.x(), 0);
        assert_eq!(char_item.y(), 0);

        assert_eq!(rect_item.x(), 5);
        assert_eq!(rect_item.y(), 5);
    }

    #[test]
    fn test_render_item_rect_bounds() {
        let mut buffer = RenderBuffer::new();
        buffer.add(
            RenderLayer::Overlay,
            RenderItem::Rect(ratatui::style::Color::DarkGray, 5, 5, 10, 10),
        );

        // Inside rect
        let top = buffer.top_item_at(10, 10);
        assert!(top.is_some());

        // Outside rect
        let top2 = buffer.top_item_at(0, 0);
        assert!(top2.is_none());

        // Edges
        assert!(buffer.top_item_at(5, 5).is_some());
        assert!(buffer.top_item_at(14, 14).is_some());
        assert!(buffer.top_item_at(15, 15).is_none());
    }
}
