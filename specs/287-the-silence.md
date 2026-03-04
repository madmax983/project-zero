# 287: The Silence

## 1. Overview
The galaxy is quiet for a reason. (The Dark Forest theory).

**Layer:** 3

**Fantasy:** The more noise you make, the more likely something terrifying finds you.

**Mechanic:** High emission of radio/energy (Tech level + Pop count) increases "Detection Risk". At certain thresholds, unknown hostile entities spawn at the galaxy edge.

**Emergence:** Advanced civilizations try to stay "quiet" or primitive to avoid detection. A loud neighbor endangers the whole sector.

**Tension:** Progress (power/tech) vs. Safety (obscurity).

## 2. Dependencies
- **042 Energy System:** Need the concept of energy generation/consumption.
- **029 Knowledge System:** Tech levels contribute to noise.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::power::PowerOutput;
    use crate::layer1::pop::Pop;
    use crate::layer3::silence::{DetectionRisk, HostileSpawnEvent, update_detection_risk_system, check_hostile_spawn_system};

    #[test]
    fn test_detection_risk_increases_with_power_and_pops() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk { current_risk: 0.0, threshold: 100.0 });

        // Spawn 10 Pops
        for _ in 0..10 {
            world.spawn(Pop);
        }
        // Spawn Power Generators
        world.spawn(PowerOutput { current: 50.0, max: 50.0 });
        world.spawn(PowerOutput { current: 30.0, max: 50.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_detection_risk_system);
        schedule.run(&mut world);

        let risk = world.resource::<DetectionRisk>();
        // e.g. risk = (10 pops * 0.1) + (80 power * 0.05) = 1.0 + 4.0 = 5.0
        assert_eq!(risk.current_risk, 5.0);
    }

    #[test]
    fn test_hostile_spawn_triggered_when_threshold_exceeded() {
        let mut world = World::new();
        world.insert_resource(DetectionRisk { current_risk: 105.0, threshold: 100.0 });
        world.insert_resource(Events::<HostileSpawnEvent>::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(check_hostile_spawn_system);
        schedule.run(&mut world);

        let events = world.resource::<Events<HostileSpawnEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some(), "HostileSpawnEvent should have been emitted.");

        // Ensure risk resets or threshold increases
        let risk = world.resource::<DetectionRisk>();
        assert!(risk.threshold > 100.0, "Threshold should increase after a spawn.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;
use crate::layer1::power::PowerOutput;
use crate::layer1::pop::Pop;

#[derive(Resource)]
pub struct DetectionRisk {
    pub current_risk: f32,
    pub threshold: f32,
}

impl Default for DetectionRisk {
    fn default() -> Self {
        Self { current_risk: 0.0, threshold: 100.0 }
    }
}

#[derive(Event)]
pub struct HostileSpawnEvent {
    pub severity: u32,
}

pub fn update_detection_risk_system(
    mut risk: ResMut<DetectionRisk>,
    pops: Query<(), With<Pop>>,
    power_sources: Query<&PowerOutput>,
) {
    let pop_count = pops.iter().count() as f32;
    let total_power: f32 = power_sources.iter().map(|p| p.current).sum();

    // Formula: 0.1 per pop, 0.05 per power unit.
    risk.current_risk = (pop_count * 0.1) + (total_power * 0.05);
}

pub fn check_hostile_spawn_system(
    mut risk: ResMut<DetectionRisk>,
    mut spawn_events: EventWriter<HostileSpawnEvent>,
) {
    if risk.current_risk >= risk.threshold {
        spawn_events.send(HostileSpawnEvent {
            severity: (risk.current_risk / 100.0) as u32,
        });

        // Increase threshold for next wave to create escalating tension
        risk.threshold *= 1.5;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: `HostileSpawnEvent` needs to be consumed by Layer 2/3 systems to physically spawn enemy fleets at the edge of the map.
- **Tech Tree Modification**: Certain technologies should inherently raise `current_risk` drastically (e.g. Subspace Radio, Antimatter Drives) and must be checked in the logic.
- **UI Element**: Add a menacing "Noise Level" or "Detection Risk" bar in the UI that pulses red when near the threshold.

## 6. Acceptance Criteria
- [ ] Tests pass in `src/layer3/silence_tests.rs`.
- [ ] `DetectionRisk` resource correctly tallies power and pop counts.
- [ ] Exceeding threshold emits `HostileSpawnEvent`.
- [ ] Code is formatted and linted properly.

## 7. Technical Guidance
- Ensure the event logic does not spawn enemies every single tick while over the threshold; the threshold bump logic should prevent this, but it must be robust.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
*Architect:* MVP is just the triggering mechanism. The actual combat and entity spawning for Layer 3 are handled by separate systems listening to `HostileSpawnEvent`.
