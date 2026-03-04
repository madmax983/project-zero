# 004: Pop Spawning and Rendering

## Overview

Pops are the colonists—individuals who work, eat, sleep, and die. This spec covers creating pop entities and rendering them on the map as characters.

## Dependencies

- `001` — Project scaffold
- `002` — Terrain grid (pops exist on tiles)
- `003` — UI layout (rendering context)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/pop.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_pop_component_exists() {
        let mut world = World::new();
        let entity = world.spawn(Pop).id();

        assert!(world.get::<Pop>(entity).is_some());
    }

    #[test]
    fn test_grid_position_creation() {
        let pos = GridPosition { x: 5, y: 10 };
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_grid_position_is_copy() {
        let pos1 = GridPosition { x: 3, y: 7 };
        let pos2 = pos1; // Should copy, not move
        assert_eq!(pos1.x, pos2.x);
        assert_eq!(pos1.y, pos2.y);
    }

    #[test]
    fn test_grid_position_negative_coords() {
        let pos = GridPosition { x: -5, y: -10 };
        assert_eq!(pos.x, -5);
        assert_eq!(pos.y, -10);
    }

    #[test]
    fn test_spawn_initial_pops_count() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5, "Should spawn exactly 5 pops");
    }

    #[test]
    fn test_spawn_initial_pops_have_positions() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let mut query = world.query::<(&Pop, &GridPosition)>();
        let all_have_positions = query.iter(&world).count() == 5;
        assert!(all_have_positions, "All pops should have GridPosition");
    }

    #[test]
    fn test_spawn_only_on_walkable_terrain() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let terrain = world.resource::<TerrainGrid>();
        for (_, pos) in world.query::<(&Pop, &GridPosition)>().iter(&world) {
            if let Some(tile_type) = terrain.get(pos.x as usize, pos.y as usize) {
                assert_ne!(tile_type, TerrainType::Water, "Pop spawned on water");
                assert_ne!(tile_type, TerrainType::Rock, "Pop spawned on rock");
            }
        }
    }

    #[test]
    fn test_spawn_within_terrain_bounds() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        for (_, pos) in world.query::<(&Pop, &GridPosition)>().iter(&world) {
            assert!(pos.x >= 0 && pos.x < 80, "Pop x out of bounds");
            assert!(pos.y >= 0 && pos.y < 50, "Pop y out of bounds");
        }
    }

    #[test]
    fn test_pop_char_and_color() {
        // Test helper function for rendering pops
        let (ch, color) = pop_display();
        assert_eq!(ch, '☺');
        assert_eq!(color, Color::Yellow);
    }
}
```

**Test Coverage Requirements:**
- Pop component can be spawned and queried
- GridPosition stores x/y coordinates (signed integers)
- GridPosition is Copy (no borrow issues)
- spawn_initial_pops creates exactly 5 pops
- All pops have both Pop and GridPosition components
- Pops only spawn on walkable terrain (Grass or Dirt)
- Pops spawn within terrain bounds
- Pop display character and color are correct
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Components

```rust
// src/layer1/pop.rs

use bevy_ecs::prelude::*;
use ratatui::style::Color;

/// Marker component for pop entities.
#[derive(Component)]
pub struct Pop;

/// Grid position in world space.
#[derive(Component, Clone, Copy, Debug)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}
```

### Spawning

```rust
// src/layer1/pop.rs

use super::terrain::{TerrainGrid, TerrainType};
use rand::Rng;

/// Spawn 5 initial pops at random walkable positions.
pub fn spawn_initial_pops(world: &mut World) {
    let terrain = world.resource::<TerrainGrid>();
    let mut rng = rand::thread_rng();
    let mut spawned = 0;

    while spawned < 5 {
        let x = rng.gen_range(0..terrain.width as i32);
        let y = rng.gen_range(0..terrain.height as i32);

        // Only spawn on walkable terrain
        if let Some(terrain_type) = terrain.get(x as usize, y as usize) {
            if terrain_type != TerrainType::Water && terrain_type != TerrainType::Rock {
                world.spawn((
                    Pop,
                    GridPosition { x, y },
                ));
                spawned += 1;
            }
        }
    }
}
```

### Pop Display Function

```rust
// src/layer1/pop.rs

/// Returns the character and color for rendering a pop.
#[must_use]
pub const fn pop_display() -> (char, Color) {
    ('☺', Color::Yellow)
}
```

### Rendering Integration

```rust
// src/layer1/terrain.rs - Modify render_terrain function

use std::collections::HashSet;
use super::pop::GridPosition;

/// Render terrain grid and pops to the given frame area with viewport offset.
pub fn render_terrain_and_pops(
    frame: &mut Frame,
    area: Rect,
    terrain: &TerrainGrid,
    viewport: &Viewport,
    pop_positions: &HashSet<(i32, i32)>,
) {
    let mut lines: Vec<Line> = Vec::new();

    for screen_y in 0..area.height {
        let world_y = viewport.y + screen_y as i32;
        let mut line_spans = Vec::new();

        for screen_x in 0..area.width {
            let world_x = viewport.x + screen_x as i32;

            // Check for pop first
            if pop_positions.contains(&(world_x, world_y)) {
                let (ch, color) = super::pop::pop_display();
                line_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                continue;
            }

            // Otherwise render terrain
            let (ch, color) = if world_x >= 0 && world_y >= 0 {
                if let Some(tile) = terrain.get(world_x as usize, world_y as usize) {
                    (tile.char(), tile.color())
                } else {
                    (' ', Color::Black)
                }
            } else {
                (' ', Color::Black)
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
pub mod pop;

pub use terrain::{TerrainGrid, TerrainType, Viewport, generate_terrain};
pub use pop::{Pop, GridPosition, spawn_initial_pops, pop_display};
```

### Main.rs Integration

```rust
// src/main.rs - Add these changes

use scale::layer1::{spawn_initial_pops, GridPosition, Pop};
use std::collections::HashSet;

fn main() -> anyhow::Result<()> {
    // ... terminal setup ...

    // ECS setup
    let mut world = World::new();
    world.insert_resource(GameState::Running);
    world.insert_resource(generate_terrain(80, 50));
    world.insert_resource(Viewport::default());
    world.insert_resource(SimulationTime::default());

    // ADD THIS:
    spawn_initial_pops(&mut world);

    // ... rest of main loop ...
}

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();

    // CHANGE THIS: Collect pop positions
    let pop_positions: HashSet<(i32, i32)> = world
        .query::<&GridPosition>()
        .iter(world)
        .map(|pos| (pos.x, pos.y))
        .collect();

    render_terrain_and_pops(frame, inner, terrain, viewport, &pop_positions);
}
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Magic number 5**: Pop count is hardcoded
   - Future: Make configurable or based on starting resources
   - Current: 5 is reasonable for testing the survival loop

2. **Infinite loop potential**: `while spawned < 5` could theoretically loop forever
   - If terrain is 100% water/rock, loop never terminates
   - Future: Add max attempts counter, fallback to any tile
   - Current: Generated terrain always has plenty of Grass/Dirt

3. **Pop rendering creates HashSet every frame**: O(n) construction
   - Future: Cache pop positions or use spatial index
   - Current: 5 pops is trivial, premature optimization

4. **Pop display is static**: All pops look identical
   - Future: Spec 005 will add needs-based visual feedback
   - Current: Single character/color sufficient for entity rendering

5. **No pop selection**: Can't tell pops apart
   - Future: Selection system spec will add cursor interaction
   - Current: Not in scope for basic spawning/rendering

### Performance Considerations

- **Spawning is one-time**: Random placement runs once at startup, O(1) amortized
- **HashSet lookup**: O(1) average case for position checking
- **Rendering overhead**: 5 pops negligible compared to terrain (80×50 = 4000 tiles)

### API Design Notes

- `Pop` is marker component - zero size, pure identity
- `GridPosition` uses i32 not usize - compatible with viewport's signed coordinates
- `spawn_initial_pops` takes `&mut World` - allows future expansion (e.g., RNG seed parameter)
- `pop_display()` is const - could be compile-time constant in future

### Future Extensibility

When adding needs (Spec 005):
- Change `pop_display()` to `pop_display(needs: &Needs)` → returns different chars/colors
- Update rendering to query `(&GridPosition, &Needs)` instead of just position
- Keep GridPosition separate from visual state

When adding movement (future spec):
- Add `Velocity` or `TargetPosition` component
- Create movement system that updates GridPosition each tick
- Rendering stays unchanged (just reads current position)

When adding pop selection (future spec):
- Add `Selected` marker component
- Modify rendering to highlight selected pops (e.g., background color)
- Info panel queries Selected pops for detail view

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for layer1/pop.rs
- [x] Running game shows 5 `☺` characters on the terrain
- [x] Pops only spawn on Grass or Dirt (not Water/Rock)
- [x] Pops render in yellow/visible color
- [x] Pops stay visible when scrolling viewport
- [x] Each pop has `Pop` and `GridPosition` components
- [x] Pops appear on top of terrain characters
- [x] Pop positions within terrain bounds (0..width, 0..height)

## Technical Guidance

### Rendering Layers

Z-order from bottom to top:
1. Terrain (base layer)
2. Pops (entities layer)
3. Buildings (future - spec 006)
4. Cursor/UI (future - spec 006)

Current implementation checks pops before terrain in the render loop.

### GridPosition vs Viewport Coordinates

- `GridPosition`: World-space coordinates (can be negative in theory, but spawning keeps positive)
- `Viewport`: Camera offset (i32, can be negative)
- Rendering: `screen_pos = grid_pos - viewport_pos`

### Walkable Terrain Definition

For this spec:
- Walkable: Grass, Dirt
- Not walkable: Water, Rock

Future specs may change this (e.g., Rock becomes walkable with tech, Water with boats).

### Common Pitfalls

1. **Forgetting to render pops**: Easy to spawn entities but forget to query them in render
2. **Wrong z-order**: If terrain checked before pops, pops render underneath (invisible)
3. **HashSet construction cost**: Don't worry about it until profile shows bottleneck
4. **Spawning out of bounds**: Test checks x/y < width/height, make sure casts are correct

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
