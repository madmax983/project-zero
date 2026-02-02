# 017: Designation System

## Overview

Allow players to designate tiles for specific operations: Mining and Demolishing. This system acts as the interface between player intent and pop labor. Players "paint" designations on the map, which are stored as components on tile entities. This enables future Utility AI to assign workers to these tasks.

## Dependencies

- `002` — Terrain grid (tiles to designate)
- `006` — Building placement (buildings to demolish)
- `012` — Input architecture (mouse/keyboard context)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/designation.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer2::building::{Building, BuildingType, OccupiedTiles};

    #[test]
    fn test_designation_type_variants() {
        let mine = DesignationType::Mine;
        let demolish = DesignationType::Demolish;
        assert_ne!(mine, demolish);
    }

    #[test]
    fn test_designation_type_char() {
        // Visualization is important for text UI, so we test the mapping exists
        assert_eq!(DesignationType::Mine.char(), '⛏');
        assert_eq!(DesignationType::Demolish.char(), 'X');
    }

    #[test]
    fn test_designation_component() {
        let designation = Designation {
            designation_type: DesignationType::Mine,
        };
        assert_eq!(designation.designation_type, DesignationType::Mine);
    }

    #[test]
    fn test_designation_mode_default() {
        let mode = DesignationMode::default();
        assert!(!mode.active);
        assert_eq!(mode.tool, DesignationType::Mine);
        assert_eq!(mode.cursor.x, 0);
        assert_eq!(mode.cursor.y, 0);
    }

    #[test]
    fn test_designation_mode_toggle() {
        let mut mode = DesignationMode::default();
        mode.active = true;
        assert!(mode.active);
        mode.active = false;
        assert!(!mode.active);
    }

    #[test]
    fn test_can_designate_mine_valid() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // Position (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        // Can mine Rock
        assert!(can_designate(&world, 5, 5, DesignationType::Mine));
    }

    #[test]
    fn test_can_designate_mine_invalid() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        // Cannot mine Grass
        assert!(!can_designate(&world, 5, 5, DesignationType::Mine));
    }

    #[test]
    fn test_can_designate_demolish_valid() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        // Can demolish occupied tile
        assert!(can_designate(&world, 5, 5, DesignationType::Demolish));
    }

    #[test]
    fn test_can_designate_demolish_invalid() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Cannot demolish empty tile
        assert!(!can_designate(&world, 5, 5, DesignationType::Demolish));
    }

    #[test]
    fn test_try_designate_creates_entity() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        let success = try_designate(&mut world, 5, 5, DesignationType::Mine);
        assert!(success);

        let count = world.query::<(&Designation, &GridPosition)>().iter(&world).count();
        assert_eq!(count, 1);

        let (designation, pos) = world.query::<(&Designation, &GridPosition)>().single(&world);
        assert_eq!(designation.designation_type, DesignationType::Mine);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_try_designate_duplicates_ignored() {
        let mut world = World::new();
        // Setup valid rock
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());

        // First designation
        try_designate(&mut world, 5, 5, DesignationType::Mine);

        // Second designation (same type/pos)
        let success = try_designate(&mut world, 5, 5, DesignationType::Mine);

        // Should return true (idempotent) or false?
        // Let's say false because "nothing happened"
        assert!(!success);

        // Count should still be 1
        let count = world.query::<&Designation>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_cancel_designation() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(OccupiedTiles::default());

        try_designate(&mut world, 5, 5, DesignationType::Mine);
        assert_eq!(world.entities().len(), 1);

        let removed = try_cancel_designation(&mut world, 5, 5);
        assert!(removed);

        // Entity should be despawned or component removed
        // Since designation is the main component, entity despawn is cleaner
        assert_eq!(world.entities().len(), 0);
    }
}
```

**Test Coverage Requirements:**
- DesignationType variants and `char()`
- DesignationMode resource
- can_designate logic (Mine on Rock, Demolish on Building)
- try_designate (creates entity, handles duplicates)
- try_cancel_designation (removes entity)
- Coverage ≥85% for layer1/designation.rs

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Components and Resources

```rust
// src/layer1/designation.rs

use bevy_ecs::prelude::*;
use crate::layer1::{GridPosition, TerrainGrid, TerrainType};
use crate::layer2::OccupiedTiles;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DesignationType {
    #[default]
    Mine,
    Demolish,
}

impl DesignationType {
    pub const fn char(&self) -> char {
        match self {
            Self::Mine => '⛏',
            Self::Demolish => 'X',
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::Mine => "Mine",
            Self::Demolish => "Demolish",
        }
    }
}

#[derive(Component)]
pub struct Designation {
    pub designation_type: DesignationType,
}

#[derive(Resource, Default)]
pub struct DesignationMode {
    pub active: bool,
    pub tool: DesignationType,
    pub cursor: GridPosition,
}
```

### Logic Functions

```rust
// src/layer1/designation.rs

pub fn can_designate(
    world: &World,
    x: i32,
    y: i32,
    designation_type: DesignationType,
) -> bool {
    // Check bounds
    if x < 0 || y < 0 {
        return false;
    }

    // Check existing designation
    let existing = world
        .query::<(&Designation, &GridPosition)>()
        .iter(world)
        .any(|(_, pos)| pos.x == x && pos.y == y);

    if existing {
        return false;
    }

    match designation_type {
        DesignationType::Mine => {
            let terrain = world.resource::<TerrainGrid>();
            if let Some(tile) = terrain.get(x as usize, y as usize) {
                // Only rocks can be mined
                tile == TerrainType::Rock
            } else {
                false
            }
        }
        DesignationType::Demolish => {
            let occupied = world.resource::<OccupiedTiles>();
            // Only occupied tiles can be demolished
            occupied.0.contains(&(x, y))
        }
    }
}

pub fn try_designate(
    world: &mut World,
    x: i32,
    y: i32,
    designation_type: DesignationType,
) -> bool {
    if !can_designate(world, x, y, designation_type) {
        return false;
    }

    world.spawn((
        Designation { designation_type },
        GridPosition { x, y },
    ));

    true
}

pub fn try_cancel_designation(
    world: &mut World,
    x: i32,
    y: i32,
) -> bool {
    let mut to_despawn = None;

    // Find designation at position
    for (entity, _, pos) in world.query::<(Entity, &Designation, &GridPosition)>().iter(world) {
        if pos.x == x && pos.y == y {
            to_despawn = Some(entity);
            break;
        }
    }

    if let Some(entity) = to_despawn {
        world.despawn(entity);
        true
    } else {
        false
    }
}
```

### Input Integration

```rust
// src/main.rs - Update handle_input

// In handle_input function:
KeyCode::Char('m') => {
    // Toggle Designation Mode (Mine)
    let mut mode = world.resource_mut::<DesignationMode>();
    if !mode.active || mode.tool != DesignationType::Mine {
        mode.active = true;
        mode.tool = DesignationType::Mine;
        // Reset cursor to viewport center...
    } else {
        mode.active = false;
    }
    // Disable BuildMode if active
    world.resource_mut::<BuildMode>().active = false;
}

KeyCode::Char('x') => {
    // Toggle Designation Mode (Demolish)
    // Same logic...
    mode.tool = DesignationType::Demolish;
}

KeyCode::Enter if designation_mode.active => {
    let mode = world.resource::<DesignationMode>();
    try_designate(world, mode.cursor.x, mode.cursor.y, mode.tool);
}

KeyCode::Backspace | KeyCode::Delete if designation_mode.active => {
    let mode = world.resource::<DesignationMode>();
    try_cancel_designation(world, mode.cursor.x, mode.cursor.y);
}
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1. **Linear Search for Designation**: `try_cancel_designation` scans all entities.
   - Future: Spatial Index (part of spec 014/016 optimization).
   - Current: Acceptable for <100 designations.

2. **Input Conflict**: 'x' is used for Demolish, but might be used for 'Cancel' in other modes.
   - Future: Context-sensitive keymap.
   - Current: Use unique keys.

3. **Rendering Overlap**: Designations render on top of buildings/terrain.
   - Ensure Z-order is correct (Cursor > Designation > Building > Terrain).

### Performance Considerations

- **Designation Check**: O(N) where N is number of designations.
- **Rendering**: Iterates designations every frame. Keep N low or index.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for layer1/designation.rs
- [ ] Pressing 'm' toggles Mine designation mode
- [ ] Pressing 'x' toggles Demolish designation mode
- [ ] Enter places designation
- [ ] Backspace/Delete removes designation
- [ ] Cannot mine Grass/Water
- [ ] Cannot demolish empty tile
- [ ] Designations render correctly (Red '⛏' or 'X' via rendering layer)

## Technical Guidance

### Rendering Layer

Update `render_map_layer` in `src/layer1/terrain.rs` to accept `designations_data`. Use colors appropriate for the designation type (e.g., Red for Mine).

```rust
// Render loop in terrain.rs
if let Some((_, dtype)) = designations_data.iter().find(|(pos, _)| pos.x == world_x && pos.y == world_y) {
    let color = match dtype {
        DesignationType::Mine => Color::Red,
        DesignationType::Demolish => Color::Red,
    };
    line_spans.push(Span::styled(dtype.char().to_string(), Style::default().fg(color)));
}
```

### Z-Order

1. Terrain
2. Buildings / Pops
3. Designations (Overlay)
4. Cursor

## Questions

*Builder: add questions here if spec is unclear.*
