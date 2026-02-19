# 167: The Monuments of Failure

## Overview

Currently, when a building is destroyed (due to Fire, Decay, or events), it simply vanishes. This spec introduces **Ruins**.

When a building reaches 0 HP, it is replaced by a `Ruin` entity. Ruins:
- Retain the original `BuildingType` (for flavor/scavenging).
- Contain a `RuinHistory` component (when/how it was destroyed).
- Block construction on the tile until cleared.
- Can be **Scavenged** (using the `Demolish` designation) to recover a fraction of the original construction cost (e.g., 20%).

This adds narrative depth ("Here stood the first reactor, destroyed by the Great Fire of 2026") and strategic gameplay (clearing rubble vs keeping it as a memorial).

## Dependencies

- `045` — Structure Durability (HP, Destruction logic)
- `004` — Building System (BuildingType, Construction costs)
- `017` — Designation System (Demolish designation)

## RED Phase: Tests First

Write these tests in `src/layer1/ruins_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::structure::{Structure, fire_damage_structure_system};
    use crate::layer1::ruins::{Ruin, RuinHistory};
    use crate::layer1::fire::{Fire, Flammable};
    use crate::layer1::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::simulation::SimulationTime;

    #[test]
    fn test_destruction_spawns_ruin() {
        let mut world = World::new();
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(SimulationTime(0));
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Spawn a building with 1 HP
        let pos = GridPosition { x: 5, y: 5 };
        let building = world.spawn((
            Building { building_type: BuildingType::Housing },
            Structure { current_hp: 1.0, max_hp: 100.0 },
            Flammable::default(),
            pos,
        )).id();

        // Register occupation
        world.resource_mut::<OccupiedTiles>().0.insert((5, 5));

        // Spawn fire to destroy it
        world.spawn((
            Fire { intensity: 10.0, lifetime: 10 },
            pos,
        ));

        // Run damage system
        fire_damage_structure_system(&mut world);

        // Assert Building is gone
        assert!(world.get_entity(building).is_none(), "Building should be despawned");

        // Assert Ruin exists at same pos
        let mut ruin_query = world.query::<(&Ruin, &GridPosition)>();
        let (ruin, ruin_pos) = ruin_query.single(&world).unwrap();

        assert_eq!(ruin.original_type, BuildingType::Housing);
        assert_eq!(*ruin_pos, pos);

        // Assert Tile is still Occupied (Ruins block construction)
        assert!(world.resource::<OccupiedTiles>().0.contains(&(5, 5)));
    }

    #[test]
    fn test_ruin_history_recorded() {
        let mut world = World::new();
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(SimulationTime(100)); // Mock time
        world.insert_resource(crate::shared::log::MessageLog::default());

        let pos = GridPosition { x: 0, y: 0 };
        world.spawn((
            Building { building_type: BuildingType::SolarPanel },
            Structure { current_hp: 0.0, max_hp: 100.0 }, // Already dead
            Flammable::default(),
            pos,
        ));

        // Trigger system (handles 0 HP check)
        fire_damage_structure_system(&mut world);

        let (history, _) = world.query::<(&RuinHistory, &Ruin)>().single(&world).unwrap();
        assert_eq!(history.destruction_tick, 100);
        assert!(history.reason.contains("Fire") || history.reason.contains("Damage"));
    }

    #[test]
    fn test_scavenge_ruin_yields_resources() {
        let mut world = World::new();
        world.insert_resource(OccupiedTiles::default());
        let pos = GridPosition { x: 2, y: 2 };

        // Spawn a Ruin
        let ruin = world.spawn((
            Ruin { original_type: BuildingType::Wall },
            GridPosition { x: 2, y: 2 },
            // Add component to mark it valid for Demolish/Scavenge
            crate::layer1::structure::Structure::default(), // Ruins might have structure? Or just exist.
        )).id();

        world.resource_mut::<OccupiedTiles>().0.insert((2, 2));

        // Mock Scavenge execution
        // Assuming a system `process_scavenge` handles this.
        let yielded = crate::layer1::ruins::process_scavenge(&mut world, ruin);

        // Should return some resources (Wall costs e.g. 5 Stone, yield 1 Stone)
        assert!(!yielded.is_empty(), "Scavenging should yield resources");

        // Ruin should be despawned
        assert!(world.get_entity(ruin).is_none());

        // OccupiedTiles should be cleared
        assert!(!world.resource::<OccupiedTiles>().0.contains(&(2, 2)));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components

In `src/layer1/ruins.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::building::BuildingType;
use crate::layer1::items::{ItemType, Item};
use crate::layer1::GridPosition;
use crate::layer1::building::OccupiedTiles;

#[derive(Component, Debug, Clone, Copy)]
pub struct Ruin {
    pub original_type: BuildingType,
}

#[derive(Component, Debug, Clone)]
pub struct RuinHistory {
    pub destruction_tick: u64,
    pub reason: String,
}

pub fn process_scavenge(world: &mut World, ruin_entity: Entity) -> Vec<ItemType> {
    let mut loot = Vec::new();
    let mut pos = None;

    if let Some(ruin) = world.get::<Ruin>(ruin_entity) {
        // Placeholder loot logic
        // In real impl, check ruin.original_type cost and return 20%
        loot.push(ItemType::Wood);
    }

    if let Some(p) = world.get::<GridPosition>(ruin_entity) {
        pos = Some(*p);
    }

    // Spawn items on ground
    if let Some(p) = pos {
        for item in &loot {
            world.spawn((
                Item { item_type: *item },
                p,
            ));
        }

        // Remove from OccupiedTiles
        if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
            occupied.0.remove(&(p.x, p.y));
        }
    }

    world.despawn(ruin_entity);
    loot
}
```

### 2. Update Destruction Logic

Modify `fire_damage_structure_system` in `src/layer1/structure.rs`:

```rust
use crate::layer1::ruins::{Ruin, RuinHistory};
use crate::simulation::SimulationTime;
use crate::layer1::building::Building;

// Inside the loop where structure.current_hp <= 0.0:

if structure.current_hp <= 0.0 {
    // Collect info before despawn
    let building_type = world.get::<Building>(entity).map(|b| b.building_type);
    // ...

    // Instead of just pushing to `destroyed` list for simple despawn,
    // push to a list that handles Ruin creation.

    // Or handle immediately if borrowing allows (it doesn't easily in query loop).
    // Better: Add `(Entity, GridPosition, Option<BuildingType>)` to `destroyed` list.
}

// In the cleanup loop:
for (entity, pos, b_type_opt) in destroyed {
    if world.get_entity(entity).is_ok() {
        world.despawn(entity);

        // If it was a building, spawn Ruin
        if let Some(b_type) = b_type_opt {
             let current_tick = world.get_resource::<SimulationTime>().map(|t| t.0).unwrap_or(0);
             world.spawn((
                Ruin { original_type: b_type },
                RuinHistory {
                    destruction_tick: current_tick,
                    reason: "Fire/Damage".to_string(),
                },
                pos,
                // Add MapLayer::Object or similar for rendering
             ));

             // DO NOT remove from OccupiedTiles if Ruin spawns
             // Only remove if NO Ruin spawns
        } else {
             // Clean up OccupiedTiles if it wasn't a building (e.g. wall?)
             // Actually Wall is a BuildingType.
             // If for some reason we don't spawn ruin, clear tile.
             if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
                occupied.0.remove(&(pos.x, pos.y));
            }
        }
        // ... Log message ...
    }
}
```

### 3. Hook into Execution System

Modify `work_execution_system` in `src/layer1/execution.rs`:

```rust
// Handle DesignationType::Demolish
// If target has Ruin component -> call process_scavenge
// If target has Building component -> call existing demolish logic
```

## REFACTOR Phase: Quality & Design

- **Visuals**: Ruins should look like broken versions of buildings. Use a consistent color (e.g., Dark Grey) or character (e.g., lowercase first letter of building?).
- **Loot Table**: Implement a proper `scavenge_yield` function in `building.rs` that returns a percentage of construction cost.
- **Auto-Scavenge**: Maybe allow designating an area for "Clear Rubble" which targets all Ruins.
- **Decay of Ruins**: Ruins themselves could decay into nothing (dust) after a very long time (years), freeing the tile automatically.

## Acceptance Criteria

- [ ] Destroyed buildings spawn `Ruin` entities.
- [ ] Ruins block new construction (tile remains occupied).
- [ ] `Demolish` designation on a Ruin triggers scavenging.
- [ ] Scavenging removes Ruin, clears occupation, and spawns resources.
- [ ] Tests pass.

## Technical Guidance

- Ensure `OccupiedTiles` resource is correctly managed.
- `Ruin` entity needs a `GridPosition` component to be found by queries.
- Render layer for Ruins should be `Object` or `Building` but with different styling.
