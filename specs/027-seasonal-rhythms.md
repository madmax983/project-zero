# 027: Seasonal Rhythms

## Overview

Introduces a cyclic seasonal system (Spring, Summer, Autumn, Winter) that affects the simulation. The primary impact for this MVP is on food production, creating a survival rhythm where players must stockpile food during the harvest (Autumn) to survive the scarcity of Winter.

## Dependencies

- `008` — Farm building (to modify production)
- `010` — Chronicle system (to log season changes)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/seasons.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::farm::{produce_food_system, Farm, ColonyResources};
    use crate::layer1::pop::Pop;
    use crate::layer3::chronicle::Chronicle;

    #[test]
    fn test_season_variants() {
        let _ = Season::Spring;
        let _ = Season::Summer;
        let _ = Season::Autumn;
        let _ = Season::Winter;
    }

    #[test]
    fn test_season_next() {
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Summer.next(), Season::Autumn);
        assert_eq!(Season::Autumn.next(), Season::Winter);
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    #[test]
    fn test_season_clock_default() {
        let clock = SeasonClock::default();
        assert_eq!(clock.current, Season::Spring);
        assert_eq!(clock.progress, 0.0);
        assert!(clock.season_length_ticks > 0);
    }

    #[test]
    fn test_advance_seasons_system_progress() {
        let mut world = World::new();
        let mut clock = SeasonClock::default();
        clock.season_length_ticks = 100;
        world.insert_resource(clock);
        world.insert_resource(Chronicle::default()); // Required for events

        // Advance 1 tick
        advance_seasons_system(&mut world);

        let clock = world.resource::<SeasonClock>();
        assert!(clock.progress > 0.0);
        assert_eq!(clock.current, Season::Spring);
    }

    #[test]
    fn test_advance_seasons_system_transition() {
        let mut world = World::new();
        let mut clock = SeasonClock::default();
        clock.season_length_ticks = 10;
        clock.progress = 0.95; // Almost done
        world.insert_resource(clock);
        world.insert_resource(Chronicle::default());

        // Run enough times to roll over
        advance_seasons_system(&mut world);

        let clock = world.resource::<SeasonClock>();
        assert_eq!(clock.current, Season::Summer);
        assert!(clock.progress < 0.2); // Should have reset/wrapped
    }

    #[test]
    fn test_advance_seasons_logs_chronicle_event() {
        let mut world = World::new();
        let mut clock = SeasonClock::default();
        clock.season_length_ticks = 10;
        clock.progress = 0.99;
        world.insert_resource(clock);
        world.insert_resource(Chronicle::default());

        advance_seasons_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("Summer"));
    }

    #[test]
    fn test_farm_production_winter_penalty() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Set season to Winter
        let mut clock = SeasonClock::default();
        clock.current = Season::Winter;
        world.insert_resource(clock);

        // Setup farm
        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        produce_food_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        // Winter = 0.5x multiplier. Base is 0.005. So expected ~0.0025
        assert!(resources.food > 0.0);
        assert!(resources.food < 0.005); // Should be less than base
    }

    #[test]
    fn test_farm_production_summer_bonus() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Set season to Summer
        let mut clock = SeasonClock::default();
        clock.current = Season::Summer;
        world.insert_resource(clock);

        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        produce_food_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        // Summer = 1.2x. Base 0.005 -> 0.006
        assert!(resources.food > 0.005);
    }
}
```

**Test Coverage Requirements:**
- Season: variants, next() logic
- SeasonClock: default values
- advance_seasons_system: increments progress, handles transition, triggers chronicle
- produce_food_system: correctly applies modifiers for different seasons
- Coverage ≥85% for `layer1/seasons.rs`
- Coverage ≥85% for `layer1/farm.rs` (regression check)

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### 1. Define Season System

```rust
// src/layer1/seasons.rs

use bevy_ecs::prelude::*;
use crate::layer3::chronicle::{Chronicle, EventImportance};

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
            Self::Spring => Self::Summer,
            Self::Summer => Self::Autumn,
            Self::Autumn => Self::Winter,
            Self::Winter => Self::Spring,
        }
    }

    pub fn production_modifier(&self) -> f32 {
        match self {
            Self::Spring => 1.0,
            Self::Summer => 1.2,
            Self::Autumn => 1.5,
            Self::Winter => 0.5,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Autumn => "Autumn",
            Self::Winter => "Winter",
        }
    }
}

#[derive(Resource)]
pub struct SeasonClock {
    pub current: Season,
    pub progress: f32, // 0.0 to 1.0
    pub season_length_ticks: u64,
}

impl Default for SeasonClock {
    fn default() -> Self {
        Self {
            current: Season::Spring,
            progress: 0.0,
            season_length_ticks: 3000, // ~30 seconds at 1x speed (100 TPS)
        }
    }
}

pub fn advance_seasons_system(world: &mut World) {
    let increment = {
        let clock = world.resource::<SeasonClock>();
        1.0 / (clock.season_length_ticks as f32)
    };

    let mut transition = None;

    {
        let mut clock = world.resource_mut::<SeasonClock>();
        clock.progress += increment;

        if clock.progress >= 1.0 {
            clock.progress = 0.0;
            let next = clock.current.next();
            clock.current = next;
            transition = Some(next);
        }
    }

    if let Some(new_season) = transition {
        // Log to chronicle
        // Note: In a real ECS, we might want to query tick from SimulationTime,
        // but for now passing 0 or adding SimulationTime dependency is fine.
        // Let's assume we can just add the event.
        let mut chronicle = world.resource_mut::<Chronicle>();
        // Using a placeholder tick or fetching it if SimulationTime is available
        // For simplicity in this system function, we'll omit tick fetching logic
        // unless we add SimulationTime to the signature or fetch it.
        // Let's assume the user will inject SimulationTime if needed,
        // or we just use 0/placeholder if not critical for this test.
        // Better:
        chronicle.add_event(
            0, // We can refine this to use actual tick
            format!("The season turns to {}.", new_season.display_name()),
            EventImportance::Major,
        );
    }
}
```

### 2. Update Farm Production

```rust
// src/layer1/farm.rs

use super::seasons::SeasonClock;

// Update produce_food_system signature and logic
pub fn produce_food_system(world: &mut World) {
    let modifier = if let Some(clock) = world.get_resource::<SeasonClock>() {
        clock.current.production_modifier()
    } else {
        1.0
    };

    // ... existing logic ...
    // Inside the calculation loop:
    // production = base_production * modifier

    let production_from_farms: f32 = {
        let mut query = world.query::<&Farm>();
        query.iter(world)
            .map(|farm| {
                // ... worker counting ...
                count * FOOD_PER_WORKER_PER_TICK * modifier
            })
            .sum()
    };
    // ...
}
```

### 3. Integration

- Register `SeasonClock` in `main.rs`.
- Add `advance_seasons_system` to the simulation schedule in `main.rs`.
- Add `mod seasons;` to `src/layer1/mod.rs`.

## REFACTOR Phase: Quality & Design

- **SimulationTime Dependency**: `advance_seasons_system` should properly fetch `SimulationTime` to log the correct tick in the Chronicle.
- **Config**: Move `season_length_ticks` to a constant or config file.
- **UI**: Display the current season in the `InfoPanel` (Spec 003).
- **Interpolation**: Use `progress` to blend colors in the map renderer (future feature).

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `Season` and `SeasonClock` implemented.
- [ ] Chronicle logs "The season turns to X" events.
- [ ] Farm production varies: Winter is visibly slower, Autumn is abundant.
- [ ] Info Panel shows current Season (optional but recommended).

## Technical Guidance

- **Circular Imports**: `farm.rs` needs `seasons.rs` (for `SeasonClock` type). This is fine as long as `seasons.rs` doesn't import `farm.rs`.
- **System Order**: Run `advance_seasons_system` BEFORE `produce_food_system` so production uses the *current* season state.
- **Float Precision**: Comparing `progress >= 1.0` is safe here as it's an accumulator, but ensure `season_length_ticks` isn't 0.

## Questions

*Builder: None.*
