# 002: Terrain Grid and Viewport

## Overview

Generate a tile-based terrain map and render it to the terminal with a scrollable viewport. This creates the spatial foundation for the colony simulation where all entities and buildings will be placed.

## Dependencies

- `001` — Project scaffold must exist

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/terrain.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_type_chars() {
        assert_eq!(TerrainType::Grass.char(), '.');
        assert_eq!(TerrainType::Dirt.char(), ',');
        assert_eq!(TerrainType::Rock.char(), '#');
        assert_eq!(TerrainType::Water.char(), '~');
    }

    #[test]
    fn test_terrain_type_colors() {
        use ratatui::style::Color;
        assert_eq!(TerrainType::Grass.color(), Color::Green);
        assert_eq!(TerrainType::Dirt.color(), Color::Rgb(139, 90, 43));
        assert_eq!(TerrainType::Rock.color(), Color::DarkGray);
        assert_eq!(TerrainType::Water.color(), Color::Blue);
    }

    #[test]
    fn test_terrain_type_as_str() {
        assert_eq!(TerrainType::Grass.as_str(), "Grass");
        assert_eq!(TerrainType::Dirt.as_str(), "Dirt");
        assert_eq!(TerrainType::Rock.as_str(), "Rock");
        assert_eq!(TerrainType::Water.as_str(), "Water");
    }

    #[test]
    fn test_terrain_grid_get_valid() {
        let grid = TerrainGrid {
            width: 3,
            height: 2,
            tiles: vec![
                TerrainType::Grass, TerrainType::Dirt, TerrainType::Rock,
                TerrainType::Water, TerrainType::Grass, TerrainType::Dirt,
            ],
        };

        assert_eq!(grid.get(0, 0), Some(TerrainType::Grass));
        assert_eq!(grid.get(1, 0), Some(TerrainType::Dirt));
        assert_eq!(grid.get(2, 0), Some(TerrainType::Rock));
        assert_eq!(grid.get(0, 1), Some(TerrainType::Water));
        assert_eq!(grid.get(2, 1), Some(TerrainType::Dirt));
    }

    #[test]
    fn test_terrain_grid_get_out_of_bounds() {
        let grid = TerrainGrid {
            width: 3,
            height: 2,
            tiles: vec![TerrainType::Grass; 6],
        };

        assert_eq!(grid.get(3, 0), None); // x out of bounds
        assert_eq!(grid.get(0, 2), None); // y out of bounds
        assert_eq!(grid.get(100, 100), None);
    }

    #[test]
    fn test_terrain_grid_bounds_checking() {
        let grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };

        // Test all edges
        assert!(grid.get(0, 0).is_some());
        assert!(grid.get(9, 0).is_some());
        assert!(grid.get(0, 9).is_some());
        assert!(grid.get(9, 9).is_some());

        // Just outside bounds
        assert!(grid.get(10, 0).is_none());
        assert!(grid.get(0, 10).is_none());
    }

    #[test]
    fn test_terrain_generation() {
        let grid = generate_terrain(80, 50);

        assert_eq!(grid.width, 80);
        assert_eq!(grid.height, 50);
        assert_eq!(grid.tiles.len(), 80 * 50);

        // Should have variety (not all grass)
        let has_grass = grid.tiles.iter().any(|&t| t == TerrainType::Grass);
        let has_dirt = grid.tiles.iter().any(|&t| t == TerrainType::Dirt);
        let has_rock = grid.tiles.iter().any(|&t| t == TerrainType::Rock);
        let has_water = grid.tiles.iter().any(|&t| t == TerrainType::Water);

        assert!(has_grass, "Generated terrain should include grass");
        assert!(has_dirt, "Generated terrain should include dirt");
        assert!(has_rock, "Generated terrain should include rock");
        assert!(has_water, "Generated terrain should include water");
    }

    #[test]
    fn test_fill_circle_clipping() {
        let mut tiles = vec![TerrainType::Grass; 10 * 10];

        // Circle partially outside grid should not panic
        fill_circle(&mut tiles, 10, 10, 0, 0, 5, TerrainType::Water);
        fill_circle(&mut tiles, 10, 10, 9, 9, 5, TerrainType::Rock);

        // Center should be affected
        assert_eq!(tiles[0], TerrainType::Water);
        assert_eq!(tiles[99], TerrainType::Rock);
    }

    #[test]
    fn test_viewport_default() {
        let viewport = Viewport::default();
        assert_eq!(viewport.x, 0);
        assert_eq!(viewport.y, 0);
    }

    #[test]
    fn test_viewport_movement() {
        let mut viewport = Viewport::default();

        viewport.x += 5;
        viewport.y += 3;
        assert_eq!(viewport.x, 5);
        assert_eq!(viewport.y, 3);

        viewport.x -= 2;
        viewport.y -= 1;
        assert_eq!(viewport.x, 3);
        assert_eq!(viewport.y, 2);
    }

    #[test]
    fn test_viewport_negative_coords() {
        let mut viewport = Viewport::default();

        // Viewport can have negative coordinates (shows empty space)
        viewport.x = -5;
        viewport.y = -10;
        assert_eq!(viewport.x, -5);
        assert_eq!(viewport.y, -10);
    }

    #[test]
    fn test_viewport_rendering_offsets() {
        let grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        let viewport = Viewport { x: 5, y: 3 };

        // When rendering at screen position (0, 0), should look at grid (5, 3)
        let world_x = viewport.x + 0;
        let world_y = viewport.y + 0;
        assert_eq!(grid.get(world_x as usize, world_y as usize), Some(TerrainType::Grass));
    }

    #[test]
    fn test_terrain_type_is_copy() {
        let t1 = TerrainType::Grass;
        let t2 = t1; // Should copy, not move
        assert_eq!(t1, t2);
    }
}
```

**Test Coverage Requirements:**
- TerrainType: all methods (char, color, as_str), all variants
- TerrainGrid: valid access, out-of-bounds, edge cases, row-major indexing
- Viewport: default, movement, negative coordinates
- Generation: produces variety, correct dimensions
- fill_circle: bounds clipping works correctly
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Add Dependency

```toml
# Cargo.toml
[dependencies]
rand = "0.8"
```

### TerrainType

```rust
// src/layer1/terrain.rs

use ratatui::style::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainType {
    Grass,
    Dirt,
    Rock,
    Water,
}

impl TerrainType {
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Grass => '.',
            Self::Dirt => ',',
            Self::Rock => '#',
            Self::Water => '~',
        }
    }

    #[must_use]
    pub const fn color(&self) -> Color {
        match self {
            Self::Grass => Color::Green,
            Self::Dirt => Color::Rgb(139, 90, 43),
            Self::Rock => Color::DarkGray,
            Self::Water => Color::Blue,
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::Dirt => "Dirt",
            Self::Rock => "Rock",
            Self::Water => "Water",
        }
    }
}
```

### TerrainGrid Resource

```rust
// src/layer1/terrain.rs

use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct TerrainGrid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TerrainType>, // row-major: index = y * width + x
}

impl TerrainGrid {
    /// Get terrain at (x, y). Returns None if out of bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<TerrainType> {
        if x < self.width && y < self.height {
            Some(self.tiles[y * self.width + x])
        } else {
            None
        }
    }
}
```

### Viewport Resource

```rust
// src/layer1/terrain.rs

#[derive(Resource, Default)]
pub struct Viewport {
    /// Top-left corner x coordinate in grid space.
    /// Can be negative (shows empty space to the left).
    pub x: i32,
    /// Top-left corner y coordinate in grid space.
    /// Can be negative (shows empty space above).
    pub y: i32,
}
```

### Terrain Generation

```rust
// src/layer1/terrain.rs

/// Generate a random terrain grid with variety.
#[must_use]
pub fn generate_terrain(width: usize, height: usize) -> TerrainGrid {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut tiles = vec![TerrainType::Grass; width * height];

    // Scatter dirt patches
    for _ in 0..50 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(2..6);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Dirt);
    }

    // Scatter rock
    for _ in 0..30 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(1..4);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Rock);
    }

    // Water features (rivers/lakes)
    for _ in 0..10 {
        let cx = rng.gen_range(0..width);
        let cy = rng.gen_range(0..height);
        let radius = rng.gen_range(3..8);
        fill_circle(&mut tiles, width, height, cx, cy, radius, TerrainType::Water);
    }

    TerrainGrid { width, height, tiles }
}

fn fill_circle(
    tiles: &mut [TerrainType],
    w: usize,
    h: usize,
    cx: usize,
    cy: usize,
    r: usize,
    t: TerrainType,
) {
    let r2 = (r * r) as i32;
    for dy in -(r as i32)..=(r as i32) {
        for dx in -(r as i32)..=(r as i32) {
            if dx * dx + dy * dy <= r2 {
                let x = cx as i32 + dx;
                let y = cy as i32 + dy;
                if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                    tiles[y as usize * w + x as usize] = t;
                }
            }
        }
    }
}
```

### Rendering

```rust
// src/layer1/terrain.rs

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// Render terrain grid to the given frame area with viewport offset.
pub fn render_terrain(frame: &mut Frame, area: Rect, terrain: &TerrainGrid, viewport: &Viewport) {
    let mut lines: Vec<Line> = Vec::new();

    for screen_y in 0..area.height {
        let world_y = viewport.y + screen_y as i32;
        let mut line_spans = Vec::new();

        for screen_x in 0..area.width {
            let world_x = viewport.x + screen_x as i32;

            let (ch, color) = if world_x >= 0 && world_y >= 0 {
                if let Some(tile) = terrain.get(world_x as usize, world_y as usize) {
                    (tile.char(), tile.color())
                } else {
                    (' ', Color::Black) // Out of bounds
                }
            } else {
                (' ', Color::Black) // Negative coordinates
            };

            line_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
        lines.push(Line::from(line_spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
```

### Module Integration

```rust
// src/layer1/mod.rs
pub mod terrain;

pub use terrain::{TerrainGrid, TerrainType, Viewport, generate_terrain, render_terrain};
```

### Main.rs Integration

```rust
// src/main.rs - Add these changes

use scale::layer1::{TerrainGrid, Viewport, generate_terrain, render_terrain};

fn main() -> anyhow::Result<()> {
    // ... terminal setup ...

    // ECS setup
    let mut world = World::new();
    world.insert_resource(GameState::Running);
    world.insert_resource(generate_terrain(80, 50)); // ADD THIS
    world.insert_resource(Viewport::default());      // ADD THIS

    // ... rest of main loop ...
}

fn render(world: &World, frame: &mut Frame) {
    let area = frame.area();

    let block = Block::default()
        .title(" SCALE ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // ADD THESE LINES:
    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    render_terrain(frame, inner_area, terrain, viewport);
}

// ADD TO handle_input():
fn handle_input(world: &mut World, key: crossterm::event::KeyEvent) {
    match key.code {
        // ... existing q, Esc, Space handlers ...

        // ADD VIEWPORT MOVEMENT:
        KeyCode::Char('w') | KeyCode::Up => {
            world.resource_mut::<Viewport>().y -= 1;
        }
        KeyCode::Char('s') | KeyCode::Down => {
            world.resource_mut::<Viewport>().y += 1;
        }
        KeyCode::Char('a') | KeyCode::Left => {
            world.resource_mut::<Viewport>().x -= 1;
        }
        KeyCode::Char('d') | KeyCode::Right => {
            world.resource_mut::<Viewport>().x += 1;
        }
        _ => {}
    }
}
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Magic numbers in generation**: 50 dirt patches, 30 rock patches, 10 water features
   - Future: Extract to constants or generation config
   - Current values produce reasonable variety for MVP

2. **fill_circle is O(r²)**: Inefficient for large radii
   - Future: Use midpoint circle algorithm
   - Current approach simple and correct, radii are small (<10)

3. **No terrain generation seed**: Maps are random each run
   - Future: Add seed parameter for reproducibility
   - Current random generation is fine for testing

4. **Viewport unbounded**: Can scroll infinitely into negative/beyond grid
   - Future: Add optional clamping or wraparound
   - Current behavior shows empty space (intentional for now)

5. **Rendering creates many small allocations**: Vec<Span> per line
   - Future: Reuse buffers or use custom widget
   - Current approach readable, terminal rendering not bottleneck

### Performance Considerations

- **Row-major indexing**: `y * width + x` is cache-friendly
- **Generation is one-time**: Only runs at startup, O(n) acceptable
- **Rendering is per-frame**: O(viewport area), not O(grid size) - good!
- **fill_circle overlaps**: Later circles overwrite earlier - intentional for layering

### API Design Notes

- `TerrainGrid::get()` returns Option - forces callers to handle bounds
- Viewport uses i32 not usize - allows negative coordinates for empty space
- TerrainType is Copy - cheap to pass, no borrow conflicts
- Rendering takes `&TerrainGrid` not `&mut` - immutable during render

### Future Extensibility

When adding underground layers:
- Change `tiles: Vec<TerrainType>` to `tiles: Vec<Vec<TerrainType>>`
- Add `depth: usize` to Viewport
- Update `get()` to take z coordinate

When adding tile resources:
- Create separate `ResourceGrid` with `Option<ResourceType>`
- Overlay resources during rendering
- Keep terrain and resources decoupled

When adding tile selection:
- Add `cursor: Option<(usize, usize)>` to Viewport
- Render cursor highlight in separate pass
- Future spec will define selection system

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer1/terrain.rs
- [x] Running shows colored terrain grid with variety
- [x] At least 3 terrain types visible on screen
- [x] WASD scrolls the viewport (verified via tests)
- [x] Arrow keys also scroll (verified via tests)
- [x] Can scroll to edges showing empty space (negative coords work)
- [x] TerrainGrid resource is queryable via World
- [x] Viewport negative coordinates render as empty space

## Technical Guidance

### Row-Major Indexing

Grid uses row-major layout: `index = y * width + x`

```
Grid 3x2:
[0,0] [1,0] [2,0]   =>  [0, 1, 2, 3, 4, 5]
[0,1] [1,1] [2,1]
```

This is cache-friendly for row-by-row iteration (which rendering does).

### Viewport Coordinate System

- World space: Grid coordinates (0..width, 0..height)
- Screen space: Terminal cell coordinates (0..area.width, 0..area.height)
- Conversion: `world_pos = viewport.pos + screen_pos`

Negative viewport coordinates are valid (show empty space).

### Rendering Empty Space

When `world_x < 0` or `world_y < 0` or out of grid bounds:
- Render black space character
- This gives visual feedback when scrolled beyond map edges

### Common Pitfalls

1. **Off-by-one in bounds**: Use `<` not `<=` for width/height checks
2. **Mixing signed/unsigned**: Viewport is i32, grid indices are usize - convert carefully
3. **Forgetting inner_area**: Must render terrain inside block.inner(), not full area
4. **WASD conflicts**: Future specs may add build mode - input routing spec needed

## Questions

*Builder: add questions here if spec is unclear.*
