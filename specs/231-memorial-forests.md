# 231: Memorial Forests

## Overview

"Memorial Forests" provide an alternative to traditional burial or recycling. Dead Pops can be "planted" to grow a **Memorial Tree**. This tree acts as a living grave, storing the Pop's identity. Relatives visit it for closure and morale boosts. Chopping it down causes massive stress ("Desecration") to those who knew the deceased.

## Dependencies

- `019` — Forestry System (Trees)
- `047` — Pop Relationships (Family/Friends)
- `031` — Pop Morale (Stress)
- `221` — Organic Recycling (Corpse Item)

## RED Phase: Tests First

Write these tests in `src/layer1/memorial_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, PopName};
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::items::{Item, ItemType, CarryingItem};
    use crate::layer1::memorial::{MemorialTree, plant_memorial_action, desecration_system, visit_memorial_action};
    use crate::layer1::memory::{Memories, MemoryType};
    use crate::layer1::social::Relationships;
    use crate::layer1::resources::{ColonyResources, chop_tree};
    use crate::layer1::designation::{Designation, DesignationType};

    #[test]
    fn test_plant_corpse_creates_memorial_tree() {
        let mut world = World::new();
        // Setup Terrain (Dirt)
        let mut tiles = vec![TerrainType::Dirt; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Spawn Corpse Entity (to be carried)
        let corpse_entity = world.spawn(crate::layer1::funeral::Corpse {
            name: "Grandpa".to_string(),
            decay: 0.0,
        }).id();

        // Spawn Pop Carrying Corpse Item
        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            CarryingItem(world.spawn(Item {
                item_type: ItemType::Corpse(corpse_entity)
            }).id())
        )).id();

        // Perform Plant Action
        plant_memorial_action(&mut world, pop, GridPosition { x: 5, y: 5 });

        // Verify Terrain is now Tree
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Tree));

        // Verify MemorialTree Component exists on a new entity at that pos
        let mut query = world.query::<(&MemorialTree, &GridPosition)>();
        let (memorial, pos) = query.single(&world);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
        assert_eq!(memorial.pop_name, "Grandpa");

        // Corpse entity should be despawned (consumed)
        assert!(world.get_entity(corpse_entity).is_err());
    }

    #[test]
    fn test_chopping_memorial_causes_desecration() {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(TerrainGrid {
            width: 10, height: 10,
            tiles: vec![TerrainType::Tree; 100]
        });

        // Spawn Memorial Tree
        let tree = world.spawn((
            MemorialTree { pop_name: "Grandpa".to_string(), deceased_id: None }, // ID might be recycled, use Name/Memory
            GridPosition { x: 5, y: 5 },
            // Forestry components would be here too
        )).id();

        // Spawn Relative (Grandson)
        let relative = world.spawn((
            Pop,
            Memories::default(),
            Relationships::default(), // Should have affinity with Grandpa if ID persisted, but name check is fallback
        )).id();

        // Mock: Add knowledge of Grandpa to relative or just check global signal
        // Ideally, Desecration checks Relationships. Since Grandpa entity is gone,
        // Relationships might track by "Deceased ID" or Name.
        // For MVP, assume `desecration_system` finds relatives of `pop_name`.

        // Perform Chop (using existing forestry logic, modified)
        // We simulate the Designation completion
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Chop },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Modify chop_tree to handle MemorialTree
        // For test, we manually call a system or function
        crate::layer1::resources::chop_tree(&mut world, designation, 10.0);

        // Run Desecration System (triggered by event or check)
        // OR chop_tree emits an event processed by desecration_system
        let mut schedule = Schedule::default();
        schedule.add_systems(desecration_system);
        schedule.run(&mut world);

        // Check Relative Stress/Memory
        let memories = world.get::<Memories>(relative).unwrap();
        // Should have "DesecratedGrave" or similar
        assert!(memories.items.iter().any(|m| m.memory_type == MemoryType::WitnessedDeath)); // Placeholder for Desecration
    }

    #[test]
    fn test_relatives_visit_memorial() {
        let mut world = World::new();
        // Setup Memorial
        let memorial = world.spawn((
            MemorialTree { pop_name: "Mom".to_string(), deceased_id: None },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Setup Relative
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 }, // Arrived
            Memories::default(),
        )).id();

        // Perform Visit Action
        visit_memorial_action(&mut world, pop, memorial);

        // Check for Closure/Comfort Memory
        let memories = world.get::<Memories>(pop).unwrap();
        assert!(memories.items.iter().any(|m| m.memory_type == MemoryType::AttendedFuneral)); // Reuse or new type
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `MemorialTree` Component

Create `src/layer1/memorial.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct MemorialTree {
    pub pop_name: String,
    pub deceased_id: Option<Entity>, // For relationship lookups if entity persists as "Dead" marker
}
```

### 2. `PlantMemorial` Action

Update `utility_ai.rs` to include `ActionType::PlantMemorial`.
Implement `plant_memorial_action` in `memorial.rs`:
-   Requires `CarryingItem` with `ItemType::Corpse`.
-   Target: `TerrainType::Dirt`.
-   Effect:
    -   Despawn Corpse Item & Corpse Entity.
    -   Set Terrain to `Tree`.
    -   Spawn `MemorialTree` entity at position.

### 3. Update `chop_tree` in `resources.rs`

Modify `chop_tree` to check for `MemorialTree` component at the position (or on the designation target entity if linked).

```rust
pub fn chop_tree(...) {
    // ...
    // Before completing:
    if let Some(memorial) = world.get::<MemorialTree>(tree_entity) {
        // Trigger Desecration Event
        world.send_event(DesecrationEvent { name: memorial.pop_name.clone() });
    }
    // ... proceed to set Dirt and spawn Wood
}
```

### 4. `Desecration` Logic

Implement `desecration_system` in `memorial.rs`:
-   Listen for `DesecrationEvent`.
-   Iterate all Pops.
-   If Pop has `Relationship` with deceased (by name or ID) OR is just "Sentimental", add `MemoryType::WitnessedDesecration`.
-   Apply massive Stress.

### 5. `VisitMemorial` Action

Update `utility_ai.rs` to score `VisitMemorial` highly for grieving relatives.
Implement `visit_memorial_action`:
-   Simply add `MemoryType::VisitedGrave` (reduces grief).

## REFACTOR Phase: Quality & Design

-   **Visuals**: Use a distinct color or character for Memorial Trees (e.g., `Color::Magenta` or `¥`).
-   **Growth**: Ideally, planting spawns a `Sapling` that grows into a Tree. For MVP, instant Tree is fine, or `Sapling` with `Memorial` component.
-   **Tooltip**: Hovering over the tree should show "Here lies [Name]".
-   **Forestry Integration**: Ensure `MemorialTree` has `Tree` component if strictly required, or acts as a proxy. Since `TerrainType` handles the "Tree-ness" for chopping, the `MemorialTree` entity is technically separate from the grid tile, but `chop_tree` must query for entities at that position.

## Acceptance Criteria

- [ ] `MemorialTree` entity exists with Pop Name.
- [ ] Planting a Corpse consumes it and creates a Tree.
- [ ] Chopping a Memorial Tree triggers Desecration event.
- [ ] Relatives visiting the tree get a positive memory.
- [ ] Tests pass.

## Technical Guidance

-   `ItemType::Corpse(Entity)` stores the world entity of the corpse. Ensure `plant_memorial_action` despawns *both* the item entity (in inventory) and the corpse entity (referenced ID).
-   `chop_tree` currently might only check `TerrainGrid`. It needs to query for entities at the position to find the `MemorialTree`.
-   Add `MemoryType::WitnessedDesecration` and `MemoryType::VisitedGrave` to `memory.rs`.

## Questions

-   Should Memorial Trees spread seeds? (Yes, treat as normal trees for biology).
    - *Architect:* Yes, they behave identically to normal trees biologically.
