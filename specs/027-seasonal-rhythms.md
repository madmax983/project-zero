# 027: Seasonal Rhythms

## Overview

Implement a cyclical seasonal system (Spring, Summer, Autumn, Winter) that influences the colony simulation. The primary mechanic for this spec is varying food production from farms, creating a survival loop where players must stockpile food during productive seasons to survive the winter.

## Dependencies

- `010` — Chronicle System (provides `TICKS_PER_YEAR`)
- `008` — Farm Building (provides `produce_food_system`)
- `001` — SimulationTime

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/seasons.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chronicle::{Chronicle, TICKS_PER_YEAR};
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_season_enum_cycling() {
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Summer.next(), Season::Autumn);
        assert_eq!(Season::Autumn.next(), Season::Winter);
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    #[test]
    fn test_season_state_default() {
        let state = SeasonState::default();
        assert_eq!(state.current_season, Season::Spring);
    }

    #[test]
    fn test_advance_season_system_initial() {
        let mut world = World::new();
        world.insert_resource(SeasonState::default());
        world.insert_resource(SimulationTime { tick: 0, ..Default::default() });
        world.insert_resource(Chronicle::default());

        advance_season_system(&mut world);

        let state = world.resource::<SeasonState>();
        assert_eq!(state.current_season, Season::Spring);
    }

    #[test]
    fn test_advance_season_system_transition() {
        let mut world = World::new();
        world.insert_resource(SeasonState::default());
        world.insert_resource(Chronicle::default());

        // Ticks per season = 1000 / 4 = 250
        // Spring: 0-249, Summer: 250-499
        world.insert_resource(SimulationTime { tick: 250, ..Default::default() });

        advance_season_system(&mut world);

        let state = world.resource::<SeasonState>();
        assert_eq!(state.current_season, Season::Summer);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("Summer"));
    }

    #[test]
    fn test_advance_season_system_no_spam() {
        let mut world = World::new();
        world.insert_resource(SeasonState {
            current_season: Season::Summer,
            ..Default::default()
        });
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime { tick: 251, ..Default::default() });

        advance_season_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert!(chronicle.events.is_empty(), "Should not add event if season hasn't changed");
    }

    #[test]
    fn test_get_food_modifier() {
        assert!((Season::Spring.food_modifier() - 1.0).abs() < f32::EPSILON);
        assert!((Season::Summer.food_modifier() - 1.2).abs() < f32::EPSILON);
        assert!((Season::Autumn.food_modifier() - 1.5).abs() < f32::EPSILON);
        assert!((Season::Winter.food_modifier() - 0.5).abs() < f32::EPSILON);
    }
}

// src/layer1/farm.rs - Add/Update tests

#[cfg(test)]
mod seasonal_tests {
    use super::*;
    use crate::layer1::seasons::{Season, SeasonState};

    #[test]
    fn test_produce_food_system_winter() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SeasonState { current_season: Season::Winter, ..Default::default() });

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        produce_food_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        // Base is 0.005. Winter mod is 0.5. Result should be 0.0025.
        // Use approximate check
        let expected = 0.0025;
        assert!((resources.food - expected).abs() < 0.0001, "Winter production should be halved");
    }

    #[test]
    fn test_produce_food_system_autumn() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(SeasonState { current_season: Season::Autumn, ..Default::default() });

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        produce_food_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        // Base is 0.005. Autumn mod is 1.5. Result should be 0.0075.
        let expected = 0.0075;
        assert!((resources.food - expected).abs() < 0.0001, "Autumn production should be boosted");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Seasons (`src/layer1/seasons.rs`)

```rust
// src/layer1/seasons.rs

use bevy_ecs::prelude::*;
use crate::layer1::chronicle::{Chronicle, EventImportance, TICKS_PER_YEAR};
use crate::shared::time::SimulationTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn next(&self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    pub fn food_modifier(&self) -> f32 {
        match self {
            Season::Spring => 1.0,
            Season::Summer => 1.2,
            Season::Autumn => 1.5,
            Season::Winter => 0.5,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }
}

#[derive(Resource, Default)]
pub struct SeasonState {
    pub current_season: Season,
}

pub fn advance_season_system(world: &mut World) {
    let tick = world.resource::<SimulationTime>().tick;
    let ticks_per_season = TICKS_PER_YEAR / 4;

    // Calculate expected season based on tick
    // 0-249: Spring (0)
    // 250-499: Summer (1)
    // 500-749: Autumn (2)
    // 750-999: Winter (3)
    let season_index = (tick / ticks_per_season) % 4;
    let new_season = match season_index {
        0 => Season::Spring,
        1 => Season::Summer,
        2 => Season::Autumn,
        _ => Season::Winter,
    };

    let mut season_changed = false;
    let mut old_season_name = "";

    {
        let mut state = world.resource_mut::<SeasonState>();
        if state.current_season != new_season {
            old_season_name = state.current_season.name();
            state.current_season = new_season;
            season_changed = true;
        }
    }

    if season_changed {
        world.resource_mut::<Chronicle>().add_event(
            tick,
            format!("The season turns. {} has arrived.", new_season.name()),
            EventImportance::Standard, // Seasonal changes are standard events
        );
    }
}
```

### 2. Update Farm Logic (`src/layer1/farm.rs`)

Modify `produce_food_system` to apply the seasonal modifier.

```rust
// src/layer1/farm.rs

use crate::layer1::seasons::SeasonState; // Import SeasonState

pub fn produce_food_system(world: &mut World) {
    let modifier = world.get_resource::<SeasonState>()
        .map(|s| s.current_season.food_modifier())
        .unwrap_or(1.0);

    let mut total_production = 0.0;

    // Use a scope to drop the borrow on world from the query
    let production_from_farms: f32 = {
        let mut query = world.query::<&Farm>();
        query
            .iter(world)
            .map(|farm| {
                #[allow(clippy::cast_precision_loss)]
                let count = farm
                    .workers
                    .iter()
                    .filter(|&&e| world.get_entity(e).is_ok())
                    .count() as f32;
                count * FOOD_PER_WORKER_PER_TICK
            })
            .sum()
    };

    total_production += production_from_farms * modifier; // Apply modifier

    if total_production > 0.0 {
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.food += total_production;
    }
}
```

### 3. Register System and Module

- Add `pub mod seasons;` to `src/layer1/mod.rs`.
- Add `pub use seasons::*;` to `src/layer1/mod.rs`.
- In `src/main.rs`:
  - `world.insert_resource(SeasonState::default());`
  - Add `advance_season_system(&mut world);` to the simulation loop (e.g., before `produce_food_system`).

## REFACTOR Phase: Quality & Design

- **Configuration**: Move `FOOD_PER_WORKER_PER_TICK` and season modifiers to a config file/struct.
- **Season Length**: `TICKS_PER_YEAR / 4` assumes clean division. If `TICKS_PER_YEAR` changes, seasons might drift if not careful.
- **UI**: Display the current season in the Status Bar or Info Panel (future task).

## Acceptance Criteria

- [ ] `Season` enum exists with 4 variants.
- [ ] `SeasonState` tracks the current season.
- [ ] `advance_season_system` updates season based on global tick count.
- [ ] Chronicle records season changes.
- [ ] Farm output is multiplied by 1.5 in Autumn and 0.5 in Winter.
- [ ] `cargo test` passes.

## Questions

- Should seasons affect movement speed? (Deferred to future spec)
- Should seasons affect temperature/needs? (Deferred to future spec)
