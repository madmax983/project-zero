# 078: The Old Guard

## Overview

The struggle between the hardened founders and the soft new arrivals.

This feature tracks when Pops arrive at the colony. Early arrivals become "Founders" with special status and morale bonuses. Later arrivals are "Immigrants" or "New Blood".
As the population grows, friction develops:
- Founders feel overwhelmed if outnumbered by Immigrants.
- Immigrants feel excluded if Founders form a dominant clique.

This adds social tension and narrative depth to population growth.

## Dependencies

- `010` — Chronicle System (provides `TICKS_PER_YEAR`)
- `031` — Pop Morale (provides `MoodModifier`)
- `004` — Pop Entity (provides `Pop`)

## RED Phase: Tests First

Write these tests in `src/layer1/social/old_guard_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::old_guard::{
        Arrival, Generation, FounderBuff, check_generational_friction_system,
        apply_founder_benefits_system, FOUNDER_CUTOFF_YEAR
    };
    use crate::layer1::chronicle::TICKS_PER_YEAR;
    use crate::layer1::morale::MoodModifier;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_arrival_component_defaults() {
        let arrival = Arrival { tick: 100 };
        assert_eq!(arrival.tick, 100);
    }

    #[test]
    fn test_generation_determination() {
        // Founders arrive before Year 5 (5000 ticks)
        let early = Arrival { tick: 100 };
        assert_eq!(early.generation(), Generation::Founder);

        let late = Arrival { tick: 6000 };
        assert_eq!(late.generation(), Generation::Immigrant);
    }

    #[test]
    fn test_apply_founder_benefits() {
        let mut world = World::new();

        // Spawn a Founder
        let founder = world.spawn((
            Pop,
            Arrival { tick: 100 }, // Year 0
            // Assuming Morale/Mood component exists, but system adds Modifier component
        )).id();

        // Spawn an Immigrant
        let immigrant = world.spawn((
            Pop,
            Arrival { tick: 6000 }, // Year 6
        )).id();

        // Run system
        world.run_system_once(apply_founder_benefits_system);

        // Check for Buff component or MoodModifier
        // Spec 031 usually uses a list of modifiers.
        // For MVP, let's assume we add a specific `FounderBuff` component that provides the mood boost.
        assert!(world.get::<FounderBuff>(founder).is_some());
        assert!(world.get::<FounderBuff>(immigrant).is_none());
    }

    #[test]
    fn test_friction_founders_overwhelmed() {
        let mut world = World::new();

        // 1 Founder
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));

        // 3 Immigrants (Ratio 3:1 > 2:1)
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));

        // Need a resource to track colony state or system iterates all?
        // System iterates all pops.

        world.run_system_once(check_generational_friction_system);

        // Founder should have "Overwhelmed" mood modifier
        // Verify via MoodModifier query
        let modifiers = world.query::<&MoodModifier>().iter(&world);
        let overwhelmed = modifiers.filter(|m| m.source == "Overwhelmed by Strangers").count();
        assert_eq!(overwhelmed, 1);
    }

    #[test]
    fn test_friction_immigrants_excluded() {
        let mut world = World::new();

        // 3 Founders
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));
        world.spawn((Pop, Arrival { tick: 100 }, Generation::Founder));

        // 1 Immigrant
        world.spawn((Pop, Arrival { tick: 6000 }, Generation::Immigrant));

        world.run_system_once(check_generational_friction_system);

        // Immigrant should have "Excluded" mood modifier
        let modifiers = world.query::<&MoodModifier>().iter(&world);
        let excluded = modifiers.filter(|m| m.source == "Excluded by Clique").count();
        assert_eq!(excluded, 1);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components and Enums

Create `src/layer1/social/old_guard.rs`.

```rust
use bevy_ecs::prelude::*;
use crate::layer1::chronicle::TICKS_PER_YEAR;
use crate::layer1::morale::MoodModifier;
use crate::layer1::pop::Pop;

pub const FOUNDER_CUTOFF_YEAR: u64 = 5;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Component)]
pub enum Generation {
    Founder,
    Immigrant,
}

#[derive(Component)]
pub struct Arrival {
    pub tick: u64,
}

impl Arrival {
    pub fn generation(&self) -> Generation {
        if self.tick < FOUNDER_CUTOFF_YEAR * TICKS_PER_YEAR {
            Generation::Founder
        } else {
            Generation::Immigrant
        }
    }
}

/// Marker component for Founders receiving the buff.
#[derive(Component)]
pub struct FounderBuff;

```

### 2. Implement `apply_founder_benefits_system`

```rust
pub fn apply_founder_benefits_system(
    mut commands: Commands,
    query: Query<(Entity, &Arrival), (With<Pop>, Without<FounderBuff>, Without<Generation>)>,
) {
    for (entity, arrival) in query.iter() {
        let gen = arrival.generation();
        commands.entity(entity).insert(gen);

        if gen == Generation::Founder {
            commands.entity(entity).insert((
                FounderBuff,
                MoodModifier {
                    value: 5.0,
                    source: "Legacy of the First".to_string(),
                    duration: f32::MAX, // Permanent
                },
            ));
        }
    }
}
```

### 3. Implement `check_generational_friction_system`

```rust
pub fn check_generational_friction_system(
    mut commands: Commands,
    pop_query: Query<(Entity, &Generation), With<Pop>>,
) {
    let mut founders = 0;
    let mut immigrants = 0;

    for (_, gen) in pop_query.iter() {
        match gen {
            Generation::Founder => founders += 1,
            Generation::Immigrant => immigrants += 1,
        }
    }

    if founders == 0 && immigrants == 0 {
        return;
    }

    // Determine global state
    let overwhelmed = founders > 0 && immigrants > founders * 2;
    let excluded = immigrants > 0 && founders > immigrants;

    // Apply modifiers
    // Note: We need to avoid adding duplicate modifiers every tick.
    // Ideally, we use a specific Component `SocialFriction` to track this state
    // or update an existing MoodModifier.
    // For MVP, we'll assume a system that cleans up old modifiers or we overwrite specific ones.
    // Here we just insert/remove based on state.

    for (entity, gen) in pop_query.iter() {
        match gen {
            Generation::Founder => {
                if overwhelmed {
                    // Add/Refresh negative modifier
                    commands.entity(entity).insert(MoodModifier {
                        value: -5.0,
                        source: "Overwhelmed by Strangers".to_string(),
                        duration: 10.0, // Short duration, refreshed constantly
                    });
                }
            }
            Generation::Immigrant => {
                if excluded {
                    commands.entity(entity).insert(MoodModifier {
                        value: -2.0,
                        source: "Excluded by Clique".to_string(),
                        duration: 10.0,
                    });
                }
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Don't iterate all pops to count them every tick. Cache the counts in a Resource `Demographics`.
- **Modifier Stacking**: Ensure `MoodModifier` doesn't stack infinitely. Use a unique ID or `Component` for the friction state (e.g., `Status: Overwhelmed`).
- **Integration**: Show "Founder" status in the Pop Inspector UI.
- **Narrative**: Trigger a `ChronicleEvent` when the population flips from Majority Founder to Majority Immigrant ("The Turning Point").

## Acceptance Criteria

- [ ] `Arrival` component tracks tick.
- [ ] `Generation` correctly identifies Founder (< Year 5) vs Immigrant.
- [ ] Founders get +5 permanent Morale buff.
- [ ] Founders get -5 Morale if Immigrants > 2x Founders.
- [ ] Immigrants get -2 Morale if Founders > Immigrants.
- [ ] Tests pass.

## Technical Guidance

- In `layer1/mod.rs`, register the systems.
- Add `Arrival` component to `spawn_pop` function in `src/layer1/pop.rs`. Set `tick` to `simulation_time.tick`.
