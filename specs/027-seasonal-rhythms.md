# 027: Seasonal Rhythms

## Overview

The world of SCALE is not static. Seasons cycle through Spring, Summer, Autumn, and Winter, affecting the colony's survival. This spec introduces the `Season` system, which drives food production rates and provides a rhythmic challenge to the player.

## Dependencies

- `008` — Farm building (food production)
- `010` — Chronicle system (time tracking)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/season.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::shared::time::SimulationTime;
    use crate::layer1::chronicle::Chronicle;

    #[test]
    fn test_season_variants() {
        let _ = Season::Spring;
        let _ = Season::Summer;
        let _ = Season::Autumn;
        let _ = Season::Winter;
    }

    #[test]
    fn test_season_from_tick() {
        // Year = 1000 ticks
        // Spring: 0-249
        // Summer: 250-499
        // Autumn: 500-749
        // Winter: 750-999

        assert_eq!(Season::from_tick(0), Season::Spring);
        assert_eq!(Season::from_tick(249), Season::Spring);
        assert_eq!(Season::from_tick(250), Season::Summer);
        assert_eq!(Season::from_tick(499), Season::Summer);
        assert_eq!(Season::from_tick(500), Season::Autumn);
        assert_eq!(Season::from_tick(749), Season::Autumn);
        assert_eq!(Season::from_tick(750), Season::Winter);
        assert_eq!(Season::from_tick(999), Season::Winter);
        assert_eq!(Season::from_tick(1000), Season::Spring); // Year 2
    }

    #[test]
    fn test_season_display() {
        assert_eq!(Season::Spring.as_str(), "Spring");
        assert_eq!(Season::Summer.as_str(), "Summer");
        assert_eq!(Season::Autumn.as_str(), "Autumn");
        assert_eq!(Season::Winter.as_str(), "Winter");
    }

    #[test]
    fn test_food_modifier() {
        assert!((Season::Spring.food_modifier() - 1.0).abs() < f32::EPSILON);
        assert!((Season::Summer.food_modifier() - 1.2).abs() < f32::EPSILON);
        assert!((Season::Autumn.food_modifier() - 1.0).abs() < f32::EPSILON);
        assert!((Season::Winter.food_modifier() - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_season_state_resource_default() {
        let state = SeasonState::default();
        assert_eq!(state.current_season, Season::Spring);
        assert!((state.food_modifier - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_update_season_system_initial() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default()); // tick 0
        world.insert_resource(SeasonState::default());
        world.insert_resource(Chronicle::default());

        update_season_system(&mut world);

        let state = world.resource::<SeasonState>();
        assert_eq!(state.current_season, Season::Spring);
    }

    #[test]
    fn test_update_season_system_change() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 250, ..Default::default() }); // Summer start
        world.insert_resource(SeasonState {
            current_season: Season::Spring, // Old state
            ..Default::default()
        });
        world.insert_resource(Chronicle::default());

        update_season_system(&mut world);

        let state = world.resource::<SeasonState>();
        assert_eq!(state.current_season, Season::Summer);
        assert!((state.food_modifier - 1.2).abs() < f32::EPSILON);

        // Check chronicle event
        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("Summer"));
    }

    #[test]
    fn test_update_season_system_no_change_no_event() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() }); // Still Spring
        world.insert_resource(SeasonState::default()); // Already Spring
        world.insert_resource(Chronicle::default());

        update_season_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert!(chronicle.events.is_empty());
    }
}

// src/layer1/farm.rs - Add to existing tests

#[test]
fn test_produce_food_with_winter_penalty() {
    let mut world = World::new();
    world.insert_resource(ColonyResources::default());
    // Winter penalty: 0.2x multiplier
    world.insert_resource(SeasonState {
        current_season: Season::Winter,
        food_modifier: 0.2,
    });

    let worker = world.spawn(Pop).id();
    let mut farm = Farm::default();
    farm.workers.push(worker);
    world.spawn(farm);

    produce_food_system(&mut world);

    let resources = world.resource::<ColonyResources>();
    // Base is 0.005 per worker
    // Expected: 0.005 * 0.2 = 0.001
    assert!((resources.food - 0.001).abs() < 0.0001, "Winter should reduce food production");
}

#[test]
fn test_produce_food_with_summer_bonus() {
    let mut world = World::new();
    world.insert_resource(ColonyResources::default());
    // Summer bonus: 1.2x multiplier
    world.insert_resource(SeasonState {
        current_season: Season::Summer,
        food_modifier: 1.2,
    });

    let worker = world.spawn(Pop).id();
    let mut farm = Farm::default();
    farm.workers.push(worker);
    world.spawn(farm);

    produce_food_system(&mut world);

    let resources = world.resource::<ColonyResources>();
    // Base is 0.005 per worker
    // Expected: 0.005 * 1.2 = 0.006
    assert!((resources.food - 0.006).abs() < 0.0001, "Summer should boost food production");
}
```

**Test Coverage Requirements:**
- Season: variants, `from_tick`, `as_str`, `food_modifier`.
- SeasonState: default, updates via system.
- update_season_system: triggers chronicle events on change, updates modifier.
- produce_food_system: correctly applies `SeasonState.food_modifier`.
- Coverage ≥85% for `src/layer1/season.rs`.

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Season Enum and Logic

```rust
// src/layer1/season.rs

use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;
use crate::layer1::chronicle::{Chronicle, EventImportance};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Determines the season based on the simulation tick.
    /// Year = 1000 ticks. Season = 250 ticks.
    pub fn from_tick(tick: u64) -> Self {
        let year_tick = tick % 1000;
        if year_tick < 250 {
            Season::Spring
        } else if year_tick < 500 {
            Season::Summer
        } else if year_tick < 750 {
            Season::Autumn
        } else {
            Season::Winter
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }

    pub fn food_modifier(&self) -> f32 {
        match self {
            Season::Spring => 1.0,
            Season::Summer => 1.2,
            Season::Autumn => 1.0,
            Season::Winter => 0.2,
        }
    }
}

#[derive(Resource)]
pub struct SeasonState {
    pub current_season: Season,
    pub food_modifier: f32,
}

impl Default for SeasonState {
    fn default() -> Self {
        Self {
            current_season: Season::Spring,
            food_modifier: 1.0,
        }
    }
}

pub fn update_season_system(world: &mut World) {
    let tick = world.resource::<SimulationTime>().tick;
    let new_season = Season::from_tick(tick);

    // We can't query and mutate SeasonState/Chronicle at the same time if we aren't careful.
    // Use resource_scope or check first.
    let season_changed = {
        let state = world.resource::<SeasonState>();
        state.current_season != new_season
    };

    if season_changed {
        let mut state = world.resource_mut::<SeasonState>();
        state.current_season = new_season;
        state.food_modifier = new_season.food_modifier();

        let mut chronicle = world.resource_mut::<Chronicle>();
        chronicle.add_event(
            tick,
            format!("The season changes to {}.", new_season.as_str()),
            EventImportance::Standard, // Or Major if we prefer
        );
    }
}
```

### Farm Modification

Modify `src/layer1/farm.rs`:

```rust
// src/layer1/farm.rs

use crate::layer1::season::SeasonState; // Import SeasonState

// In produce_food_system:

pub fn produce_food_system(world: &mut World) {
    let mut total_production = 0.0;

    // Get modifier (default to 1.0 if resource missing, though it should exist)
    let modifier = world.get_resource::<SeasonState>()
        .map(|s| s.food_modifier)
        .unwrap_or(1.0);

    // ... existing query logic ...
    let production_from_farms: f32 = {
        // ... (same as before) ...
            .sum()
    };

    // Apply modifier
    total_production += production_from_farms * modifier;

    if total_production > 0.0 {
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.food += total_production;
    }
}
```

### Module Registration

```rust
// src/layer1/mod.rs
pub mod season;
pub use season::*;
```

```rust
// src/main.rs
// In main():
world.insert_resource(SeasonState::default());

// In simulation tick loop (after time update, before production?):
// Time updates at END of tick usually.
// Best place: Start of tick, before farm production.
update_season_system(&mut world);
```

### UI Display (Optional but good)

Add season to `render_status_bar` in `src/main.rs`.

```rust
// src/main.rs
// In render_status_bar:
let season = world.resource::<SeasonState>().current_season.as_str();
// Add to status text: e.g., "Year 1 | Spring | Tick 150"
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1.  **Magic Numbers**: 1000 ticks/year, 250 ticks/season.
    -   *Improvement*: Move to `TimeConfig` or `SeasonConfig`.
2.  **Hardcoded Modifiers**: 1.2, 0.2 in code.
    -   *Improvement*: Move to data/config file for balancing.
3.  **Visual Feedback**: Map doesn't change color.
    -   *Improvement*: Add simple color tinting to terrain rendering based on season (White for Winter, Orange for Autumn).

### Performance Considerations

-   `update_season_system` runs every tick but does very little math. Negligible impact.
-   `Season::from_tick` is a few comparisons. Fast.

### API Design Notes

-   `SeasonState` is a Resource, making it globally accessible.
-   `food_modifier` is pre-calculated in the resource to avoid re-calculating it in every system that needs it.
-   Future: Add `movement_modifier`, `mood_modifier`, etc. to `SeasonState`.

## Acceptance Criteria (Testable!)

-   [ ] All tests in RED phase pass.
-   [ ] `cargo test` returns 0 failures.
-   [ ] `produce_food_system` applies the correct modifier for the season.
-   [ ] Chronicle records "The season changes to X" events.
-   [ ] Winter significantly penalizes food production (0.2x).
-   [ ] Summer boosts food production (1.2x).

## Technical Guidance

### Common Pitfalls

1.  **System Ordering**: `update_season_system` must run before `produce_food_system` if you want the season change to apply immediately on that tick. Not critical, but cleaner.
2.  **Resource Initialization**: Don't forget `world.insert_resource(SeasonState::default())` in `main.rs`.
3.  **Farm Test Setup**: When testing `produce_food_system`, you MUST insert `SeasonState` into the test world, or handle the `unwrap_or(1.0)` case (though explicit is better for tests).

## Questions

*Builder: add questions here if spec is unclear.*
