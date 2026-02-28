# 252: Tectonic Stress

## Overview

"The ground remembers every bomb you dropped."

The planet has a global **Stress Meter**. Industrial activities (Mining, Explosions, Heavy Manufacturing) increase it. It dissipates slowly over time.
If Stress reaches 100%, a **Mega-Quake** occurs, causing massive damage across the map.
Players can perform controlled "Relief Quakes" (Action) to lower the meter safely, but this causes minor local damage.

- **Global Resource**: `TectonicStress` (0.0 - 100.0).
- **Triggers**: `MiningAction`, `ExplosionEvent`, `HeavyIndustry`.
- **Consequence**: `MegaQuakeEvent` (Map-wide destruction).

## Dependencies

- `018` — Mining Resources (Source of stress)
- `153` — Geological Instability (Quake mechanics)
- `223` — Volatile Resources (Explosions add stress)

## RED Phase: Tests First

Write these tests in `src/layer1/geology/tectonic_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::geology::tectonic::{TectonicStress, update_stress_system, check_quake_system, MegaQuakeEvent};
    use crate::layer1::mining::MiningEvent;
    use crate::layer1::volatile::ExplosionEvent; // Assuming exists

    #[test]
    fn test_mining_increases_stress() {
        let mut world = World::new();
        world.insert_resource(TectonicStress::default());
        world.insert_resource(Events::<MiningEvent>::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(update_stress_system);

        // Send mining event
        world.send_event(MiningEvent { amount: 10 }); // Abstract payload

        schedule.run(&mut world);

        let stress = world.resource::<TectonicStress>();
        assert!(stress.current > 0.0);
    }

    #[test]
    fn test_stress_dissipation() {
        let mut world = World::new();
        world.insert_resource(TectonicStress { current: 50.0, dissipation_rate: 1.0, ..Default::default() });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_stress_system);

        schedule.run(&mut world);

        let stress = world.resource::<TectonicStress>();
        assert_eq!(stress.current, 49.0);
    }

    #[test]
    fn test_mega_quake_trigger() {
        let mut world = World::new();
        world.insert_resource(TectonicStress { current: 100.0, threshold: 100.0, ..Default::default() });
        world.init_resource::<Events<MegaQuakeEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_quake_system);

        schedule.run(&mut world);

        let events = world.resource::<Events<MegaQuakeEvent>>();
        assert!(!events.is_empty());

        let stress = world.resource::<TectonicStress>();
        assert_eq!(stress.current, 0.0); // Reset after quake
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Resources & Events

```rust
// src/layer1/geology/tectonic.rs

use bevy_ecs::prelude::*;
use crate::layer1::mining::MiningEvent;
use crate::layer1::volatile::ExplosionEvent;

#[derive(Resource, Debug)]
pub struct TectonicStress {
    pub current: f32,
    pub threshold: f32,
    pub dissipation_rate: f32,
}

impl Default for TectonicStress {
    fn default() -> Self {
        Self {
            current: 0.0,
            threshold: 100.0,
            dissipation_rate: 0.1,
        }
    }
}

#[derive(Event)]
pub struct MegaQuakeEvent;

pub fn update_stress_system(
    mut stress: ResMut<TectonicStress>,
    mut mining: EventReader<MiningEvent>,
    mut explosions: EventReader<ExplosionEvent>,
) {
    // 1. Add Stress
    for _ in mining.read() {
        stress.current += 0.5; // Tuning value
    }
    for event in explosions.read() {
        stress.current += event.damage * 0.1;
    }

    // 2. Dissipate
    stress.current = (stress.current - stress.dissipation_rate).max(0.0);
}

pub fn check_quake_system(
    mut stress: ResMut<TectonicStress>,
    mut quake_writer: EventWriter<MegaQuakeEvent>,
) {
    if stress.current >= stress.threshold {
        quake_writer.send(MegaQuakeEvent);
        stress.current = 0.0; // Reset
        // Optional: reduce threshold?
    }
}
```

## REFACTOR Phase: Quality & Design

- **UI**: Add a `Seismograph` widget. Green -> Yellow -> Red -> Flashing.
- **Relief Valve**: Implement the "Relief Quake" action (Spec 021 ActionType). It should trigger a smaller `LocalQuakeEvent` and reduce `stress.current` by 20.
- **Feedback Loop**: MegaQuake should damage `Structure` entities. Hook into `handle_quake_damage` (from Spec 153).

## Acceptance Criteria

- [ ] `TectonicStress` resource tracks value.
- [ ] Mining/Explosions increase stress.
- [ ] Stress dissipates over time.
- [ ] `MegaQuakeEvent` fires at 100%.
- [ ] Tests pass.

## Technical Guidance

- Use `MiningEvent` if available from `018`. If not, hook into `finish_mining_action`.
- Register `MegaQuakeEvent` in `simulation.rs`.

## Questions

*Builder: Does digging a hole reduce stress?*
*Architect: Normal mining increases stress slightly. You need specialized "Relief Drilling" designations to lower it.*
*Architect: No, digging *causes* stress. Only "Relief Quakes" (controlled explosions) or Time reduce it.*
