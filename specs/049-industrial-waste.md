# 049: Industrial Waste Management

## Overview

Industrial processes are messy. Refining raw resources into useful materials now produces **Waste** (Slag, Tailings, Sawdust) as a byproduct. Organic matter can also rot into Waste.

Waste has negative **Beauty** and must be managed. Leaving it on the ground pollutes the colony's environment (lowering morale). Hauling it to a **Landfill** contains the ugliness in one place.

This spec introduces:
1.  **Waste Resource**: A new `ResourceType::Waste` representing generic industrial byproduct or rot.
2.  **Landfill Building**: A specialized stockpile that accepts Waste and increases `max_waste` capacity. It emits static negative beauty.
3.  **Pollution Mechanics**: `ResourceItem` entities of type Waste emit negative beauty on the map.
4.  **Refining Byproducts**: `LumberMill` and `StoneMason` produce Waste items upon completing a recipe.

## Dependencies

- `023` — Refining Industry (for `RefiningProgress` and recipes)
- `044` — Horticulture and Beauty (for `BeautyGrid` and negative beauty)
- `025` — Hauling Logistics (for `ResourceItem` and `Haul` action)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/waste_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::beauty::{BeautyGrid, update_beauty_grid_system};
    use crate::layer1::refining::{RefiningProgress, process_refining_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::GridPosition;
    use crate::layer1::stockpile::{Stockpile, update_resource_caps_system};

    // 1. Waste Resource & Landfill Cap
    #[test]
    fn test_colony_resources_has_waste() {
        let res = ColonyResources::default();
        assert_eq!(res.waste, 0.0);
        assert_eq!(res.max_waste, 0.0); // Default 0 means no storage = no hauling?
        // Actually, default should be >0 or 0 if we want to force building a dump?
        // Let's say default is 0. You MUST build a dump to haul waste.
    }

    #[test]
    fn test_landfill_increases_waste_cap() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Landfill (Stockpile variant?)
        // Or specific building type if logic differs.
        // Let's use Stockpile component but BuildingType::Landfill for unique stats.
        world.spawn((
            Building { building_type: BuildingType::Landfill },
            Stockpile { waste_bonus: 50.0, ..Default::default() },
        ));

        update_resource_caps_system(&mut world);

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.max_waste, 50.0);
    }

    // 2. Refining Produces Waste
    #[test]
    fn test_refining_produces_waste_item() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.planks = 0.0;
        resources.max_planks = 10.0;
        world.insert_resource(resources);

        // Spawn LumberMill and Worker
        let mill_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            mill_pos,
            RefiningProgress { current: 9.9, max: 10.0 },
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system
        process_refining_system(&mut world);

        // Check for Waste item at mill position
        let mut waste_query = world.query::<(&ResourceItem, &GridPosition)>();
        let found = waste_query.iter(&world).any(|(item, pos)| {
            item.resource_type == ResourceType::Waste && *pos == mill_pos
        });
        assert!(found, "Refining should spawn Waste item");
    }

    // 3. Pollution (Negative Beauty)
    #[test]
    fn test_waste_item_emits_negative_beauty() {
        let mut world = World::new();
        world.insert_resource(BeautyGrid::new(10, 10));

        // Spawn Waste Item
        world.spawn((
            ResourceItem { resource_type: ResourceType::Waste, amount: 1.0 },
            GridPosition { x: 2, y: 2 },
        ));

        // Spawn Normal Item (Wood) - should not affect beauty
        world.spawn((
            ResourceItem { resource_type: ResourceType::Wood, amount: 1.0 },
            GridPosition { x: 3, y: 3 },
        ));

        update_beauty_grid_system(&mut world);

        let grid = world.resource::<BeautyGrid>();
        assert_eq!(grid.get(2, 2), -5.0); // Waste is ugly
        assert_eq!(grid.get(3, 3), 0.0);  // Wood is neutral
    }

    #[test]
    fn test_landfill_emits_negative_beauty() {
        let mut world = World::new();
        world.insert_resource(BeautyGrid::new(10, 10));

        world.spawn((
            Building { building_type: BuildingType::Landfill },
            GridPosition { x: 4, y: 4 },
        ));

        update_beauty_grid_system(&mut world);

        let grid = world.resource::<BeautyGrid>();
        assert_eq!(grid.get(4, 4), -10.0); // Landfill is very ugly
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ResourceType` and `ColonyResources`

```rust
// src/layer1/resources.rs

pub enum ResourceType {
    // ... existing ...
    Waste,
}

pub struct ColonyResources {
    // ... existing ...
    pub waste: f32,
    pub max_waste: f32,
}

// Update Default to init waste=0, max_waste=0
```

### 2. Update `BuildingType` and `Stockpile`

```rust
// src/layer1/building.rs
pub enum BuildingType {
    // ...
    Landfill,
}

impl BuildingType {
    pub fn beauty_value(&self) -> f32 {
        match self {
            Self::FlowerBed => 5.0,
            Self::Statue => 10.0,
            Self::Landfill => -10.0, // Static ugliness
            _ => 0.0,
        }
    }
}

// src/layer1/stockpile.rs
pub struct Stockpile {
    // ... existing ...
    pub waste_bonus: f32,
}
// Update update_resource_caps_system to sum waste_bonus
```

### 3. Update `process_refining_system`

Modify `src/layer1/refining.rs` to spawn `ResourceItem` when completing a recipe.

```rust
// Inside process_refining_system, when progress complete:

// ... deduct input, add output (global) ...

// Spawn Waste
world.spawn((
    ResourceItem {
        resource_type: ResourceType::Waste,
        amount: 1.0, // Or based on recipe
    },
    *pos, // Building position
    // Optional: Add Perishable? (Waste might rot away or just stay forever?)
    // Let's say Industrial Waste stays forever (no Perishable).
));
```

### 4. Update `update_beauty_grid_system`

Modify `src/layer1/beauty.rs` to query items too.

```rust
pub fn update_beauty_grid_system(
    mut grid: ResMut<BeautyGrid>,
    buildings: Query<(&GridPosition, &Building)>,
    items: Query<(&GridPosition, &ResourceItem)>, // New query
) {
    grid.clear();

    // 1. Buildings
    for (pos, building) in &buildings {
        let value = building.building_type.beauty_value();
        if value != 0.0 {
            // ... apply to grid ...
        }
    }

    // 2. Items
    for (pos, item) in &items {
        if item.resource_type == ResourceType::Waste {
            // Apply negative beauty
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                let current = grid.get(x, y);
                grid.set(x, y, current - 5.0); // Stackable?
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Stacking Ugliness**: If 10 waste items are on one tile, is it -50 beauty? Yes, this encourages cleaning up!
- **Recipe Config**: The amount of waste per refining job should be configurable.
- **Visuals**: `Landfill` needs a distinct char (maybe `xxxx` or `░░░░`). `Waste` item needs a char (`💩` or `⚠️`).
- **Hauling Priority**: Pops might prioritize hauling Waste if it's lowering morale (via Beauty/Leisure). This requires updating Utility AI weights for "Clean Up" vs "Haul Resources". For MVP, standard Haul logic applies.

## Acceptance Criteria

- [ ] `ResourceType::Waste` exists.
- [ ] `Landfill` building can be built and increases waste cap.
- [ ] `Refining` generates Waste items on the map.
- [ ] Waste items on the map reduce local Beauty.
- [ ] `Landfill` building reduces local Beauty.
- [ ] Pops can haul Waste to Landfill (removing the item and adding to global waste count).
- [ ] Tests pass.

## Technical Guidance

- **Beauty System**: Ensure you don't overwrite grid values when iterating items. Use `grid.get() + delta`.
- **Hauling**: Ensure `evaluate_haul` checks `max_waste` before attempting to haul waste. If `max_waste` is 0 (no landfill), utility should be 0.
- **Map Rendering**: Update `render_map` in `src/ui/map.rs` to show Waste items if not already generic.

## Questions

*Builder: Should rotting food turn into Waste items? (Yes, update `spoilage_system` from Spec 032 to spawn `ResourceItem(Waste)` instead of just despawning).*
