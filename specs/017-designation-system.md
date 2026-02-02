# 017: Designation System

## Overview

A system to mark tiles or entities for future worker actions. This bridges Player Input (Layer 1) and Unit Behavior (Utility AI). Players can designate tiles to be Mined (removing Rock) or Demolished (removing Buildings).

## Dependencies

- `002` — Terrain grid (rock detection)
- `006` — Building placement (building detection)
- `012` — Input architecture (integration point)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/designation.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::GridPosition;

    #[test]
    fn test_designation_type_enum() {
        let mine = DesignationType::Mine;
        let demolish = DesignationType::Demolish;

        assert_ne!(mine, demolish);
    }

    #[test]
    fn test_designation_component() {
        let designation = Designation {
            d_type: DesignationType::Mine,
        };
        assert_eq!(designation.d_type, DesignationType::Mine);
    }

    #[test]
    fn test_can_designate_mine_valid() {
        let mut world = World::new();
        // Setup Rock terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Rock; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Should be able to mine Rock
        assert!(can_designate(&world, 5, 5, DesignationType::Mine));
    }

    #[test]
    fn test_can_designate_mine_invalid() {
        let mut world = World::new();
        // Setup Grass terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Should NOT be able to mine Grass
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
        // Occupied by building
        let mut occupied = OccupiedTiles::default();
        occupied.0.insert((5, 5));
        world.insert_resource(occupied);

        // Should be able to demolish if occupied
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
        world.insert_resource(OccupiedTiles::default()); // Empty

        // Should NOT be able to demolish empty tile
        assert!(!can_designate(&world, 5, 5, DesignationType::Demolish));
    }

    #[test]
    fn test_try_designate_success() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Rock; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Apply Mine designation
        let success = try_designate(&mut world, 5, 5, DesignationType::Mine);
        assert!(success);

        // Check if entity spawned
        let count = world.query::<(&Designation, &GridPosition)>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_try_designate_duplicate() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Rock; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // First time
        try_designate(&mut world, 5, 5, DesignationType::Mine);

        // Second time - should fail (already designated)
        let success = try_designate(&mut world, 5, 5, DesignationType::Mine);
        assert!(!success);

        // Should still only have 1 entity
        let count = world.query::<(&Designation, &GridPosition)>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_remove_designation() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Rock; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        try_designate(&mut world, 5, 5, DesignationType::Mine);
        assert!(has_designation(&world, 5, 5));

        remove_designation(&mut world, 5, 5);
        assert!(!has_designation(&world, 5, 5));
    }

    #[test]
    fn test_designation_mode_resource() {
        let mode = DesignationMode::default();
        assert!(!mode.active);
        assert_eq!(mode.current_type, DesignationType::Mine);
    }
}
```

**Test Coverage Requirements:**
- `DesignationType` enum (Mine, Demolish)
- `Designation` component structure
- `can_designate` logic (Mine needs Rock, Demolish needs Occupied)
- `try_designate` spawns entity only if valid and not duplicate
- `remove_designation` cleans up entity
- `DesignationMode` resource state
- All tests must pass before spec is considered complete

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Core Components

```rust
// src/layer1/designation.rs

use bevy_ecs::prelude::*;
use crate::layer1::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::building::OccupiedTiles;
use ratatui::style::Color;

/// Types of designations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DesignationType {
    Mine,
    Demolish,
}

impl Default for DesignationType {
    fn default() -> Self {
        Self::Mine
    }
}

impl DesignationType {
    #[must_use]
    pub const fn char(&self) -> char {
        match self {
            Self::Mine => '⛏',
            Self::Demolish => 'X',
        }
    }

    #[must_use]
    pub const fn color(&self) -> Color {
        match self {
            Self::Mine => Color::Red,
            Self::Demolish => Color::Red,
        }
    }

    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Mine => "Mine",
            Self::Demolish => "Demolish",
        }
    }
}

/// Marks a tile for work.
#[derive(Component, Debug)]
pub struct Designation {
    pub d_type: DesignationType,
}

/// State for the UI designation tool.
#[derive(Resource, Default)]
pub struct DesignationMode {
    pub active: bool,
    pub cursor: GridPosition,
    pub current_type: DesignationType,
}
```

### Validation & Logic

```rust
// src/layer1/designation.rs

/// Check if a designation is valid for a location.
#[must_use]
pub fn can_designate(
    world: &World,
    x: i32,
    y: i32,
    d_type: DesignationType,
) -> bool {
    let terrain = world.resource::<TerrainGrid>();
    let occupied = world.resource::<OccupiedTiles>();

    // Check bounds
    if x < 0 || y < 0 || x >= terrain.width as i32 || y >= terrain.height as i32 {
        return false;
    }

    match d_type {
        DesignationType::Mine => {
            // Can only mine rocks
            #[allow(clippy::cast_sign_loss)]
            if let Some(tile) = terrain.get(x as usize, y as usize) {
                tile == TerrainType::Rock
            } else {
                false
            }
        }
        DesignationType::Demolish => {
            // Can only demolish if occupied
            occupied.0.contains(&(x, y))
        }
    }
}

/// Check if a location already has any designation.
#[must_use]
pub fn has_designation(world: &World, x: i32, y: i32) -> bool {
    let mut query = world.query::<(&Designation, &GridPosition)>();
    for (_, pos) in query.iter(world) {
        if pos.x == x && pos.y == y {
            return true;
        }
    }
    false
}

/// Attempt to apply a designation. Returns true if successful.
pub fn try_designate(
    world: &mut World,
    x: i32,
    y: i32,
    d_type: DesignationType,
) -> bool {
    // 1. Validation
    if !can_designate(world, x, y, d_type) {
        return false;
    }

    // 2. Check for duplicate
    if has_designation(world, x, y) {
        return false;
    }

    // 3. Spawn Designation entity
    world.spawn((
        Designation { d_type },
        GridPosition { x, y },
    ));

    true
}

/// Remove designation at a location (cancel).
pub fn remove_designation(world: &mut World, x: i32, y: i32) {
    // Find entity
    let mut target = None;
    let mut query = world.query::<(Entity, &GridPosition, &Designation)>();
    for (entity, pos, _) in query.iter(world) {
        if pos.x == x && pos.y == y {
            target = Some(entity);
            break;
        }
    }

    // Despawn
    if let Some(entity) = target {
        world.despawn(entity);
    }
}
```

### Module Integration

```rust
// src/layer1/mod.rs
pub mod designation;
pub use designation::{Designation, DesignationType, DesignationMode, try_designate};
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1.  **Linear Search for Designation**: `has_designation` and `remove_designation` iterate all designations. O(N).
    -   *Improvement*: Maintain a `DesignationGrid` resource or hash map for O(1) lookups.
    -   *Current*: Acceptable for MVP (N < 100).

2.  **Shared Input Logic**: `DesignationMode` duplicates `BuildMode` cursor logic.
    -   *Improvement*: Abstract "GridCursor" logic into a shared component/resource.

3.  **Visual Overlap**: `Mine` char '⛏' might overlap with terrain chars.
    -   *Improvement*: Use Z-ordering in renderer (Spec 014) to ensure designations float above.

### Performance Considerations

-   **Entity Spawning**: Spawning an entity for every designated tile is fine for hundreds of tiles, but inefficient for thousands.
    -   *Future*: Use a sparse grid resource if designations become massive.

### Integration Points

-   **Renderer**: Needs to query `Designation` components and render them (usually blinking or colored).
-   **Utility AI (016)**: `Mine` designation creates a potential task for Workers. The AI system will query `Designation` entities to find jobs.
    -   Example: `evaluate_mine_action` will look for `Designation { d_type: Mine }`.

## Acceptance Criteria (Testable!)

-   [ ] All tests in RED phase pass.
-   [ ] `Designation` component exists.
-   [ ] `DesignationType` supports Mine and Demolish.
-   [ ] Validations prevent mining Grass or demolishing empty tiles.
-   [ ] Validations allow mining Rock and demolishing Buildings.
-   [ ] Designations are unique per tile (no stacking).
-   [ ] Can remove designations.

## Technical Guidance

### Rendering Designations

Since this spec doesn't cover the Renderer update (which might be in 014), for now, ensure the `Designation` has a `char()` method so the renderer *can* be updated easily.

Suggested rendering logic (in `src/layer1/terrain.rs` or similar):
```rust
// In render loop
for (pos, designation) in designation_query.iter() {
    if is_visible(pos) {
        buffer.put(pos.x, pos.y, designation.d_type.char(), designation.d_type.color());
    }
}
```

### Input Integration (Spec 012)

You will need to add a new `InputContext` for Designation Mode.
-   Key `D`: Toggle Designation Mode.
-   Key `Tab`: Cycle Designation Type.
-   Key `Enter` / `Space`: Apply designation.
-   Key `X` / `Delete`: Remove designation.

## Questions

*Builder: add questions here if spec is unclear.*
