# 019: Forestry System

## Overview

Implement the Forestry system to allow harvesting Wood from Trees. This follows the pattern established by Spec 018 (Mining). Trees are represented as `TerrainType::Tree`. This feature adds a new terrain type, a new designation type, and the logic to harvest trees.

## Dependencies

- `018` — Mining and Resources (for `ColonyResources`, `MiningProgress`, and existing patterns)
- `017` — Designation System
- `002` — Terrain Grid

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/resources.rs (Add these tests to the existing tests module)

#[cfg(test)]
mod tests_forestry {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::pop::GridPosition;

    #[test]
    fn test_terrain_type_tree_exists() {
        // This test ensures TerrainType::Tree is defined
        let tree = TerrainType::Tree;
        assert_eq!(tree.name(), "Tree");
        assert_eq!(tree.as_str(), "T"); // Or whatever character is chosen
    }

    #[test]
    fn test_designation_type_chop_exists() {
        // This test ensures DesignationType::Chop is defined
        let chop = DesignationType::Chop;
        assert_eq!(chop.name(), "Chop"); // Assuming DesignationType has a name() or debug impl
    }

    #[test]
    fn test_chop_tree_increments_progress() {
        let mut world = World::new();
        // Setup Tree tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5, 5)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation with MiningProgress (reused for chopping)
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Chop },
            MiningProgress { current: 0.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Perform work (simulate 1 tick of work)
        chop_tree(&mut world, designation, 1.0);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert_eq!(progress.current, 1.0);
    }

    #[test]
    fn test_chop_tree_completion() {
        let mut world = World::new();
        // Setup Tree tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Chop },
            MiningProgress { current: 9.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Complete the work
        chop_tree(&mut world, designation, 1.0);

        // 1. Entity should be despawned (Designation removed)
        assert!(world.get_entity(designation).is_none());

        // 2. Terrain should be Grass (or Dirt? Let's say Grass for now as it grows on it)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Grass));

        // 3. Resources should increase
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.wood, 1.0);
    }

    #[test]
    fn test_chop_tree_ignores_non_tree() {
        let mut world = World::new();
        // Setup Grass tile (cannot chop grass)
        let mut tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        let designation = world.spawn((
            Designation { designation_type: DesignationType::Chop },
            MiningProgress { current: 0.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        chop_tree(&mut world, designation, 5.0);

        // Should not progress
        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert_eq!(progress.current, 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `TerrainType`

```rust
// src/layer1/terrain.rs

pub enum TerrainType {
    Grass,
    Dirt,
    Rock,
    Water,
    Tree, // Add this
}

impl TerrainType {
    pub const fn as_str(self) -> &'static str {
        match self {
            // ...
            Self::Tree => "T", // or "♣" or "♠"
        }
    }

    pub const fn color(self) -> Color {
        match self {
            // ...
            Self::Tree => Color::Green, // Maybe DarkGreen?
        }
    }
}
```

### 2. Update `DesignationType`

```rust
// src/layer1/designation.rs

pub enum DesignationType {
    Mine,
    Demolish,
    Chop, // Add this
}

impl DesignationType {
    pub const fn as_str(self) -> &'static str {
        match self {
            // ...
            Self::Chop => "🪓", // Axe icon
        }
    }
}
```

### 3. Implement `chop_tree`

```rust
// src/layer1/resources.rs

/// Applies work to a chop designation.
/// If complete, transforms terrain and awards wood.
pub fn chop_tree(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // 1. Get position and verify terrain
    let pos = if let Some(pos) = world.get::<GridPosition>(designation_entity) {
        *pos
    } else {
        return;
    };

    let is_tree = {
        let terrain = world.resource::<TerrainGrid>();
        terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Tree)
    };

    if !is_tree {
        return;
    }

    // 2. Update progress
    let mut completed = false;
    if let Some(mut progress) = world.get_mut::<MiningProgress>(designation_entity) {
        progress.current += work_amount;
        if progress.current >= progress.max {
            completed = true;
        }
    }

    // 3. Handle completion
    if completed {
        // Change terrain to Grass
        let mut terrain = world.resource_mut::<TerrainGrid>();
        if let Some(idx) = (pos.y as usize).checked_mul(terrain.width).and_then(|y| y.checked_add(pos.x as usize)) {
            if idx < terrain.tiles.len() {
                terrain.tiles[idx] = TerrainType::Grass;
            }
        }

        // Add resources
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.wood += 1.0;

        // Remove designation
        world.despawn(designation_entity);
    }
}
```

### 4. Update Terrain Generation

Update `generate_terrain` in `src/layer1/terrain.rs` to scatter trees (forests).

## REFACTOR Phase: Quality & Design

- **Code Duplication**: `mine_rock` and `chop_tree` are nearly identical.
    - **Refactor**: Create a generic `gather_resource(world, entity, amount, target_terrain, result_terrain, resource_type)` function?
    - Or keep them separate if they might diverge (e.g., trees might regrow, rocks don't). For now, separate is fine, but look for patterns.
- **Tree Representation**: currently a Tile. Later might need to be an Entity if we want saplings, growth stages, etc. For MVP, Tile is perfect (KISS).
- **Hardness**: Trees should be easier to chop than rocks are to mine. Adjust `max` progress accordingly in `designation.rs` logic (when creating the designation).

## Acceptance Criteria

- [ ] `TerrainType::Tree` exists and renders.
- [ ] `DesignationType::Chop` exists and renders.
- [ ] `chop_tree` logic works (consumes tree, gives wood, removes designation).
- [ ] Trees appear in generated terrain.
- [ ] Coverage ≥85% for `src/layer1/resources.rs`.
- [ ] `cargo test` passes.

## Technical Guidance

- **Icons**: Use a distinct character for Trees. 'T' is safe, '♠' or '♣' might look like farm crops. Maybe '↑' (arrow up) or '🌲' (might be double width, careful with TUI). Standard ASCII 'T' or 't' is safest.
- **Color**: Make Trees distinct from Grass. `Color::DarkGreen` vs `Color::Green`.
- **Integration**: Ensure the Input/Designation system (017) can actually set `DesignationType::Chop`. (This spec creates the type, the logic to *apply* it via UI might be generic enough if it just cycles enum variants, or might need a tweak in 017/012 code). *Self-correction: 017/012 likely needs to know about `Chop` to apply it.*

## Questions

*Builder: add questions here if spec is unclear.*
