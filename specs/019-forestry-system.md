# 019: Forestry System

## Overview

Implement the forestry mechanics, allowing players to harvest Wood from trees. This extends the resource system established in Spec 018 and the designation system from Spec 017.

## Dependencies

- `002` — Terrain Grid (for `TerrainType::Tree`)
- `017` — Designation System (for `DesignationType::Chop`)
- `018` — Mining and Resources (for `ColonyResources` and logic patterns)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/forestry_tests.rs (or similar, can be in terrain.rs or resources.rs)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use ratatui::style::Color;
    use crate::layer1::terrain::{TerrainType, TerrainGrid};
    use crate::layer1::designation::{DesignationType, Designation, can_designate};
    use crate::layer1::resources::{ColonyResources, ForestryProgress, chop_tree};
    use crate::layer1::GridPosition;

    #[test]
    fn test_terrain_type_tree() {
        // Test new variant properties
        assert_eq!(TerrainType::Tree.as_str(), "↑");
        assert_eq!(TerrainType::Tree.color(), Color::DarkGreen);
        assert_eq!(TerrainType::Tree.name(), "Tree");
    }

    #[test]
    fn test_designation_type_chop() {
        // Test new variant properties
        assert_eq!(DesignationType::Chop.char(), '🪓'); // Axe character
        assert_eq!(DesignationType::Chop.label(), "Chop");
    }

    #[test]
    fn test_can_designate_chop_valid() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5, 5)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Should be able to chop a Tree
        assert!(can_designate(&world, 5, 5, DesignationType::Chop));
    }

    #[test]
    fn test_can_designate_chop_invalid() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Cannot chop Grass
        assert!(!can_designate(&world, 5, 5, DesignationType::Chop));
    }

    #[test]
    fn test_forestry_progress_component() {
        let progress = ForestryProgress {
            current: 0.0,
            max: 50.0,
        };
        assert_eq!(progress.current, 0.0);
        assert_eq!(progress.max, 50.0);
    }

    #[test]
    fn test_chop_tree_increments_progress() {
        let mut world = World::new();
        // Setup Tree
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Chop },
            ForestryProgress { current: 0.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Perform work
        chop_tree(&mut world, designation, 1.0);

        let progress = world.get::<ForestryProgress>(designation).unwrap();
        assert_eq!(progress.current, 1.0);
    }

    #[test]
    fn test_chop_tree_completion() {
        let mut world = World::new();
        // Setup Tree
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Chop },
            ForestryProgress { current: 9.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Complete work
        chop_tree(&mut world, designation, 1.0);

        // 1. Entity should be despawned
        assert!(world.get_entity(designation).is_none());

        // 2. Terrain should be Dirt (cleared land)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // 3. Resources should increase (Wood)
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.wood, 1.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums

```rust
// src/layer1/terrain.rs
pub enum TerrainType {
    // ... existing
    Tree,
}

// In impl TerrainType:
// as_str -> "↑"
// color -> Color::DarkGreen
// name -> "Tree"
```

```rust
// src/layer1/designation.rs
pub enum DesignationType {
    // ... existing
    Chop,
}

// In impl DesignationType:
// char -> '🪓'
// label -> "Chop"
```

### 2. Update Designation Logic

```rust
// src/layer1/designation.rs

pub fn can_designate(...) -> bool {
    match designation_type {
        // ...
        DesignationType::Chop => {
             let terrain = world.resource::<TerrainGrid>();
             if let Some(tile) = terrain.get(x as usize, y as usize) {
                 tile == TerrainType::Tree
             } else {
                 false
             }
        }
    }
}
```

### 3. Implement Forestry Logic

```rust
// src/layer1/resources.rs

#[derive(Component, Debug, Default)]
pub struct ForestryProgress {
    pub current: f32,
    pub max: f32,
}

pub fn chop_tree(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // Logic identical to mine_rock but checks for Tree and ForestryProgress
    // Awards wood += 1.0
    // Changes terrain to Dirt
}
```

## REFACTOR Phase: Quality & Design

- **Unify Progress Components**: `MiningProgress` and `ForestryProgress` are identical structurally. Consider refactoring them into a generic `WorkProgress` or `GatheringProgress` component in `resources.rs`.
- **Configurable Yields**: Trees should ideally yield variable wood, or multiple drops.
- **Tree Regeneration**: Trees currently just disappear. Future specs (or "Seasonal Rhythms") might handle regrowth.
- **Input Integration**: Ensure `DesignationMode` supports switching to Chop (e.g., 'C' key). This might require updating `main.rs` or `input.rs` if keys are hardcoded. *Note: Spec 012/017 handles this via `DesignationMode`, but key bindings need to be updated in the Builder's implementation.*

## Acceptance Criteria

- [ ] `TerrainType::Tree` exists and renders correctly.
- [ ] `DesignationType::Chop` exists and renders correctly.
- [ ] Can designate trees for chopping.
- [ ] Chopping a tree takes time (progress).
- [ ] Completing a chop transforms Tree -> Dirt.
- [ ] Completing a chop adds +1 Wood to `ColonyResources`.
- [ ] Tests pass with ≥85% coverage.

## Technical Guidance

- **Terrain Generation**: You might want to update `generate_terrain` in `src/layer1/terrain.rs` to spawn some trees (maybe "Forest" clusters) so there is something to chop in the game.
- **Key Binding**: Add 'C' for Chop mode in `handle_input`.

## Questions

*Builder: add questions here if spec is unclear.*
