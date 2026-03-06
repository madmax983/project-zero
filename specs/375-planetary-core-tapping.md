# 375: Planetary Core Tapping

## Overview

"Infinite energy at the cost of the world."

**Planetary Core Tapping** introduces a massive "Core Drill" that taps into the planet's mantle for limitless geothermal energy (Layer 2 -> Layer 1 interaction). This slowly destabilizes the planet's core, increasing the frequency of earthquakes, volcanic eruptions, and eventually, planetary destruction.

This creates tension: absolute energy supremacy for orbital defenses and decisive battles vs. the literal ticking clock of the planet's lifespan. Tapping the core might power your weapons to win a battle, but the resulting tectonic shift destroys your primary agricultural zone.

## Dependencies

- `042` — Energy System (for generating the immense power)
- `153` — Geological Instability (for triggering earthquakes)

## RED Phase: Tests First

Write these tests in `src/layer1/tech/core_tapping_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::core_tapping::{CoreDrill, PlanetaryStability, process_core_drill_system};
    use crate::layer1::energy::PowerGenerator;
    use crate::layer1::hazards::EarthquakeEvent;

    #[test]
    fn test_core_drill_generates_massive_power() {
        let mut world = World::new();

        let drill = world.spawn((
            CoreDrill { active: true },
            PowerGenerator { output: 0.0 }, // Initially zero
        )).id();

        world.insert_resource(PlanetaryStability { level: 1.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_core_drill_system);
        schedule.run(&mut world);

        // Output should be massive
        let generator = world.get::<PowerGenerator>(drill).unwrap();
        assert!(generator.output >= 10000.0);
    }

    #[test]
    fn test_core_drill_degrades_stability() {
        let mut world = World::new();

        world.spawn((
            CoreDrill { active: true },
            PowerGenerator { output: 0.0 },
        )).id();

        world.insert_resource(PlanetaryStability { level: 1.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_core_drill_system);
        schedule.run(&mut world);

        // Stability should decrease while active
        let stability = world.resource::<PlanetaryStability>();
        assert!(stability.level < 1.0);
    }

    #[test]
    fn test_low_stability_triggers_earthquakes() {
        let mut world = World::new();
        world.init_resource::<Events<EarthquakeEvent>>();

        // Very low stability
        world.insert_resource(PlanetaryStability { level: 0.1 });

        let mut schedule = Schedule::default();
        schedule.add_systems(trigger_earthquake_system);
        schedule.run(&mut world);

        // Given low stability, an earthquake should trigger
        let events = world.resource::<Events<EarthquakeEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components & Resources

```rust
// src/layer1/tech/core_tapping.rs

use bevy_ecs::prelude::*;
use crate::layer1::energy::PowerGenerator;
use crate::layer1::hazards::EarthquakeEvent;

#[derive(Component, Debug, Clone)]
pub struct CoreDrill {
    pub active: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct PlanetaryStability {
    pub level: f32, // 1.0 down to 0.0
}
```

### 2. Systems

```rust
pub fn process_core_drill_system(
    mut drills: Query<(&CoreDrill, &mut PowerGenerator)>,
    mut stability: ResMut<PlanetaryStability>,
) {
    let mut any_active = false;
    for (drill, mut generator) in drills.iter_mut() {
        if drill.active {
            generator.output = 10000.0; // Massive power
            any_active = true;
        } else {
            generator.output = 0.0;
        }
    }

    // Degrade stability if any drill is active
    if any_active {
        stability.level = (stability.level - 0.001).max(0.0);
    }
}

pub fn trigger_earthquake_system(
    stability: Res<PlanetaryStability>,
    mut events: EventWriter<EarthquakeEvent>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Higher chance the lower the stability
    let chance = (1.0 - stability.level) * 0.05;

    if rng.gen::<f32>() < chance {
        events.send(EarthquakeEvent {
            magnitude: 5.0 + (1.0 - stability.level) * 5.0, // Scales with instability
        });
    }
}
```

## REFACTOR Phase: Quality & Design

- **Catastrophic Failure**: When `PlanetaryStability` hits 0.0, it should trigger a colony wipe or a permanent massive debuff (e.g., magma flooding the map).
- **Toggle Control**: Ensure the player has clear UI to toggle the `CoreDrill` on and off to manage stability decay.
- **Layer 2 Integration**: The massive power output should be exportable to Layer 2 ships or orbital defenses, rather than just powering local Layer 1 buildings.

## Acceptance Criteria

- [ ] `CoreDrill` component drastically increases `PowerGenerator` output when active.
- [ ] Active drills slowly reduce `PlanetaryStability`.
- [ ] Low stability increases the frequency and magnitude of `EarthquakeEvent`s.
- [ ] Tests pass.

## Technical Guidance

- Integrate `process_core_drill_system` before energy consumption calculations in `Layer1SystemSet::Execution`.
- Ensure `EarthquakeEvent` connects to actual physical damage systems defined in `153`.

## Questions

*Builder: Add any questions here.*
