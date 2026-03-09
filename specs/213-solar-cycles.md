# 213: Solar Cycles

## Overview

Implements a multi-year **Solar Cycle** that affects global conditions, primarily **Solar Power** generation. The local star is dynamic, cycling through phases of activity that impact the colony's energy infrastructure.

This adds a long-term rhythm (4 years) on top of the short-term seasonal rhythm (1 year).

### Cycle Phases (4 Years Total)
1.  **Solar Minimum** (Year 1): Low activity. Solar power is reduced (80%).
2.  **Solar Rising** (Year 2): Increasing activity. Solar power is normal (100%).
3.  **Solar Maximum** (Year 3): Peak activity. Solar power is boosted (150%). High radiation risk (future spec).
4.  **Solar Falling** (Year 4): Decreasing activity. Solar power is above average (120%).

## Dependencies

- `010` — Chronicle System (provides `TICKS_PER_YEAR`)
- `042` — Energy System (provides `PowerSource`)
- `001` — SimulationTime

## RED Phase: Tests First

Write these tests in `src/layer1/solar_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::solar::{SolarCycle, SolarCycleState, SolarPower, update_solar_cycle_system, update_solar_output_system};
    use crate::layer1::chronicle::TICKS_PER_YEAR;
    use crate::shared::time::SimulationTime;
    use crate::layer1::energy::PowerSource;

    #[test]
    fn test_solar_cycle_phases() {
        assert_eq!(SolarCycle::Minimum.next(), SolarCycle::Rising);
        assert_eq!(SolarCycle::Rising.next(), SolarCycle::Maximum);
        assert_eq!(SolarCycle::Maximum.next(), SolarCycle::Falling);
        assert_eq!(SolarCycle::Falling.next(), SolarCycle::Minimum);
    }

    #[test]
    fn test_solar_modifier() {
        assert!((SolarCycle::Minimum.power_modifier() - 0.8).abs() < f32::EPSILON);
        assert!((SolarCycle::Rising.power_modifier() - 1.0).abs() < f32::EPSILON);
        assert!((SolarCycle::Maximum.power_modifier() - 1.5).abs() < f32::EPSILON);
        assert!((SolarCycle::Falling.power_modifier() - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cycle_update_system() {
        let mut world = World::new();
        world.insert_resource(SolarCycleState::default()); // Defaults to Minimum
        world.insert_resource(SimulationTime { tick: 0, ..Default::default() });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_solar_cycle_system);

        // Year 0 (0 to TICKS_PER_YEAR-1) -> Minimum
        schedule.run(&mut world);
        assert_eq!(world.resource::<SolarCycleState>().current_cycle, SolarCycle::Minimum);

        // Year 1 (TICKS_PER_YEAR to 2*TICKS_PER_YEAR-1) -> Rising
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR;
        schedule.run(&mut world);
        assert_eq!(world.resource::<SolarCycleState>().current_cycle, SolarCycle::Rising);

        // Year 2 -> Maximum
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR * 2;
        schedule.run(&mut world);
        assert_eq!(world.resource::<SolarCycleState>().current_cycle, SolarCycle::Maximum);

        // Year 3 -> Falling
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR * 3;
        schedule.run(&mut world);
        assert_eq!(world.resource::<SolarCycleState>().current_cycle, SolarCycle::Falling);

        // Year 4 -> Minimum
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR * 4;
        schedule.run(&mut world);
        assert_eq!(world.resource::<SolarCycleState>().current_cycle, SolarCycle::Minimum);
    }

    #[test]
    fn test_solar_output_scaling() {
        let mut world = World::new();
        world.insert_resource(SolarCycleState { current_cycle: SolarCycle::Maximum });

        // Spawn a solar panel with Base Output 10.0
        let panel = world.spawn((
            PowerSource { output: 10.0, ..Default::default() },
            SolarPower { base_output: 10.0 },
        )).id();

        // Spawn a generator (non-solar)
        let generator = world.spawn((
            PowerSource { output: 10.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_solar_output_system);

        // Run system
        schedule.run(&mut world);

        // Panel should be boosted by 1.5x -> 15.0
        let panel_power = world.get::<PowerSource>(panel).unwrap();
        assert!((panel_power.output - 15.0).abs() < 0.001);

        // Generator should remain 10.0
        let gen_power = world.get::<PowerSource>(generator).unwrap();
        assert!((gen_power.output - 10.0).abs() < 0.001);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Enums and Resources (`src/layer1/solar.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::chronicle::TICKS_PER_YEAR;
use crate::shared::time::SimulationTime;
use crate::layer1::energy::PowerSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SolarCycle {
    #[default]
    Minimum,
    Rising,
    Maximum,
    Falling,
}

impl SolarCycle {
    pub fn next(&self) -> Self {
        match self {
            Self::Minimum => Self::Rising,
            Self::Rising => Self::Maximum,
            Self::Maximum => Self::Falling,
            Self::Falling => Self::Minimum,
        }
    }

    pub fn power_modifier(&self) -> f32 {
        match self {
            Self::Minimum => 0.8,
            Self::Rising => 1.0,
            Self::Maximum => 1.5,
            Self::Falling => 1.2,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Minimum => "Solar Minimum",
            Self::Rising => "Solar Rising",
            Self::Maximum => "Solar Maximum",
            Self::Falling => "Solar Falling",
        }
    }
}

#[derive(Resource, Default)]
pub struct SolarCycleState {
    pub current_cycle: SolarCycle,
}

/// Component for buildings that generate solar power.
/// Tracks the base output before modifiers.
#[derive(Component, Default)]
pub struct SolarPower {
    pub base_output: f32,
}

pub fn update_solar_cycle_system(mut state: ResMut<SolarCycleState>, time: Res<SimulationTime>) {
    let year = time.tick / TICKS_PER_YEAR;
    let cycle_index = year % 4;

    let new_cycle = match cycle_index {
        0 => SolarCycle::Minimum,
        1 => SolarCycle::Rising,
        2 => SolarCycle::Maximum,
        _ => SolarCycle::Falling,
    };

    if state.current_cycle != new_cycle {
        state.current_cycle = new_cycle;
        // Optionally trigger a Chronicle event here
    }
}

pub fn update_solar_output_system(
    state: Res<SolarCycleState>,
    mut query: Query<(&mut PowerSource, &SolarPower)>,
) {
    let modifier = state.current_cycle.power_modifier();

    for (mut source, solar) in query.iter_mut() {
        source.output = solar.base_output * modifier;
    }
}
```

### 2. Update `BuildingType` and `configure_power`

In `src/layer1/building.rs`:
1.  Add `SolarPanel` to `BuildingType`.
2.  Update `configure_power` to add `SolarPower` component to `SolarPanel`.

```rust
// In configure_power
BuildingType::SolarPanel => {
    entity.insert((
        PowerSource { output: 10.0, ..Default::default() },
        SolarPower { base_output: 10.0 },
        // ... LightSource/etc
    ));
}
```

### 3. Register Systems

Add `update_solar_cycle_system` and `update_solar_output_system` to the scheduler in `src/layer1/mod.rs` or `src/main.rs`.

## REFACTOR Phase: Quality & Design

- **Day/Night Integration**: Solar output should also be 0 at night! The `update_solar_output_system` should check `TimeOfDay` (Spec 065) and multiply by 0 if night (or reduce during dawn/dusk).
- **UI**: Add a solar cycle indicator to the top bar.
- **Events**: Add Chronicle events for cycle changes (especially Maximum/Minimum).

## Acceptance Criteria

- [ ] `SolarCycle` enum exists with 4 phases.
- [ ] `SolarCycleState` tracks the current phase based on game year.
- [ ] `SolarPower` component allows base output tracking.
- [ ] `update_solar_output_system` scales power output based on cycle.
- [ ] `SolarPanel` building exists and uses this system.
- [ ] Tests pass.

## Technical Guidance

- **System Ordering**: Ensure `update_solar_output_system` runs *before* `power_grid_system` (or whatever system calculates grid load) to ensure output is up-to-date.
- **Resource Management**: Remember to register `SolarCycleState` as a resource in `src/main.rs`.
- **Chronicle Integration**: Use `EventWriter<AddChronicleEvent>` in `update_solar_cycle_system` to log changes.

## Questions

- **Q:** Should `SolarPanel` degrade over time (maintenance)?
  - **A:** Not in this spec. Handled by `045` Structure Durability generally.
- **Q:** Does `SolarCycle` affect temperature?
  - **A:** Yes, ideally it should feed into `063` Atmospheric Simulation, but let's keep it scoped to power for MVP.
  - *Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
