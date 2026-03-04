# 015: Selection and Inspection System

## Overview

Allow players to click or hover over tiles, pops, and buildings to inspect their state. Selected entities display detailed information in the info panel (right side of UI). This creates the foundation for player interaction beyond placement and provides visibility into simulation state for debugging and strategy.

## Dependencies

- `002` — Terrain grid (tiles to select)
- `003` — UI layout (info panel to populate)
- `004` — Pop entities (pops to select)
- `006` — Building placement (buildings to select)
- `012` — Input architecture (mouse/keyboard context)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/shared/selection.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_selection_default_none() {
        let selection = Selection::default();
        assert_eq!(selection.target(), SelectionTarget::None);
    }

    #[test]
    fn test_selection_tile() {
        let mut selection = Selection::default();
        selection.select_tile(10, 5);

        assert_eq!(selection.target(), SelectionTarget::Tile(10, 5));
    }

    #[test]
    fn test_selection_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let mut selection = Selection::default();
        selection.select_entity(entity);

        assert_eq!(selection.target(), SelectionTarget::Entity(entity));
    }

    #[test]
    fn test_selection_clear() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let mut selection = Selection::default();
        selection.select_entity(entity);
        assert!(selection.is_selected());

        selection.clear();
        assert!(!selection.is_selected());
        assert_eq!(selection.target(), SelectionTarget::None);
    }

    #[test]
    fn test_selection_is_selected() {
        let mut selection = Selection::default();
        assert!(!selection.is_selected());

        selection.select_tile(0, 0);
        assert!(selection.is_selected());

        selection.clear();
        assert!(!selection.is_selected());
    }

    #[test]
    fn test_selection_get_tile() {
        let mut selection = Selection::default();
        selection.select_tile(10, 20);

        if let SelectionTarget::Tile(x, y) = selection.target() {
            assert_eq!(*x, 10);
            assert_eq!(*y, 20);
        } else {
            panic!("Expected tile selection");
        }
    }

    #[test]
    fn test_selection_get_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        let mut selection = Selection::default();
        selection.select_entity(entity);

        if let SelectionTarget::Entity(e) = selection.target() {
            assert_eq!(*e, entity);
        } else {
            panic!("Expected entity selection");
        }
    }

    #[test]
    fn test_selection_overwrite() {
        let mut selection = Selection::default();
        selection.select_tile(5, 5);
        assert!(selection.is_selected());

        // Selecting new target should overwrite
        selection.select_tile(10, 10);
        assert_eq!(selection.target(), SelectionTarget::Tile(10, 10));
    }

    #[test]
    fn test_inspect_tile_with_terrain() {
        let mut world = World::new();
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        world.insert_resource(terrain);

        let info = inspect_tile(&world, 5, 5);

        assert!(info.contains("Grass"));
        assert!(info.contains("5"));
    }

    #[test]
    fn test_inspect_tile_out_of_bounds() {
        let mut world = World::new();
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        world.insert_resource(terrain);

        let info = inspect_tile(&world, 100, 100);

        assert!(info.contains("Empty") || info.contains("Nothing"));
    }

    #[test]
    fn test_inspect_entity_pop() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            GridPosition { x: 10, y: 5 },
            Needs { hunger: 0.75, rest: 0.50 },
        )).id();

        let info = inspect_entity(&world, entity);

        assert!(info.contains("Pop"));
        assert!(info.contains("10"));
        assert!(info.contains("hunger") || info.contains("Hunger"));
    }

    #[test]
    fn test_inspect_entity_nonexistent() {
        let world = World::new();
        let fake_entity = Entity::from_raw(999999);

        let info = inspect_entity(&world, fake_entity);

        assert!(info.contains("not found") || info.contains("None"));
    }

    #[test]
    fn test_selection_target_is_copy() {
        let target1 = SelectionTarget::Tile(5, 5);
        let target2 = target1; // Should copy
        assert_eq!(target1, target2);
    }

    #[test]
    fn test_click_to_screen_pos() {
        let viewport = Viewport { x: 10, y: 5 };
        let screen_x = 3;
        let screen_y = 2;

        let (world_x, world_y) = screen_to_world(screen_x, screen_y, &viewport);

        assert_eq!(world_x, 13); // 10 + 3
        assert_eq!(world_y, 7);  // 5 + 2
    }
}
```

**Test Coverage Requirements:**
- Selection: select tile, entity, clear, overwrite
- SelectionTarget: all variants, type checking
- Inspection: tile info, entity info, out-of-bounds handling
- Coordinate conversion: screen → world with viewport offset
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Selection Types

```rust
// src/shared/selection.rs

use bevy_ecs::prelude::*;

/// What is currently selected by the player.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectionTarget {
    /// Nothing selected.
    None,

    /// A tile at grid position (x, y).
    Tile(i32, i32),

    /// An entity (pop, building, etc.).
    Entity(Entity),
}

/// Tracks the current player selection.
#[derive(Resource, Default, Debug)]
pub struct Selection {
    target: SelectionTarget,
}

impl Default for SelectionTarget {
    fn default() -> Self {
        Self::None
    }
}

impl Selection {
    #[must_use]
    pub fn new() -> Self {
        Self {
            target: SelectionTarget::None,
        }
    }

    /// Get the current selection target.
    #[must_use]
    pub fn target(&self) -> SelectionTarget {
        self.target
    }

    /// Select a tile at grid coordinates.
    pub fn select_tile(&mut self, x: i32, y: i32) {
        self.target = SelectionTarget::Tile(x, y);
    }

    /// Select an entity.
    pub fn select_entity(&mut self, entity: Entity) {
        self.target = SelectionTarget::Entity(entity);
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.target = SelectionTarget::None;
    }

    /// Check if anything is selected.
    #[must_use]
    pub fn is_selected(&self) -> bool {
        !matches!(self.target, SelectionTarget::None)
    }
}
```

### Inspection Functions

```rust
// src/shared/selection.rs

use crate::layer1::{TerrainGrid, TerrainType, Pop, GridPosition};
use crate::layer1::needs::Needs;

/// Generate info panel text for a selected tile.
#[must_use]
pub fn inspect_tile(world: &World, x: i32, y: i32) -> String {
    let terrain = world.resource::<TerrainGrid>();

    if x < 0 || y < 0 {
        return String::from("Empty space\n(outside map)");
    }

    if let Some(tile) = terrain.get(x as usize, y as usize) {
        format!(
            "Tile ({}, {})\n\nTerrain: {}\n\n(Click entity for details)",
            x, y, tile.as_str()
        )
    } else {
        String::from("Empty space\n(outside map)")
    }
}

/// Generate info panel text for a selected entity.
#[must_use]
pub fn inspect_entity(world: &World, entity: Entity) -> String {
    // Check if entity exists
    if !world.entities().contains(entity) {
        return String::from("Entity not found");
    }

    // Try to get Pop components
    if let Some(pos) = world.get::<GridPosition>(entity) {
        let mut info = format!("Pop\nPosition: ({}, {})\n\n", pos.x, pos.y);

        if let Some(needs) = world.get::<Needs>(entity) {
            info.push_str(&format!(
                "Hunger: {:.0}%\nRest: {:.0}%\n",
                needs.hunger * 100.0,
                needs.rest * 100.0
            ));
        }

        return info;
    }

    // Try to get Building components (future)
    // ...

    String::from("Unknown entity")
}

/// Convert screen coordinates to world coordinates.
#[must_use]
pub fn screen_to_world(screen_x: u16, screen_y: u16, viewport: &Viewport) -> (i32, i32) {
    let world_x = viewport.x + screen_x as i32;
    let world_y = viewport.y + screen_y as i32;
    (world_x, world_y)
}
```

### Input Handling

```rust
// src/shared/selection.rs

use crossterm::event::{KeyEvent, KeyCode, MouseEvent, MouseEventKind};

/// Handle selection input in normal mode.
pub fn handle_selection_input(
    world: &mut World,
    key: KeyEvent,
) {
    match key.code {
        KeyCode::Esc => {
            // Clear selection
            world.resource_mut::<Selection>().clear();
        }
        _ => {}
    }
}

/// Handle mouse click for selection.
/// Note: Terminal mouse support requires crossterm::event::EnableMouseCapture.
pub fn handle_selection_click(
    world: &mut World,
    mouse: MouseEvent,
    viewport: &Viewport,
) {
    if let MouseEventKind::Down(_button) = mouse.kind {
        let (world_x, world_y) = screen_to_world(mouse.column, mouse.row, viewport);

        // Check if clicking on an entity
        let mut selected_entity = None;
        let mut query = world.query::<(Entity, &GridPosition)>();
        for (entity, pos) in query.iter(world) {
            if pos.x == world_x && pos.y == world_y {
                selected_entity = Some(entity);
                break;
            }
        }

        // Update selection
        let mut selection = world.resource_mut::<Selection>();
        if let Some(entity) = selected_entity {
            selection.select_entity(entity);
        } else {
            selection.select_tile(world_x, world_y);
        }
    }
}
```

### Info Panel Rendering

```rust
// src/main.rs - Update render_info_panel

fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let selection = world.resource::<Selection>();

    let text_content = match selection.target() {
        SelectionTarget::None => {
            String::from("Select something\nto see info here")
        }
        SelectionTarget::Tile(x, y) => {
            inspect_tile(world, *x, *y)
        }
        SelectionTarget::Entity(entity) => {
            inspect_entity(world, *entity)
        }
    };

    let text = Paragraph::new(text_content);
    frame.render_widget(text, inner);
}
```

### Integration with Main

```rust
// src/main.rs

use scale::shared::selection::{Selection, handle_selection_input, handle_selection_click};

fn main() -> anyhow::Result<()> {
    // ... terminal setup ...

    let mut world = World::new();
    world.insert_resource(GameState::Running);
    world.insert_resource(generate_terrain(80, 50));
    world.insert_resource(Viewport::default());
    world.insert_resource(SimulationTime::default());
    world.insert_resource(InputContextStack::default());
    world.insert_resource(Selection::default()); // ADD THIS

    // ... main loop ...
}

// In handle_normal_mode (spec 012):
// Add Esc to clear selection
KeyCode::Esc => {
    if world.resource::<Selection>().is_selected() {
        world.resource_mut::<Selection>().clear();
    } else {
        *world.resource_mut::<GameState>() = GameState::Quitting;
    }
}

// Note: Mouse support requires:
// crossterm::execute!(stdout, EnableMouseCapture)?;
// And checking Event::Mouse in event loop
```

### Module Integration

```rust
// src/shared/mod.rs
pub mod time;
pub mod input;
pub mod schedule;
pub mod rendering;
pub mod selection; // ADD THIS
```

```rust
// src/lib.rs
pub use shared::selection::{Selection, SelectionTarget, inspect_tile, inspect_entity};
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Linear entity search on click**: O(n) to find entity at position
   - Future: Use spatial index (spec 014 improvement)
   - Current linear search acceptable for <100 entities

2. **String allocation per frame**: inspect_* create new Strings each render
   - Future: Cache inspection text, only update when selection changes
   - Current allocation acceptable for terminal UI

3. **No multi-selection**: Can only select one thing at a time
   - Future: Allow box selection, select multiple pops
   - Current single selection sufficient for MVP

4. **No selection highlight rendering**: Selected entity not visually distinct
   - Future: Render outline or background color on selected tile/entity
   - Current info panel feedback sufficient

5. **Mouse support commented out**: Requires extra terminal setup
   - Future: Add mouse capture with feature flag
   - Current keyboard-only selection works

### Performance Considerations

- **Selection is Copy**: Cheap to pass, check equality
- **Entity lookup is linear**: O(n) but n is small (<1000)
- **Text formatting per frame**: Negligible for <100 chars
- **No selection history**: Memory efficient, stateless

### API Design Notes

- SelectionTarget is Copy - can store, compare, match
- Selection resource is mutable - can change any time
- Inspection functions are pure - don't modify world
- screen_to_world is stateless helper - easy to test

### Future Extensibility

When adding multi-selection:
```rust
pub struct Selection {
    targets: Vec<SelectionTarget>,
    primary: Option<usize>, // Index of primary selection
}
```

When adding selection highlight:
```rust
// In rendering layer 4 (Cursor)
if let Some((x, y)) = selection.tile_coords() {
    buffer.add(
        RenderLayer::Cursor,
        RenderItem::Rect(Color::Yellow, x, y, 1, 1),
    );
}
```

When adding hover preview:
```rust
pub struct Selection {
    target: SelectionTarget,
    hover: Option<SelectionTarget>,
}
```

## Acceptance Criteria (Testable!)

- [x] All tests in RED phase pass
- [x] `cargo test` returns 0 failures
- [x] `cargo clippy -- -D warnings` passes
- [x] Test coverage ≥85% for shared/selection.rs
- [x] Selection resource tracks current target
- [x] Can select tiles by clicking/key input
- [x] Can select entities (pops, buildings)
- [x] Info panel shows tile terrain type when tile selected
- [x] Info panel shows pop needs when pop selected
- [x] Esc clears selection (or quits if nothing selected)
- [x] Selection target is queryable by other systems

## Technical Guidance

### Selection Flow

```
User clicks tile (15, 20):
  1. screen_to_world(mouse.x, mouse.y, viewport) → (15, 20)
  2. Query entities at (15, 20)
  3. If entity found → select_entity(entity)
     Else → select_tile(15, 20)
  4. Next frame: render_info_panel checks selection
  5. Calls inspect_tile or inspect_entity
  6. Displays result in info panel
```

### Info Panel Content

**No selection:**
```
Select something
to see info here
```

**Tile selected (10, 5) with Grass:**
```
Tile (10, 5)

Terrain: Grass

(Click entity for details)
```

**Pop selected:**
```
Pop
Position: (15, 20)

Hunger: 75%
Rest: 50%
```

**Building selected (future):**
```
Housing
Position: (8, 12)

Residents: 3 / 5
```

### Coordinate Conversion

```
Viewport: (10, 5)
Screen click: (3, 2) (relative to frame inner area)
World position: (10 + 3, 5 + 2) = (13, 7)
```

Remember to account for:
- Block borders (use block.inner())
- Viewport offset (add viewport.x/y)
- Panel offset (info panel starts at different x)

### Common Pitfalls

1. **Forgetting viewport offset**: Clicking screen (0, 0) → world (viewport.x, viewport.y)
2. **Selecting through overlays**: Should block selection when modal open (spec 012 integration)
3. **Dead entity references**: Selected entity might despawn - check world.entities().contains()
4. **Screen vs world coords**: Always convert before selection logic

## Questions

*Builder: add questions here if spec is unclear.*

## Future Work

This spec intentionally leaves unimplemented:
- **Multi-selection** - Box select, select multiple entities
- **Selection highlight rendering** - Visual feedback on map
- **Mouse support** - Requires EnableMouseCapture terminal setup
- **Hover preview** - Show info on hover before clicking
- **Selection history** - Undo/redo selection changes
- **Selection persistence** - Save/load selected entities
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
