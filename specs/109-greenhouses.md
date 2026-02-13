# 109: Greenhouses

## Overview

The **Greenhouse** is a specialized farm building that allows crops to grow at full efficiency regardless of the season. It provides immunity to the Winter food production penalty (which normally reduces output by 50%).
This mechanic is critical for survival in harsh climates or long winters.

## Dependencies

- `008` — Farming (for `produce_food_system` and `Farm` component)
- `027` — Seasonal Rhythms (for `SeasonState` and `food_modifier`)
- `024` — Metal Industry (for construction cost: Metal + Glass? For MVP, just Metal + Stone).

## RED Phase: Tests First

Write these tests in `src/layer1/greenhouse_tests.rs`. They will initially FAIL.

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
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_building_type_greenhouse_exists() {
        let b = BuildingType::Greenhouse;
        assert_eq!(b.label(), "Greenhouse");
        // Ensure it has a distinct character/color in `rendering.rs` later
    }

    #[test]
    fn test_greenhouse_ignores_winter_penalty() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        // Set Season to Winter (0.5 modifier usually)
        world.insert_resource(SeasonState {
            current_season: Season::Winter,
        });

        // Spawn Greenhouse
        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Greenhouse,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        // Run production
        world.run_system_once(produce_food_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        // Normal Farm in Winter: 0.005 * 0.5 = 0.0025
        // Greenhouse in Winter: 0.005 * 1.0 = 0.005
        // Food starts at 10.0
        // Expected: 10.005
        assert!(
            (resources.food - 10.005).abs() < 0.0001,
            "Greenhouse should ignore winter penalty. Food: {}",
            resources.food
        );
    }

    #[test]
    fn test_greenhouse_cost() {
        // Should be expensive
        let cost = BuildingType::Greenhouse.cost(crate::layer1::building::MaterialType::Stone);
        assert!(cost.metal >= 10.0, "Greenhouse should require metal");
        assert!(cost.stone >= 20.0, "Greenhouse should require stone/glass equivalent");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType`

```rust
// src/layer1/building.rs

pub enum BuildingType {
    // ...
    Greenhouse,
}

impl BuildingType {
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Greenhouse => "Greenhouse",
            // ...
        }
    }

    pub const fn char(&self) -> char {
        match self {
            Self::Greenhouse => 'G', // Or specific char
            // ...
        }
    }

    pub const fn cost(&self, material: MaterialType) -> ColonyResources {
        match self {
            Self::Greenhouse => ColonyResources {
                wood: 10.0,
                stone: 20.0,
                metal: 10.0, // Requires Metal Industry (024)
                ..ColonyResources::zeroed()
            },
            // ...
        }
    }
}
```

### 2. Update `produce_food_system`

```rust
// src/layer1/farm.rs

pub fn produce_food_system(
    farm_query: Query<(&crate::layer1::building::Building, &GridPosition), With<Farm>>,
    // ... other params ...
    season: Option<Res<SeasonState>>,
    // ...
) {
    let global_modifier = season.map_or(1.0, |s| s.current_season.food_modifier());

    // ... existing map logic ...

    for (_, pos, action, skills_opt) in &mut pop_query {
        if action.current != ActionType::Farm {
            continue;
        }

        if let Some(building_type) = farm_map.get(pos) {
            // ... efficiency calc ...

            // Calculate modifier based on building type
            let effective_modifier = match building_type {
                crate::layer1::building::BuildingType::Greenhouse => 1.0, // Ignore season
                _ => global_modifier,
            };

            let production = efficiency * FOOD_PER_WORKER_PER_TICK * effective_modifier;

            // ... apply ...
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Season Logic Helper**: Move `effective_modifier` logic to `BuildingType::get_seasonal_modifier(season)`.
- **Soil Fertility**: If Spec 059 is implemented, Greenhouses might also ignore fertility penalties (hydroponics), or act as "High Fertility" tiles. For now, just season immunity.
- **Visuals**: Greenhouses should look distinct (Glass texture/char).

## Acceptance Criteria

- [ ] `BuildingType::Greenhouse` exists.
- [ ] Construction requires Metal.
- [ ] Greenhouses produce food at 100% efficiency in Winter.
- [ ] Normal Farms produce food at 50% efficiency in Winter.
- [ ] Tests pass.

## Technical Guidance

- Ensure `produce_food_system` correctly identifies the building type. The current logic in `farm.rs` already collects `BuildingType` into a map, so checking `match building_type` is easy.
- If Metal (024) is not fully available/balanced, use high Stone cost as placeholder for "Glass".

## Questions

*Builder: add questions here if spec is unclear.*
