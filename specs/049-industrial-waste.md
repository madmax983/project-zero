# 049: Industrial Waste Management

## Overview

Industrial progress comes at a cost. Refining raw materials and processing goods generates **Waste** (Slag, Tailings, Chemical Ash).
This spec introduces a waste management loop:
1.  **Generation**: Refining buildings produce `Waste` items as a byproduct.
2.  **Pollution**: `Waste` items on the ground emit negative **Beauty**, reducing the morale of nearby Pops.
3.  **Storage**: A new `Landfill` building acts as a specialized stockpile for Waste.
4.  **Disposal**: Pops must haul Waste to the Landfill to "contain" the pollution (removing the item from the map and adding it to a global `Waste` counter).

## Dependencies

- `023` — Refining Industry (for `process_refining_system`)
- `044` — Horticulture and Beauty (for `BeautyGrid` and negative beauty)
- `025` — Hauling Logistics (for `ResourceItem` and hauling logic)
- `022` — Resource Stockpiles (for `max_waste` logic)

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
    use crate::layer1::GridPosition;
    use crate::layer1::pop::Pop;

    // 1. ResourceType::Waste exists
    #[test]
    fn test_resource_type_waste() {
        let waste = ResourceType::Waste;
        // Verify it can be used in ResourceItem
        let item = ResourceItem { resource_type: waste, amount: 1.0 };
        assert_eq!(item.resource_type, ResourceType::Waste);
    }

    // 2. Refining produces Waste
    #[test]
    fn test_refining_produces_waste_item() {
        let mut world = World::new();
        // Setup Resources (Wood -> Planks + Waste)
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.planks = 0.0;
        world.insert_resource(resources);

        // Spawn LumberMill and Worker
        let mill_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            mill_pos,
            RefiningProgress { current: 9.9, max: 10.0 }, // Almost done
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system
        process_refining_system(&mut world);

        // Check for Waste item on ground near mill
        let mut found_waste = false;
        let mut query = world.query::<(&GridPosition, &ResourceItem)>();
        for (pos, item) in query.iter(&world) {
            if item.resource_type == ResourceType::Waste && pos == &mill_pos {
                found_waste = true;
            }
        }
        assert!(found_waste, "Refining should spawn Waste item at building location");
    }

    // 3. Waste item emits negative beauty
    #[test]
    fn test_waste_item_negative_beauty() {
        let mut world = World::new();
        world.insert_resource(BeautyGrid::new(10, 10));

        // Spawn Waste Item
        world.spawn((
            ResourceItem { resource_type: ResourceType::Waste, amount: 1.0 },
            GridPosition { x: 2, y: 2 },
        ));

        // Run beauty system
        update_beauty_grid_system(&mut world);

        let grid = world.resource::<BeautyGrid>();
        // Expect negative value (e.g., -5.0)
        assert!(grid.get(2, 2) < 0.0, "Waste item should emit negative beauty");
    }

    // 4. Landfill Building
    #[test]
    fn test_landfill_building_properties() {
        let lf = BuildingType::Landfill;
        // Should have negative intrinsic beauty
        assert!(lf.beauty_value() < 0.0);
    }

    // 5. Landfill increases Waste Cap
    #[test]
    fn test_landfill_increases_waste_cap() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Landfill
        world.spawn((
            Building { building_type: BuildingType::Landfill },
            // Stockpile component with waste_bonus
            crate::layer1::stockpile::Stockpile { waste_bonus: 100.0, ..Default::default() },
        ));

        // Run cap update system
        crate::layer1::stockpile::update_resource_caps_system(&mut world);

        let res = world.resource::<ColonyResources>();
        assert!(res.max_waste >= 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update ResourceType

```rust
// src/layer1/resources.rs

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceType {
    // ...
    Waste, // New
}

pub struct ColonyResources {
    // ...
    pub waste: f32,
    pub max_waste: f32,
}
// Add Default impl (waste: 0, max_waste: 0 by default - MUST build Landfill to store)
```

### 2. Update Stockpile Logic

```rust
// src/layer1/stockpile.rs

pub struct Stockpile {
    // ...
    pub waste_bonus: f32, // New
}

// Update update_resource_caps_system to sum waste_bonus
```

### 3. Update BuildingType

```rust
// src/layer1/building.rs

pub enum BuildingType {
    // ...
    Landfill,
}

impl BuildingType {
    pub fn beauty_value(&self) -> f32 {
        match self {
            Self::Landfill => -10.0, // Static ugliness
            Self::FlowerBed => 5.0,
            // ...
        }
    }
}
```

### 4. Update Refining System to Spawn Waste

```rust
// src/layer1/refining.rs

pub fn process_refining_system(world: &mut World) {
    // ... existing logic ...
    // Inside the loop where work completes:

    // Check if we produced waste (random chance or deterministic)
    // For MVP: 1 Waste per operation for LumberMill/StoneMason?
    // Or maybe 1 Waste per 5 operations to avoid clutter?
    // Let's say: 50% chance of 1 Waste per operation.

    if completed {
        // ... update resources ...

        // Spawn Waste Item
        world.spawn((
            ResourceItem { resource_type: ResourceType::Waste, amount: 1.0 },
            *pos, // At building location
            // Add Perishable if we want it to Rot? (Maybe Waste rots into... nothing? or toxic sludge?)
            // For now, Waste is permanent until hauled.
        ));
    }
}
```

### 5. Update Beauty System

```rust
// src/layer1/beauty.rs

pub fn update_beauty_grid_system(
    mut grid: ResMut<BeautyGrid>,
    buildings: Query<(&GridPosition, &Building)>,
    items: Query<(&GridPosition, &ResourceItem)>, // New Query
) {
    grid.clear();

    // 1. Buildings
    for (pos, building) in &buildings {
        let value = building.building_type.beauty_value();
        if value != 0.0 {
             // ... apply ...
        }
    }

    // 2. Items
    for (pos, item) in &items {
        if item.resource_type == ResourceType::Waste {
            // Apply negative beauty (e.g. -5.0)
             if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                let current = grid.get(x, y);
                grid.set(x, y, current - 5.0);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Refining Output**: Hardcoded waste generation in `process_refining_system` is ugly. Move to a `RefiningRecipe` struct.
- **Waste Types**: Differentiate `Slag` (Stone) vs `Sawdust` (Wood)? For MVP, generic `Waste` is fine.
- **Visualization**: `Landfill` needs a distinct char. `Waste` needs a distinct char (maybe `x` or `%`).
- **Optimization**: Querying all items every frame for beauty might be slow if there are thousands. Spatial hashing or `Changed<ResourceItem>` could help.

## Acceptance Criteria

- [ ] `ResourceType::Waste` exists.
- [ ] `Landfill` building is placeable and holds waste cap.
- [ ] Refining operations spawn `Waste` items on the map.
- [ ] `Waste` items emit negative beauty.
- [ ] Pops can haul `Waste` to `Landfill`.
- [ ] Tests pass.

## Technical Guidance

- **Hauling**: Ensure `evaluate_haul` (Spec 025) considers `Waste` a valid target. If `max_waste` is 0 (no landfill), utility should be 0 (cannot haul).
- **Stockpile Config**: Ensure `Landfill` is configured as a `Stockpile` with `waste_bonus > 0` and `food/wood/stone_bonus = 0`.
- **UI**: Add `Waste` to the resource panel in `src/ui/mod.rs`.

## Questions

*Builder: Should Waste decay naturally?*
*Architect:* No, industrial waste should not decay naturally. It must be actively processed, stored, or hauled to a designated disposal area to maintain the core tension of industrial pollution.
*Architect: No, Industrial Waste is persistent and must be hauled to a Landfill.*
