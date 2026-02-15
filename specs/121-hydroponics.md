# 121: Hydroponics

## Overview

Hydroponics is the pinnacle of agricultural technology on the colony. Unlike traditional soil farming or simple greenhouses, **Hydroponics Bays** consume **Water** and **Power** to produce food at extremely high rates, completely independent of seasons or soil fertility.

This building serves as a mid-to-late game solution for food security, trading labor/land constraints for infrastructure (water/power) constraints.

## Dependencies

- `008` — Farming (Base system)
- `042` — Energy System (Power consumption)
- `096` — Water Simulation (Water consumption)
- `109` — Greenhouses (Conceptual predecessor)
- `120` — Crop Diversity (Compatible with specific crops)

## RED Phase: Tests First

Write these tests in `src/layer1/hydroponics_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::farm::{Farm, produce_food_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::water::WaterGrid;
    use crate::layer1::energy::PowerGrid; // Assuming PowerGrid resource or component
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_hydroponics_consumes_water_and_power() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.water = 100.0;
        res.power = 100.0; // Assuming power is tracked in resources or grid
        world.insert_resource(res);

        // Setup Grid for Power/Water if needed, or just Resource check
        // Assuming simpler resource consumption for MVP test

        world.spawn((
            Farm::default(),
            Building { building_type: BuildingType::HydroponicsBay },
            GridPosition { x: 5, y: 5 },
        ));

        // Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction { current: ActionType::Farm, ..Default::default() },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(res.water < 100.0, "Hydroponics must consume water");
        // Power check might depend on implementation (042 uses PowerSource/Consumer)
        // If produce_food_system handles consumption, check it here.
    }

    #[test]
    fn test_hydroponics_production_multiplier() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SeasonState { current_season: Season::Winter });

        // Hydroponics
        world.spawn((
            Farm::default(),
            Building { building_type: BuildingType::HydroponicsBay },
            GridPosition { x: 5, y: 5 },
        ));

        // Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction { current: ActionType::Farm, ..Default::default() },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        // Base 0.005. Hydroponics multiplier 2.0. Winter ignore.
        // Expected ~0.01
        assert!(res.food >= 0.009, "Hydroponics should produce ~2x base yield");
    }

    #[test]
    fn test_hydroponics_fails_without_water() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.water = 0.0; // No water
        world.insert_resource(res);

        world.spawn((
            Farm::default(),
            Building { building_type: BuildingType::HydroponicsBay },
            GridPosition { x: 5, y: 5 },
        ));

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction { current: ActionType::Farm, ..Default::default() },
        ));

        world.run_system_once(produce_food_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.food, 0.0, "Should not produce food without water");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType`

```rust
// src/layer1/building.rs
pub enum BuildingType {
    // ...
    HydroponicsBay,
}

impl BuildingType {
    pub const fn cost(&self, material: MaterialType) -> ColonyResources {
        match self {
            Self::HydroponicsBay => ColonyResources {
                metal: 30.0,
                stone: 10.0,
                glass: 10.0, // If Glass exists, otherwise Stone
                ..ColonyResources::zeroed()
            },
            // ...
        }
    }
}
```

### 2. Update `produce_food_system` in `src/layer1/farm.rs`

```rust
// Constants
const HYDROPONICS_WATER_COST: f32 = 0.1;
const HYDROPONICS_MULTIPLIER: f32 = 2.0;

pub fn produce_food_system(
    // ... params
    mut resources: ResMut<ColonyResources>,
) {
    // ... loop ...

    let (water_cost, modifier) = match building_type {
        BuildingType::HydroponicsBay => (HYDROPONICS_WATER_COST, HYDROPONICS_MULTIPLIER),
        BuildingType::Greenhouse => (0.0, 1.0),
        _ => (0.0, season_modifier),
    };

    // Check constraints
    if water_cost > 0.0 && resources.water < water_cost {
        // Log "Missing Water" notification?
        continue;
    }

    // Produce
    let production = efficiency * base_yield * modifier;

    if production > 0.0 {
        if water_cost > 0.0 {
            resources.water -= water_cost;
        }
        resources.add_food(production);
    }
}
```

### 3. Power Integration (Spec 042)

Add `PowerConsumer` component to `spawn_building` for `HydroponicsBay`.
Update `produce_food_system` to check `PowerConsumer.active`. If not active, treat as `modifier = 0.0` or fallback to very low yield.

## REFACTOR Phase: Quality & Design

- **Resource Pipe**: Future spec for piped water. For now, abstract consumption from global `ColonyResources`.
- **Visuals**: bubbling water sound/particle effect (Spec 060/Particles).
- **Balance**: Ensure water cost isn't too punishing early game.

## Acceptance Criteria

- [ ] `BuildingType::HydroponicsBay` exists.
- [ ] Consumes Water from `ColonyResources`.
- [ ] Requires Power (via `PowerConsumer` check).
- [ ] Produces 2x Food compared to Farm.
- [ ] Ignores Season penalties.
- [ ] Stops production if Water or Power is missing.

## Technical Guidance

- Use `Spec 042`'s `PowerConsumer` component. The `produce_food_system` should query `Option<&PowerConsumer>` and check `pc.is_powered` (or equivalent).
- Don't hardcode "Water" item if `Spec 096` introduced a `WaterGrid`. If `WaterGrid` is for terrain fluid, and `ColonyResources.water` is for stockpiled water, use `ColonyResources`.
