# 062: Pop Lifecycle and Aging

## Overview

Pops are not immortal. They are born, grow, age, and eventually die. This specification introduces a lifecycle system where Pops progress through life stages (Child, Adult, Elder) based on their age in ticks. As they age, their attributes change, and eventually, they face the risk of natural death.

This adds a critical pressure to the colony: the workforce is not permanent. New generations must be raised (or printed) to replace the old.

## Dependencies

- `004` — Pop Entity (Base entity)
- `027` — Seasonal Rhythms (Time/Ticks per Year)
- `034` — Pop Health (Death mechanics)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/aging_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Speed};
    use crate::layer1::health::Health;
    use crate::shared::time::SimulationTime;
    use crate::layer1::balance::TICKS_PER_YEAR;

    // Define constants for tests if not yet in balance.rs
    const AGE_ADULT: u64 = 18 * TICKS_PER_YEAR;
    const AGE_ELDER: u64 = 60 * TICKS_PER_YEAR;

    #[test]
    fn test_pop_has_age_component() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            crate::layer1::lifecycle::Age::default(),
        )).id();

        let age = world.get::<crate::layer1::lifecycle::Age>(entity);
        assert!(age.is_some());
        assert_eq!(age.unwrap().ticks_alive, 0);
    }

    #[test]
    fn test_aging_system_increments_age() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let entity = world.spawn((
            Pop,
            crate::layer1::lifecycle::Age { ticks_alive: 100, ..Default::default() },
        )).id();

        // Run system
        crate::layer1::lifecycle::aging_system(&mut world);

        let age = world.get::<crate::layer1::lifecycle::Age>(entity).unwrap();
        assert_eq!(age.ticks_alive, 101);
    }

    #[test]
    fn test_lifestage_transitions() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        // 1. Child -> Adult
        let child = world.spawn((
            Pop,
            crate::layer1::lifecycle::Age {
                ticks_alive: AGE_ADULT - 1,
                stage: crate::layer1::lifecycle::LifeStage::Child
            },
            Speed::default(),
        )).id();

        // 2. Adult -> Elder
        let adult = world.spawn((
            Pop,
            crate::layer1::lifecycle::Age {
                ticks_alive: AGE_ELDER - 1,
                stage: crate::layer1::lifecycle::LifeStage::Adult
            },
            Speed::default(),
        )).id();

        crate::layer1::lifecycle::aging_system(&mut world);

        let child_age = world.get::<crate::layer1::lifecycle::Age>(child).unwrap();
        assert_eq!(child_age.stage, crate::layer1::lifecycle::LifeStage::Adult);

        let adult_age = world.get::<crate::layer1::lifecycle::Age>(adult).unwrap();
        assert_eq!(adult_age.stage, crate::layer1::lifecycle::LifeStage::Elder);
    }

    #[test]
    fn test_elder_speed_penalty() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        // Spawn a pop about to become Elder
        let entity = world.spawn((
            Pop,
            crate::layer1::lifecycle::Age {
                ticks_alive: AGE_ELDER - 1,
                stage: crate::layer1::lifecycle::LifeStage::Adult
            },
            Speed { base: 1.0, current: 1.0, accumulator: 0.0 },
        )).id();

        crate::layer1::lifecycle::aging_system(&mut world);

        let speed = world.get::<Speed>(entity).unwrap();
        // Expect penalty (e.g. 0.8x)
        assert!(speed.base < 1.0);
        assert!((speed.base - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_natural_death_chance() {
        // Probabilistic test: excessive age should eventually kill
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        // Add random source resource if needed, or rely on thread_rng in system

        let entity = world.spawn((
            Pop,
            crate::layer1::lifecycle::Age {
                ticks_alive: 120 * TICKS_PER_YEAR, // Very old
                stage: crate::layer1::lifecycle::LifeStage::Elder
            },
            Health { current: 10.0, max: 10.0 },
        )).id();

        // Run many times to trigger probability.
        // With 0.00001 base chance * (120-60) = 0.0006 per tick.
        // 10,000 ticks gives ~99.7% chance of death.
        let mut died = false;
        for _ in 0..10_000 {
            crate::layer1::lifecycle::natural_death_system(&mut world);
            let health = world.get::<Health>(entity).unwrap();
            if !health.is_alive() {
                died = true;
                break;
            }
        }

        assert!(died, "Very old pop should eventually die naturally");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `Age` Component and `LifeStage` Enum

```rust
// src/layer1/lifecycle.rs
use bevy_ecs::prelude::*;
use crate::layer1::balance::TICKS_PER_YEAR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LifeStage {
    #[default]
    Child,
    Adult,
    Elder,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Age {
    pub ticks_alive: u64,
    pub stage: LifeStage,
}

impl Age {
    pub fn new(years: u64) -> Self {
        let ticks = years * TICKS_PER_YEAR;
        let stage = if years < 18 {
            LifeStage::Child
        } else if years < 60 {
            LifeStage::Adult
        } else {
            LifeStage::Elder
        };
        Self { ticks_alive: ticks, stage }
    }
}
```

### 2. `aging_system`

```rust
// src/layer1/lifecycle.rs
use crate::layer1::pop::Speed;
use crate::shared::log::MessageLog;

pub fn aging_system(
    mut query: Query<(Entity, &mut Age, &mut Speed)>,
    mut log: Option<ResMut<MessageLog>>,
) {
    const AGE_ADULT: u64 = 18 * TICKS_PER_YEAR;
    const AGE_ELDER: u64 = 60 * TICKS_PER_YEAR;

    for (entity, mut age, mut speed) in query.iter_mut() {
        age.ticks_alive += 1;

        let new_stage = if age.ticks_alive >= AGE_ELDER {
            LifeStage::Elder
        } else if age.ticks_alive >= AGE_ADULT {
            LifeStage::Adult
        } else {
            LifeStage::Child
        };

        if new_stage != age.stage {
            // Apply transition effects
            if new_stage == LifeStage::Elder {
                speed.base *= 0.8; // 20% slow down
                if let Some(l) = log.as_mut() {
                    l.add("A colonist has become an Elder.");
                }
            }
            age.stage = new_stage;
        }
    }
}
```

### 3. `natural_death_system`

```rust
// src/layer1/lifecycle.rs
use crate::layer1::health::Health;
use rand::Rng;

pub fn natural_death_system(mut query: Query<(&Age, &mut Health)>) {
    let mut rng = rand::thread_rng();

    for (age, mut health) in query.iter_mut() {
        if age.stage == LifeStage::Elder {
            // Base chance increases with age
            // Example: 0.1% chance per tick at 100 years? Too high.
            // 1000 ticks/year.
            // At 80 years = 80,000 ticks.
            // Let's say max age is roughly 100.
            // Probability P = (age - 60) * factor?

            let years = age.ticks_alive as f64 / TICKS_PER_YEAR as f64;
            if years > 60.0 {
                // Exponential curve?
                // Or simple threshold check.
                // Let's say 1/1000 chance per year -> 1/1,000,000 per tick?
                // Maybe check once per day/season to save perf?
                // For MVP, check every tick but with very low prob.

                // Let's say at 80 years, 10% chance to die that year.
                // 10% / 1000 ticks = 0.0001 per tick.

                let chance = (years - 60.0) * 0.00001;
                if rng.gen_bool(chance.max(0.0)) {
                    health.current = 0.0; // Die
                }
            }
        }
    }
}
```

### 4. Update `spawn_initial_pops`

In `src/layer1/pop.rs`, initialize `Age` with a random range (e.g., 20-40 years) so they are Adults.

## REFACTOR Phase: Quality & Design

- **Performance**: Run `aging_system` and `natural_death_system` less frequently (e.g., once per second or strictly on season change) rather than every tick.
- **Config**: Move `AGE_ADULT`, `AGE_ELDER` thresholds to `balance.rs`.
- **Events**: Fire `LifeStageChangedEvent` for UI notifications or other systems (e.g., changing sprite).
- **Children**: Implement growth scaling (Speed < 1.0 for very young children?).
- **Skills**: Elders might gain a `Wisdom` trait or skill boost.

## Acceptance Criteria

- [ ] `Age` component exists and tracks ticks.
- [ ] Pops transition Child -> Adult -> Elder correctly.
- [ ] Elders suffer movement speed penalty.
- [ ] Very old Pops die naturally.
- [ ] Initial Pops spawn with valid Adult ages (not 0).
- [ ] Tests pass.

## Technical Guidance

- **Performance Optimization**: `aging_system` iterates all pops every tick. This is acceptable for MVP (<100 pops). If pop count grows, consider running it every 60 ticks (1s) and incrementing `ticks_alive` by 60.
- **Randomness**: Use `rand::thread_rng()` for probability. For deterministic tests, inject an RNG source or mock it.
- **Integration**: Ensure `death_system` (Spec 034) runs *after* `natural_death_system` in the schedule to clean up dead entities immediately.
- **Time Scale**: Stick to `TICKS_PER_YEAR = 1000` until global balance changes.

## Questions

*Builder: How fast should a "year" be?*
Answer: `TICKS_PER_YEAR` is currently 1000. This is very fast (minutes). Adjust `balance.rs` if a longer game loop is desired, but for now, adhere to the 1000 constant.
