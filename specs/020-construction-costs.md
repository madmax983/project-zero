# 020: Construction Costs

## Overview

Implement resource costs for constructing buildings. This connects the economy loop: Pops mine/chop resources (Spec 018/019), and players spend those resources to expand the colony.

## Dependencies

- `006` — Building Placement (for `try_place_building`)
- `018` — Mining and Resources (for `ColonyResources` and `Stone`)
- `019` — Forestry System (for `Wood`)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/building_costs_tests.rs (or inside building.rs tests module)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{BuildingType, try_place_building, OccupiedTiles};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    #[test]
    fn test_housing_cost() {
        let cost = BuildingType::Housing.cost();
        assert_eq!(cost.wood, 10.0);
        assert_eq!(cost.stone, 0.0);
    }

    #[test]
    fn test_farm_cost() {
        let cost = BuildingType::Farm.cost();
        assert_eq!(cost.wood, 20.0);
        assert_eq!(cost.stone, 5.0);
    }

    #[test]
    fn test_can_afford_success() {
        let cost = ColonyResources { wood: 10.0, stone: 0.0, ..Default::default() };
        let available = ColonyResources { wood: 15.0, stone: 5.0, ..Default::default() };

        assert!(available.can_afford(&cost));
    }

    #[test]
    fn test_can_afford_failure() {
        let cost = ColonyResources { wood: 10.0, stone: 0.0, ..Default::default() };
        let available = ColonyResources { wood: 5.0, stone: 5.0, ..Default::default() };

        assert!(!available.can_afford(&cost));
    }

    #[test]
    fn test_try_place_building_deducts_resources() {
        let mut world = World::new();
        // Setup terrain
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Setup resources (enough for Housing: 10 wood)
        world.insert_resource(ColonyResources {
            wood: 15.0,
            ..Default::default()
        });

        // Attempt placement
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);

        assert!(success);

        // Verify deduction
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.wood, 5.0);
    }

    #[test]
    fn test_try_place_building_fails_insufficient_funds() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());

        // Setup resources (not enough for Housing)
        world.insert_resource(ColonyResources {
            wood: 5.0,
            ..Default::default()
        });

        // Attempt placement
        let success = try_place_building(&mut world, 5, 5, BuildingType::Housing);

        assert!(!success);

        // Verify no deduction
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.wood, 5.0);

        // Verify no building
        assert!(world.query::<&crate::layer1::building::Building>().iter(&world).count() == 0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Costs

Update `BuildingType` to include cost values.

```rust
// src/layer1/building.rs
use crate::layer1::resources::ColonyResources;

impl BuildingType {
    // ... existing methods

    #[must_use]
    pub fn cost(&self) -> ColonyResources {
        match self {
            Self::Housing => ColonyResources { wood: 10.0, ..Default::default() },
            Self::Farm => ColonyResources { wood: 20.0, stone: 5.0, ..Default::default() },
        }
    }
}
```

### 2. Implement Affordability Logic

Update `ColonyResources` to check and deduct costs.

```rust
// src/layer1/resources.rs

impl ColonyResources {
    // ... existing

    #[must_use]
    pub fn can_afford(&self, cost: &ColonyResources) -> bool {
        self.wood >= cost.wood && self.stone >= cost.stone && self.food >= cost.food
    }

    pub fn deduct(&mut self, cost: &ColonyResources) {
        self.wood -= cost.wood;
        self.stone -= cost.stone;
        self.food -= cost.food;
    }
}
```

### 3. Enforce Costs in Placement

Update `try_place_building` to check and deduct costs.

```rust
// src/layer1/building.rs

pub fn try_place_building(world: &mut World, x: i32, y: i32, building_type: BuildingType) -> bool {
    // 1. Validate placement (position/occupied)
    if validate_building_placement(world, x, y).is_err() {
         // ... existing error logging
         return false;
    }

    // 2. Check costs
    let cost = building_type.cost();
    let can_afford = {
        let resources = world.resource::<ColonyResources>();
        resources.can_afford(&cost)
    };

    if !can_afford {
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored(format!("Not enough resources for {}", building_type.label()), Color::Red);
        }
        return false;
    }

    // 3. Deduct costs
    world.resource_mut::<ColonyResources>().deduct(&cost);

    // 4. Spawn building (existing logic)
    // ... spawn code ...
    // ... mark occupied ...

    // Log success
    // ...
    true
}
```

## REFACTOR Phase: Quality & Design

- **UI Feedback**: The Build Mode UI (status bar) should show the cost of the currently selected building (e.g., "Housing (10 Wood)").
- **Validation Refactor**: `can_place_building` (used for cursor color) currently only checks terrain/occupation. Should it also check costs?
    - *Decision*: Yes, cursor should probably turn Yellow or specialized color if placement is valid but unaffordable. For MVP, keeping it separate is fine, but `try_place_building` is the authority.
- **Resource Locking**: In a real-time game, resources might disappear between check and deduct (rare in single-threaded tick, but good to keep in mind).

## Acceptance Criteria

- [ ] `BuildingType::cost()` returns correct values.
- [ ] `ColonyResources::can_afford()` works correctly.
- [ ] Placement fails if resources are insufficient.
- [ ] Placement succeeds and deducts resources if sufficient.
- [ ] User receives feedback ("Not enough resources") on failure.
- [ ] Tests pass with ≥85% coverage.

## Technical Guidance

- **Import Cycles**: `building.rs` needs `resources.rs` (for `ColonyResources`). `resources.rs` does NOT need `building.rs`. This is safe.
- **Default Trait**: `ColonyResources` implementing `Default` is helpful for constructing costs (fields default to 0.0).

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
