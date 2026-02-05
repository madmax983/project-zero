# 027: Seasonal Rhythms

## Overview

Introduces a **Seasonal Cycle** (Spring, Summer, Autumn, Winter) to the simulation.
The colony now "breathes" with the planet.
- **Seasons** cycle automatically based on simulation ticks.
- **Farms** produce varying amounts of food based on the season (Summer > Spring/Autumn > Winter).
- **Visuals** (Future): Can drive palette swaps or snow rendering.

## Dependencies

- `008` — Farm Building (for production modifiers)
- `010` — Chronicle System (for `TICKS_PER_YEAR` constant)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/season_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::season::{Season, CurrentSeason, update_season_system};
    use crate::layer1::farm::{Farm, produce_food_system};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;
    use crate::layer1::chronicle::TICKS_PER_YEAR;

    #[test]
    fn test_season_variants() {
        let _ = Season::Spring;
        let _ = Season::Summer;
        let _ = Season::Autumn;
        let _ = Season::Winter;
    }

    #[test]
    fn test_initial_season() {
        let current = CurrentSeason::default();
        assert_eq!(current.season, Season::Spring);
    }

    #[test]
    fn test_season_progression() {
        let mut world = World::new();
        world.insert_resource(CurrentSeason::default());
        world.insert_resource(SimulationTime::default());

        // TICKS_PER_YEAR = 1000. So each season is 250 ticks.
        // Tick 0 -> Spring
        update_season_system(&mut world);
        assert_eq!(world.resource::<CurrentSeason>().season, Season::Spring);

        // Tick 250 -> Summer
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR / 4;
        update_season_system(&mut world);
        assert_eq!(world.resource::<CurrentSeason>().season, Season::Summer);

        // Tick 500 -> Autumn
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR / 2;
        update_season_system(&mut world);
        assert_eq!(world.resource::<CurrentSeason>().season, Season::Autumn);

        // Tick 750 -> Winter
        world.resource_mut::<SimulationTime>().tick = (TICKS_PER_YEAR * 3) / 4;
        update_season_system(&mut world);
        assert_eq!(world.resource::<CurrentSeason>().season, Season::Winter);

        // Tick 1000 -> Spring (Next Year)
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR;
        update_season_system(&mut world);
        assert_eq!(world.resource::<CurrentSeason>().season, Season::Spring);
    }

    #[test]
    fn test_farm_production_varies_by_season() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Setup a farm with 1 worker
        let worker = world.spawn(Pop).id();
        let mut farm = Farm::default();
        farm.workers.push(worker);
        world.spawn(farm);

        // Helper to run production for a specific season
        let produce_in_season = |world: &mut World, season: Season| -> f32 {
            world.insert_resource(CurrentSeason { season });
            world.resource_mut::<ColonyResources>().food = 0.0;
            produce_food_system(world);
            world.resource::<ColonyResources>().food
        };

        let spring_yield = produce_in_season(&mut world, Season::Spring);
        let summer_yield = produce_in_season(&mut world, Season::Summer);
        let autumn_yield = produce_in_season(&mut world, Season::Autumn);
        let winter_yield = produce_in_season(&mut world, Season::Winter);

        // Verify relationships
        assert!(summer_yield > spring_yield, "Summer should be most productive");
        assert!(spring_yield > winter_yield, "Spring should beat Winter");
        assert!(winter_yield < spring_yield, "Winter should be lowest");

        // Specific multipliers (Spring/Autumn: 1.0, Summer: 1.5, Winter: 0.2)
        // Base is 0.005
        assert!((spring_yield - 0.005).abs() < f32::EPSILON);
        assert!((autumn_yield - 0.005).abs() < f32::EPSILON);
        assert!((summer_yield - 0.0075).abs() < f32::EPSILON);
        assert!((winter_yield - 0.001).abs() < f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Season Resource

```rust
// src/layer1/season.rs

use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;
use crate::layer1::chronicle::TICKS_PER_YEAR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Resource, Default, Debug)]
pub struct CurrentSeason {
    pub season: Season,
}

impl Season {
    pub fn yield_modifier(&self) -> f32 {
        match self {
            Self::Spring | Self::Autumn => 1.0,
            Self::Summer => 1.5,
            Self::Winter => 0.2,
        }
    }
}

pub fn update_season_system(world: &mut World) {
    let tick = world.resource::<SimulationTime>().tick;
    // Calculate position in year (0..TICKS_PER_YEAR)
    let year_progress = tick % TICKS_PER_YEAR;
    let quarter = TICKS_PER_YEAR / 4;

    let new_season = if year_progress < quarter {
        Season::Spring
    } else if year_progress < quarter * 2 {
        Season::Summer
    } else if year_progress < quarter * 3 {
        Season::Autumn
    } else {
        Season::Winter
    };

    let mut current = world.resource_mut::<CurrentSeason>();
    if current.season != new_season {
        current.season = new_season;
    }
}
```

### 2. Update Farm Logic

```rust
// src/layer1/farm.rs
use crate::layer1::season::CurrentSeason;

pub fn produce_food_system(world: &mut World) {
    // ... setup ...

    // Get modifier
    let modifier = world.get_resource::<CurrentSeason>()
        .map(|cs| cs.season.yield_modifier())
        .unwrap_or(1.0);

    let production_from_farms: f32 = {
        // ... query loop ...
            .map(|farm| {
                // ... count workers ...
                count * FOOD_PER_WORKER_PER_TICK * modifier // Apply modifier
            })
            .sum()
    };

    // ... apply ...
}
```

### 3. Register System

Update `src/main.rs` or `src/lib.rs` to include `update_season_system` in the `Update` schedule.

## REFACTOR Phase: Quality & Design

- **Time Constants**: `TICKS_PER_YEAR` is currently in `chronicle.rs`. Consider moving it and the "tick to date" logic to a dedicated `Calendar` module/resource to avoid `season` depending on `chronicle`.
- **Event Notification**: When the season changes, trigger a `ChronicleEvent` (e.g., "Winter has come."). This requires `update_season_system` to access `Chronicle`.
- **Visuals**: Add a `Season` field to `MapDisplay` or similar to tint the terrain in the UI.

## Acceptance Criteria

- [ ] `CurrentSeason` resource tracks the season correctly based on ticks.
- [ ] Cycle is Spring -> Summer -> Autumn -> Winter -> Spring.
- [ ] Farm output matches multipliers: Summer (1.5x), Winter (0.2x), Others (1.0x).
- [ ] `cargo test` passes.

## Technical Guidance

- Ensure `update_season_system` runs *before* `produce_food_system` to ensure accurate yield for the tick.
- Use `f32::EPSILON` for float comparisons in tests.
- Remember to register the new module `mod season;` in `src/layer1/mod.rs`.
