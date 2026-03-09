# 065: Day/Night Cycle

## Overview

Introduces a **Day/Night Cycle** to the colony simulation. The cycle progresses through `Dawn`, `Day`, `Dusk`, and `Night` phases based on simulation ticks. This cycle drives:
1.  **Ambient Light**: The global light level (`AmbientLight`) changes dynamically, creating visual atmosphere and gameplay constraints (darkness penalties).
2.  **Circadian Rhythms**: Pops experience increased `Rest` decay at night or prefer to sleep during `Night` hours.

This feature adds temporal depth to the simulation, making "Time" a tangible resource and setting the stage for nocturnal threats or solar power mechanics.

## Dependencies

- `013` — Schedule System Ordering (`SimulationTime`)
- `053` — Lighting System (`AmbientLight`, `LightMap`)
- `027` — Seasonal Rhythms (Future integration for day length)
- `005` — Pop Needs (`Rest` need)

## RED Phase: Tests First

Write these tests in `src/layer1/day_night_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay, update_day_night_cycle_system, update_ambient_light_from_cycle_system, circadian_rhythm_system};
    use crate::layer1::lighting::AmbientLight;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::shared::time::SimulationTime;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(DayNightCycle::default());
        world.insert_resource(AmbientLight::default());
        world
    }

    #[test]
    fn test_initialization() {
        let world = setup_world();
        let cycle = world.resource::<DayNightCycle>();
        assert_eq!(cycle.time_of_day, TimeOfDay::Day); // Default start
        assert_eq!(cycle.day_count, 0);
    }

    #[test]
    fn test_cycle_progression() {
        let mut world = setup_world();

        // Advance time to Night
        // Assuming Day Length = 100 ticks for test simplicity (configurable)
        // Dawn: 0-25, Day: 25-75, Dusk: 75-85, Night: 85-100
        {
            let mut time = world.resource_mut::<SimulationTime>();
            time.tick = 90;
        }

        world.run_system_once(update_day_night_cycle_system).unwrap();

        let cycle = world.resource::<DayNightCycle>();
        assert_eq!(cycle.time_of_day, TimeOfDay::Night);
    }

    #[test]
    fn test_day_increment() {
        let mut world = setup_world();

        // Cross day boundary (tick 99 -> 100)
        {
            let mut time = world.resource_mut::<SimulationTime>();
            time.tick = 101;
        }

        world.run_system_once(update_day_night_cycle_system).unwrap();

        let cycle = world.resource::<DayNightCycle>();
        assert_eq!(cycle.day_count, 1);
        assert_eq!(cycle.time_of_day, TimeOfDay::Dawn); // Start of new day
    }

    #[test]
    fn test_ambient_light_update() {
        let mut world = setup_world();

        // Set to Night
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        world.run_system_once(update_ambient_light_from_cycle_system).unwrap();

        let ambient = world.resource::<AmbientLight>();
        assert!(ambient.level < 0.3); // Should be dark
    }

    #[test]
    fn test_circadian_rhythm_rest_decay() {
        let mut world = setup_world();

        // Spawn Pop
        let pop = world.spawn((
            Pop,
            Needs { rest: 1.0, ..Default::default() }
        )).id();

        // Set to Day
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Day;
        }

        // Run system
        world.run_system_once(circadian_rhythm_system).unwrap();
        let rest_day = world.get::<Needs>(pop).unwrap().rest;

        // Reset
        world.get_mut::<Needs>(pop).unwrap().rest = 1.0;

        // Set to Night
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        // Run system
        world.run_system_once(circadian_rhythm_system).unwrap();
        let rest_night = world.get::<Needs>(pop).unwrap().rest;

        // Verify decay was stronger at night (lower resulting rest)
        // Note: circadian_rhythm_system might apply *additional* decay or modify a multiplier resource.
        // For this test, we assume it directly modifies Needs or applies a debuff.
        assert!(rest_night < rest_day, "Pops should tire faster at night");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `DayNightCycle` (`src/layer1/day_night.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::shared::time::SimulationTime;
use crate::layer1::lighting::AmbientLight;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;

/// Phases of the day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeOfDay {
    Dawn,
    #[default]
    Day,
    Dusk,
    Night,
}

/// Tracks the progress of the day.
#[derive(Resource)]
pub struct DayNightCycle {
    pub time_of_day: TimeOfDay,
    pub day_count: u32,
    pub ticks_per_day: u64,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: TimeOfDay::Day,
            day_count: 0,
            ticks_per_day: 250, // 4 days per year (1000 ticks)
        }
    }
}

pub fn update_day_night_cycle_system(
    time: Res<SimulationTime>,
    mut cycle: ResMut<DayNightCycle>,
) {
    let tick = time.tick;
    let ticks_in_day = cycle.ticks_per_day;

    // Calculate current tick within the day
    let day_tick = tick % ticks_in_day;
    cycle.day_count = (tick / ticks_in_day) as u32;

    // Define phases (simple hardcoded thresholds for MVP)
    // Dawn: 0-10%
    // Day: 10-75%
    // Dusk: 75-85%
    // Night: 85-100%
    let pct = day_tick as f32 / ticks_in_day as f32;

    cycle.time_of_day = if pct < 0.1 {
        TimeOfDay::Dawn
    } else if pct < 0.75 {
        TimeOfDay::Day
    } else if pct < 0.85 {
        TimeOfDay::Dusk
    } else {
        TimeOfDay::Night
    };
}
```

### 2. Implement Ambient Light Integration

```rust
pub fn update_ambient_light_from_cycle_system(
    cycle: Res<DayNightCycle>,
    mut ambient: ResMut<AmbientLight>,
) {
    ambient.level = match cycle.time_of_day {
        TimeOfDay::Dawn => 0.6,
        TimeOfDay::Day => 1.0,
        TimeOfDay::Dusk => 0.5,
        TimeOfDay::Night => 0.2, // Moonlight
    };
}
```

### 3. Implement Circadian Rhythms

```rust
/// Increases Rest decay during Night.
pub fn circadian_rhythm_system(
    cycle: Res<DayNightCycle>,
    mut query: Query<&mut Needs, With<Pop>>,
) {
    // Only apply extra decay at night
    if cycle.time_of_day != TimeOfDay::Night {
        return;
    }

    let extra_decay = 0.0005; // Additional fatigue per tick

    for mut needs in &mut query {
        needs.rest = (needs.rest - extra_decay).max(0.0);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Smooth Transitions**: Instead of jumping from 0.2 to 0.6, interpolate `ambient.level` based on exact tick count for smooth sunrises/sunsets.
- **Seasonal Variation**: Integrate with `SeasonState` (`027`) to change day length.
    - Winter: Night starts earlier (pct > 0.6).
    - Summer: Night starts later (pct > 0.9).
- **UI**: Add a clock/sun icon to the UI status bar.
- **Work Shifts**: Use `DayNightCycle` to trigger "End of Shift" logic for future Scheduling features.

## Acceptance Criteria

- [ ] `DayNightCycle` resource tracks phases (`Dawn`, `Day`, `Dusk`, `Night`) correctly.
- [ ] `AmbientLight` changes automatically based on time of day.
- [ ] Pops get tired faster at night (Circadian Rhythm).
- [ ] `cargo test` passes for new module.
- [ ] `053` Lighting tests still pass (might need update if `AmbientLight` defaults changed).

## Technical Guidance

- Register `DayNightCycle` resource in `main.rs`.
- Add systems to `SimulationSchedule`:
    - `update_day_night_cycle_system` (Early Update)
    - `update_ambient_light_from_cycle_system` (Before Lighting System)
    - `circadian_rhythm_system` (Update)

## Questions

- Should we visualize the sun/moon position? (Not for MVP).
- Does artificial light stop the "Circadian Rhythm" penalty? (Future feature: "Artificial Lights" trait or logic).
  - *Architect:* No, artificial lights do not stop the penalty in the MVP.
