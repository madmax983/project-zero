# 014: Rendering Architecture and Z-Order

## Overview

Establish a formal rendering pipeline with defined z-order layers for terrain, entities, buildings, UI overlays, and cursor. This prevents visual glitches where elements render in the wrong order (e.g., cursor under terrain, pops behind buildings) and provides a foundation for modal overlays and complex UI.

## Dependencies

- `002` — Terrain rendering
- `003` — UI layout (multi-panel rendering)
- `004` — Pop rendering
- `006` — Building placement (cursor)
- `010` — Chronicle (modal overlay)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/shared/rendering.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_layer_ordering() {
        // Verify layers render in correct z-order (back to front)
        assert!(RenderLayer::Terrain as u8 < RenderLayer::Resources as u8);
        assert!(RenderLayer::Resources as u8 < RenderLayer::Buildings as u8);
        assert!(RenderLayer::Buildings as u8 < RenderLayer::Entities as u8);
        assert!(RenderLayer::Entities as u8 < RenderLayer::Cursor as u8);
        assert!(RenderLayer::Cursor as u8 < RenderLayer::Overlay as u8);
    }

    #[test]
    fn test_render_buffer_layer_collection() {
        let mut buffer = RenderBuffer::new();

        // Add items to different layers
        buffer.add(RenderLayer::Terrain, RenderItem::Char('.', Color::Green, 5, 5));
        buffer.add(RenderLayer::Entities, RenderItem::Char('☺', Color::Yellow, 5, 5));
        buffer.add(RenderLayer::Cursor, RenderItem::Char('X', Color::Red, 5, 5));

        // Should have 3 layers with items
        assert_eq!(buffer.layer_count(), 3);
    }

    #[test]
    fn test_render_buffer_sorts_by_layer() {
        let mut buffer = RenderBuffer::new();

        // Add in random order
        buffer.add(RenderLayer::Cursor, RenderItem::Char('C', Color::Red, 0, 0));
        buffer.add(RenderLayer::Terrain, RenderItem::Char('T', Color::Green, 0, 0));
        buffer.add(RenderLayer::Entities, RenderItem::Char('E', Color::Yellow, 0, 0));

        let sorted = buffer.sorted_items();

        // Should be sorted: Terrain, Entities, Cursor
        assert_eq!(sorted[0].1.char(), 'T');
        assert_eq!(sorted[1].1.char(), 'E');
        assert_eq!(sorted[2].1.char(), 'C');
    }

    #[test]
    fn test_render_item_at_position() {
        let mut buffer = RenderBuffer::new();

        buffer.add(RenderLayer::Terrain, RenderItem::Char('.', Color::Green, 10, 5));
        buffer.add(RenderLayer::Entities, RenderItem::Char('☺', Color::Yellow, 10, 5));

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

        buffer.add(RenderLayer::Terrain, RenderItem::Char('.', Color::Green, 0, 0));
        assert_eq!(buffer.layer_count(), 1);

        buffer.clear();
        assert_eq!(buffer.layer_count(), 0);
    }

    #[test]
    fn test_overlay_blocks_lower_layers() {
        let mut buffer = RenderBuffer::new();

        // Add items on all layers
        buffer.add(RenderLayer::Terrain, RenderItem::Char('.', Color::Green, 5, 5));
        buffer.add(RenderLayer::Entities, RenderItem::Char('☺', Color::Yellow, 5, 5));
        buffer.add(RenderLayer::Overlay, RenderItem::Rect(Color::DarkGray, 0, 0, 20, 20));

        // Overlay should be on top
        let layers = buffer.sorted_items();
        assert_eq!(layers.last().unwrap().0, RenderLayer::Overlay);
    }

    #[test]
    fn test_render_item_variants() {
        let char_item = RenderItem::Char('A', Color::Red, 0, 0);
        let rect_item = RenderItem::Rect(Color::Blue, 5, 5, 10, 10);

        assert_eq!(char_item.char(), 'A');
        assert_eq!(char_item.x(), 0);
        assert_eq!(char_item.y(), 0);

        assert_eq!(rect_item.x(), 5);
        assert_eq!(rect_item.y(), 5);
    }
}
```

**Test Coverage Requirements:**
- RenderLayer: enum ordering, all layers defined
- RenderBuffer: add items, sorting by layer, clear
- RenderItem: position queries, char/rect variants
- Z-order: higher layers render on top
- Overlay blocking: overlay covers everything below
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Render Layers

```rust
// src/shared/rendering.rs

use ratatui::style::Color;

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
```

### Render Items

```rust
// src/shared/rendering.rs

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
```

### Render Buffer

```rust
// src/shared/rendering.rs

use std::collections::BTreeMap;

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
                if item.x() == x && item.y() == y {
                    return Some(item);
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
```

### Integration with Rendering

```rust
// src/main.rs - Modify render functions to use RenderBuffer

use scale::shared::rendering::{RenderBuffer, RenderLayer, RenderItem};

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Collect all render items into buffer
    let mut buffer = RenderBuffer::new();

    // Layer 0: Terrain
    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    render_terrain_to_buffer(&mut buffer, viewport, terrain, &inner);

    // Layer 2: Buildings (if spec 006 implemented)
    // render_buildings_to_buffer(&mut buffer, world, viewport, &inner);

    // Layer 3: Pops
    render_pops_to_buffer(&mut buffer, world, viewport, &inner);

    // Layer 4: Cursor (if build mode active)
    // render_cursor_to_buffer(&mut buffer, world, viewport, &inner);

    // Convert buffer to ratatui widgets and render
    render_buffer_to_frame(frame, inner, &buffer);
}

fn render_terrain_to_buffer(
    buffer: &mut RenderBuffer,
    viewport: &Viewport,
    terrain: &TerrainGrid,
    area: &Rect,
) {
    for screen_y in 0..area.height {
        let world_y = viewport.y + screen_y as i32;
        for screen_x in 0..area.width {
            let world_x = viewport.x + screen_x as i32;

            if world_x >= 0 && world_y >= 0 {
                if let Some(tile) = terrain.get(world_x as usize, world_y as usize) {
                    buffer.add(
                        RenderLayer::Terrain,
                        RenderItem::Char(tile.char(), tile.color(), world_x, world_y),
                    );
                }
            }
        }
    }
}

fn render_pops_to_buffer(
    buffer: &mut RenderBuffer,
    world: &World,
    viewport: &Viewport,
    _area: &Rect,
) {
    // Query all pops with GridPosition
    let mut query = world.query::<&GridPosition>().with::<Pop>();
    for pos in query.iter(world) {
        buffer.add(
            RenderLayer::Entities,
            RenderItem::Char('☺', Color::Yellow, pos.x, pos.y),
        );
    }
}

fn render_buffer_to_frame(frame: &mut Frame, area: Rect, buffer: &RenderBuffer) {
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;
    use std::collections::HashMap;

    // Build position map of top items
    let mut position_map: HashMap<(i32, i32), &RenderItem> = HashMap::new();

    for (_layer, item) in buffer.sorted_items() {
        // Later items (higher z) overwrite earlier items
        position_map.insert((item.x(), item.y()), item);
    }

    // Convert to ratatui Lines
    let mut lines = Vec::new();
    for screen_y in 0..area.height {
        let mut spans = Vec::new();
        for screen_x in 0..area.width {
            let world_x = screen_x as i32;
            let world_y = screen_y as i32;

            if let Some(item) = position_map.get(&(world_x, world_y)) {
                spans.push(Span::styled(
                    item.char().to_string(),
                    Style::default().fg(item.color()),
                ));
            } else {
                spans.push(Span::raw(" "));
            }
        }
        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
```

### Module Integration

```rust
// src/shared/mod.rs
pub mod time;
pub mod input;
pub mod schedule;
pub mod rendering; // ADD THIS
```

```rust
// src/lib.rs
pub use shared::rendering::{RenderBuffer, RenderLayer, RenderItem};
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Position map rebuild every frame**: HashMap allocation per frame
   - Future: Reuse buffer across frames
   - Current approach simple and clear

2. **No spatial indexing**: O(n) lookup for position queries
   - Future: Use quadtree or grid-based spatial index
   - Current linear search acceptable for <1000 entities

3. **No batching**: Each item rendered individually
   - Future: Batch by color/style for fewer draw calls
   - Current approach readable, terminal rendering not bottleneck

4. **Overlay doesn't actually block input**: Just visual layer
   - Future: Connect to InputContext system (spec 012)
   - Current visual-only overlay is fine

5. **No dirty rectangles**: Re-renders entire screen every frame
   - Future: Track changed regions, only redraw dirty areas
   - Current full redraw acceptable for terminal

### Performance Considerations

- **BTreeMap for layers**: Keeps layers sorted, O(log n) insert
- **Vec for items per layer**: Fast iteration, append-only
- **HashMap for position lookup**: O(1) position queries
- **Render buffer cleared each frame**: No memory accumulation

### API Design Notes

- RenderLayer is Ord - can sort, use as BTreeMap key
- RenderItem is cloneable - can cache, duplicate
- RenderBuffer owns items - no lifetime issues
- Layer-based API prevents z-fighting bugs

### Future Extensibility

When adding animations:
```rust
pub enum RenderItem {
    Char(char, Color, i32, i32),
    AnimatedChar(Vec<char>, Color, i32, i32, f32), // frames, fps
}
```

When adding sprites/images:
```rust
pub enum RenderItem {
    Char(...),
    Image(ImageId, Rect),
}
```

When adding dirty rectangles:
```rust
pub struct RenderBuffer {
    layers: BTreeMap<RenderLayer, Vec<RenderItem>>,
    dirty_regions: Vec<Rect>,
}
```

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for shared/rendering.rs
- [x] RenderLayer defines all z-order layers
- [x] RenderBuffer accumulates items by layer
- [x] Items render back-to-front (terrain under pops)
- [x] Position queries return topmost item
- [x] Overlay layer renders on top of everything
- [x] No z-fighting (items at same position have defined order)

## Technical Guidance

### Z-Order Layers (Back to Front)

```
Layer 0: Terrain     (., ,, #, ~)
Layer 1: Resources   (trees, ore - future)
Layer 2: Buildings   (⌂, ♣)
Layer 3: Entities    (☺)
Layer 4: Cursor      (X, selection highlights)
Layer 5: Overlay     (modal backgrounds, chrome)
```

### Rendering Pipeline

```
1. Clear buffer
2. Add terrain to Layer 0
3. Add buildings to Layer 2
4. Add pops to Layer 3
5. Add cursor to Layer 4 (if build mode)
6. Add overlay to Layer 5 (if modal open)
7. Sort items by layer
8. Build position map (higher layers overwrite)
9. Convert to ratatui Lines
10. Render to frame
```

### Position Map Behavior

At position (10, 5):
- Terrain adds: '.' green
- Pop adds: '☺' yellow
- Position map after sort: '☺' yellow (pop wins, higher z)

### Common Pitfalls

1. **Forgetting to clear buffer**: Items accumulate across frames
2. **Wrong layer**: Cursor on Terrain layer → invisible under pops
3. **Viewport offset**: Remember to add viewport.x/y to world coords
4. **Overlay not full-screen**: Rect must cover entire area

## Questions

*Builder: add questions here if spec is unclear.*

## Future Work

This spec intentionally leaves unimplemented:
- **Dirty rectangles** - Only redraw changed regions
- **Spatial indexing** - Fast position queries via quadtree
- **Render batching** - Group items by style for efficiency
- **Animation support** - Frame-based character cycling
- **Sprite/image rendering** - Beyond ASCII characters
