# 027: Seasonal Rhythms

## Overview

Introduces a cyclic seasonal system (Spring, Summer, Autumn, Winter) that influences the simulation.
Primary effects for this spec:
1.  **Farm Yield**: Crops grow better in Summer/Autumn, poorly in Winter.
2.  **Chronicle**: Season changes are significant events recorded in history.

This adds a rhythmic difficulty curve to the survival loop.

## Dependencies

- `008` — Farm Building (yield logic)
- `010` — Chronicle System (logging events)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/seasons.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer3::chronicle::{Chronicle, EventImportance};
    use crate::shared::time::SimulationTime;
    use crate::layer1::farm::Farm;

    #[test]
    fn test_season_enum_cycling() {
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Summer.next(), Season::Autumn);
        assert_eq!(Season::Autumn.next(), Season::Winter);
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    #[test]
    fn test_season_yield_modifiers() {
        // Defined in spec:
        // Spring: 1.0 (Standard)
        // Summer: 1.2 (Growth bonus)
        // Autumn: 1.5 (Harvest time)
        // Winter: 0.2 (Harsh)
        assert!((Season::Spring.yield_modifier() - 1.0).abs() < f32::EPSILON);
        assert!((Season::Summer.yield_modifier() - 1.2).abs() < f32::EPSILON);
        assert!((Season::Autumn.yield_modifier() - 1.5).abs() < f32::EPSILON);
        assert!((Season::Winter.yield_modifier() - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_seasonal_cycle_initialization() {
        let cycle = SeasonalCycle::default();
        assert_eq!(cycle.current, Season::Spring);
        assert_eq!(cycle.last_change_tick, 0);
    }

    #[test]
    fn test_update_season_system_changes_season() {
        let mut world = World::new();
        world.insert_resource(SeasonalCycle::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime { tick: 250, ..Default::default() });

        // Assuming TICKS_PER_SEASON = 250
        update_season_system(&mut world);

        let cycle = world.resource::<SeasonalCycle>();
        assert_eq!(cycle.current, Season::Summer);
        assert_eq!(cycle.last_change_tick, 250);
    }

    #[test]
    fn test_update_season_logs_to_chronicle() {
        let mut world = World::new();
        world.insert_resource(SeasonalCycle::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime { tick: 250, ..Default::default() });

        update_season_system(&mut world);

        let chronicle = world.resource::<Chronicle>();
        assert_eq!(chronicle.events.len(), 1);
        assert!(chronicle.events[0].text.contains("Summer"));
    }

    #[test]
    fn test_no_change_mid_season() {
        let mut world = World::new();
        world.insert_resource(SeasonalCycle::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });

        update_season_system(&mut world);

        let cycle = world.resource::<SeasonalCycle>();
        assert_eq!(cycle.current, Season::Spring);
    }
}
```

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Season Enum & Resource

```rust
// src/layer1/seasons.rs

use bevy_ecs::prelude::*;
use crate::layer3::chronicle::{Chronicle, EventImportance};
use crate::shared::time::SimulationTime;

pub const TICKS_PER_SEASON: u64 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            Self::Spring => Self::Summer,
            Self::Summer => Self::Autumn,
            Self::Autumn => Self::Winter,
            Self::Winter => Self::Spring,
        }
    }

    #[must_use]
    pub fn yield_modifier(&self) -> f32 {
        match self {
            Self::Spring => 1.0,
            Self::Summer => 1.2,
            Self::Autumn => 1.5,
            Self::Winter => 0.2,
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Spring => "Spring",
            Self::Summer => "Summer",
            Self::Autumn => "Autumn",
            Self::Winter => "Winter",
        }
    }
}

#[derive(Resource, Default)]
pub struct SeasonalCycle {
    pub current: Season,
    pub last_change_tick: u64,
}
```

### System

```rust
// src/layer1/seasons.rs

pub fn update_season_system(world: &mut World) {
    let tick = world.resource::<SimulationTime>().tick;

    // Use a scope to allow mutable borrow of cycle, then chronicle
    let (should_change, next_season) = {
        let cycle = world.resource::<SeasonalCycle>();
        if tick >= cycle.last_change_tick + TICKS_PER_SEASON {
            (true, cycle.current.next())
        } else {
            (false, cycle.current)
        }
    };

    if should_change {
        let mut cycle = world.resource_mut::<SeasonalCycle>();
        cycle.current = next_season;
        cycle.last_change_tick = tick;

        // Log event
        let mut chronicle = world.resource_mut::<Chronicle>();
        chronicle.add_event(
            tick,
            format!("The season turns to {}.", next_season.name()),
            EventImportance::Standard,
        );
    }
}
```

### Farm Integration (Patch Required)

This step modifies `src/layer1/farm.rs`.

```rust
// src/layer1/farm.rs

use crate::layer1::seasons::{Season, SeasonalCycle};

pub fn produce_food_system(world: &mut World) {
    // Fetch season modifier
    let season_mod = world
        .get_resource::<SeasonalCycle>()
        .map(|c| c.current.yield_modifier())
        .unwrap_or(1.0);

    // ... inside loop ...
    // let production = base * season_mod;
}
```

## REFACTOR Phase: Quality & Design

- **Strict Time Math**: Instead of `last_change_tick + duration`, use `tick / duration % 4` to prevent drift if system paused or skipped (though simulation should be continuous).
- **Config**: Move `TICKS_PER_SEASON` and yield modifiers to `GameConfig` if we add one.
- **Visuals**: Add season indicator to UI (Status Bar) in future spec.
- **Snow**: Future spec for tile rendering changes.

## Acceptance Criteria

- [ ] `Season` enum exists with 4 variants.
- [ ] `SeasonalCycle` resource tracks current season.
- [ ] Season changes every `TICKS_PER_SEASON` (250 ticks).
- [ ] Chronicle records season changes.
- [ ] Farm production is multiplied by:
    - Spring: 1.0x
    - Summer: 1.2x
    - Autumn: 1.5x
    - Winter: 0.2x
- [ ] All tests pass.

## Technical Guidance

- **Farm Patch**: The `produce_food_system` in `farm.rs` currently iterates and sums directly. You'll need to inject the `SeasonalCycle` resource lookup before the calculation.
- **Circular Deps**: `seasons.rs` depends on `chronicle`, `farm` depends on `seasons`. Ensure `mod.rs` structure allows this (it should, as they are separate modules in layer1).

## Questions

*Builder: add questions here if spec is unclear.*
