# 371: The Hive Mind Resurgence

## Overview

"A forgotten biological weapon wakes up."

**The Hive Mind Resurgence** introduces a cross-layer biological threat. A dormant hive mind spore, discovered as a Layer 3 anomaly, infects a colony. Infected Pops act normally but secretly spread the spore to others and sabotage defenses. Once a critical mass is reached, they rebel and mutate into a coordinated, horrifying sub-faction.

This creates tension: the infected Pops actually receive an initial efficiency boost (working harder due to hive coordination), making the infection subtly beneficial before the inevitable, devastating rebellion.

## Dependencies

- `068` — Pop Factions (for handling the rebellion sub-faction)
- `084` — Pop Traits (for tracking infection state)

## RED Phase: Tests First

Write these tests in `src/layer1/contagion/hive_mind_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, PopState};
    use crate::layer1::traits::{Traits, Trait};
    use crate::layer1::contagion::hive_mind::{HiveSpore, InfectionStage, process_hive_infection_system, trigger_hive_rebellion_system, HiveMindRebellionEvent};

    #[test]
    fn test_infection_spreads_and_progresses() {
        let mut world = World::new();

        // Infected Pop
        let infected = world.spawn((
            Pop,
            HiveSpore { stage: InfectionStage::Latent, spread_cooldown: 0.0 },
            Traits::default(),
        )).id();

        // Healthy Pop
        let healthy = world.spawn((
            Pop,
            Traits::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_hive_infection_system);
        schedule.run(&mut world);

        // The latent infection should progress towards active/critical over time
        // And potentially spread to the healthy pop if close enough
        let infected_spore = world.get::<HiveSpore>(infected).unwrap();
        assert!(matches!(infected_spore.stage, InfectionStage::Active) || infected_spore.spread_cooldown > 0.0);
    }

    #[test]
    fn test_infected_pops_gain_efficiency_buff() {
        let mut world = World::new();

        let infected = world.spawn((
            Pop,
            HiveSpore { stage: InfectionStage::Active, spread_cooldown: 0.0 },
            Traits::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_hive_infection_system);
        schedule.run(&mut world);

        let traits = world.get::<Traits>(infected).unwrap();
        // The HiveCoordination trait grants the efficiency buff
        assert!(traits.0.contains(&Trait::HiveCoordination));
    }

    #[test]
    fn test_critical_mass_triggers_rebellion() {
        let mut world = World::new();
        world.init_resource::<Events<HiveMindRebellionEvent>>();

        // Spawn many infected Pops at Critical stage
        for _ in 0..10 {
            world.spawn((
                Pop,
                HiveSpore { stage: InfectionStage::Critical, spread_cooldown: 0.0 },
            ));
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(trigger_hive_rebellion_system);
        schedule.run(&mut world);

        let events = world.resource::<Events<HiveMindRebellionEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/contagion/hive_mind.rs

use bevy_ecs::prelude::*;
use crate::layer1::traits::Trait;

#[derive(Debug, Clone, PartialEq)]
pub enum InfectionStage {
    Latent,
    Active,
    Critical,
}

#[derive(Component, Debug, Clone)]
pub struct HiveSpore {
    pub stage: InfectionStage,
    pub spread_cooldown: f32,
}

#[derive(Event, Debug, Clone)]
pub struct HiveMindRebellionEvent;
```

### 2. Systems

```rust
use crate::layer1::pop::Pop;
use crate::layer1::traits::Traits;

pub fn process_hive_infection_system(
    mut query: Query<(Entity, &mut HiveSpore, &mut Traits), With<Pop>>,
) {
    for (entity, mut spore, mut traits) in query.iter_mut() {
        // Decrease cooldown
        spore.spread_cooldown = (spore.spread_cooldown - 1.0).max(0.0);

        // Progression logic (simplified for GREEN)
        if spore.stage == InfectionStage::Latent && spore.spread_cooldown == 0.0 {
            spore.stage = InfectionStage::Active;
            spore.spread_cooldown = 100.0;
        } else if spore.stage == InfectionStage::Active && spore.spread_cooldown == 0.0 {
            spore.stage = InfectionStage::Critical;
            traits.0.insert(Trait::HiveCoordination);
        }
    }
}

pub fn trigger_hive_rebellion_system(
    query: Query<&HiveSpore, With<Pop>>,
    mut events: EventWriter<HiveMindRebellionEvent>,
) {
    let critical_count = query.iter().filter(|s| s.stage == InfectionStage::Critical).count();

    // Threshold for rebellion
    if critical_count >= 10 {
        events.send(HiveMindRebellionEvent);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Proximity Spread**: Spread logic should check spatial proximity using `GridPosition` to infect nearby Pops during the `process_hive_infection_system`.
- **Sabotage Actions**: Critical stage pops should gain utility actions to secretly sabotage defenses (e.g., lower turret health) without alerting the player directly.
- **Rebellion Transformation**: The `HiveMindRebellionEvent` should be caught by a system that forcibly changes the Pop's faction and transforms them into hostile entities.

## Acceptance Criteria

- [ ] `HiveSpore` component exists with stages.
- [ ] Infection progresses over time.
- [ ] Active infection grants a trait (`Trait::HiveCoordination`) for efficiency buffs.
- [ ] Critical mass of infections triggers `HiveMindRebellionEvent`.
- [ ] Tests pass.

## Technical Guidance

- Integrate `process_hive_infection_system` into `Layer1SystemSet::Observation`.
- Add `Trait::HiveCoordination` to the `Trait` enum in `src/layer1/traits.rs`.
- Ensure the `HiveMindRebellionEvent` is registered in the app builder.

## Questions

*Builder: Add any questions here.*
