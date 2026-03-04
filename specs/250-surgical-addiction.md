# 250: Surgical Addiction

## Overview

"The flesh is weak. But the chrome is hungry."

Pops with the `Transhumanist` trait or those who have undergone extensive cybernetic augmentation (Spec 151) can develop **Surgical Addiction**. They gain a new Need: `Augmentation`. This need decays over time. If unsatisfied (by installing new cybernetics or "maintenance" surgery), they suffer severe Mood penalties.

In extreme cases (Withdrawal), addicted Pops may perform **Self-Surgery**, consuming Raw Metal or Scrap to "upgrade" themselves, resulting in severe injuries (`Bleeding`, `Infected`) but temporarily satisfying the addiction.

## Dependencies

- `151` — Cybernetic Augmentation (Prosthetics, Surgery)
- `084` — Pop Traits (Transhumanist)
- `034` — Pop Health (Injuries)
- `031` — Pop Morale (Mood)

## RED Phase: Tests First

Write these tests in `src/layer1/addiction_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Mood, Trait};
    use crate::layer1::cybernetics::{Augmentations, ProstheticType};
    use crate::layer1::addiction::{SurgicalAddiction, update_addiction_system, check_self_surgery_system};
    use crate::layer1::health::{Health, InjuryType};
    use crate::layer1::inventory::Inventory;
    use crate::layer1::items::ItemType;

    #[test]
    fn test_addiction_starts_after_surgery_if_transhumanist() {
        let mut world = World::new();
        // Hook for surgery completion event? Or just check existing augs.
        // Let's assume addiction is added when Augmentations count > threshold AND trait exists.

        let pop = world.spawn((
            Pop,
            Augmentations { installed: vec![Entity::from_raw(1)] }, // 1 aug
            Trait::Transhumanist,
            // No addiction yet
        )).id();

        // Run system that initializes addiction
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::addiction::init_addiction_system);
        schedule.run(&mut world);

        assert!(world.get::<SurgicalAddiction>(pop).is_some());
    }

    #[test]
    fn test_addiction_decay_lowers_mood() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Mood { value: 100.0, ..Default::default() },
            SurgicalAddiction { craving: 100.0, decay_rate: 1.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_addiction_system);
        schedule.run(&mut world);

        let addiction = world.get::<SurgicalAddiction>(pop).unwrap();
        assert_eq!(addiction.craving, 99.0);

        // Run enough times to trigger withdrawal mood penalty
        for _ in 0..100 {
            schedule.run(&mut world);
        }

        let mood = world.get::<Mood>(pop).unwrap();
        assert!(mood.value < 100.0);
    }

    #[test]
    fn test_self_surgery_triggers_at_zero_craving() {
        let mut world = World::new();

        // Spawn pop with 0 craving and scrap metal in inventory
        let pop = world.spawn((
            Pop,
            SurgicalAddiction { craving: 0.0, decay_rate: 1.0 },
            Health { current: 100.0, max: 100.0 },
            Inventory { items: vec![ItemType::ScrapMetal] }, // Abstracted
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_self_surgery_system);
        schedule.run(&mut world);

        // Check consequences
        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0); // Took damage

        let addiction = world.get::<SurgicalAddiction>(pop).unwrap();
        assert!(addiction.craving > 50.0); // Satisfied temporarily

        // Inventory check (simplified)
        // assert_inventory_empty(pop);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/addiction.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, Mood, Trait};
use crate::layer1::cybernetics::Augmentations;
use crate::layer1::health::Health;

#[derive(Component, Debug, Default)]
pub struct SurgicalAddiction {
    pub craving: f32, // 0.0 (Withdrawal) to 100.0 (Satisfied)
    pub decay_rate: f32,
}

pub fn init_addiction_system(
    mut commands: Commands,
    query: Query<(Entity, &Augmentations, &Trait), Without<SurgicalAddiction>>,
) {
    for (entity, augs, trait_) in query.iter() {
        if *trait_ == Trait::Transhumanist && !augs.installed.is_empty() {
            commands.entity(entity).insert(SurgicalAddiction {
                craving: 100.0,
                decay_rate: 0.1,
            });
        }
    }
}

pub fn update_addiction_system(
    mut query: Query<(&mut SurgicalAddiction, &mut Mood)>,
) {
    for (mut addiction, mut mood) in query.iter_mut() {
        addiction.craving = (addiction.craving - addiction.decay_rate).max(0.0);

        if addiction.craving < 20.0 {
            // Withdrawal penalty
            mood.value -= 0.5; // Heavy decay per tick
        }
    }
}

pub fn check_self_surgery_system(
    mut query: Query<(&mut SurgicalAddiction, &mut Health, &crate::layer1::inventory::Inventory)>,
) {
    for (mut addiction, mut health, inventory) in query.iter_mut() {
        if addiction.craving <= 0.0 {
            // Check for material (Mock check)
            // if inventory.contains(ScrapMetal) ...

            // Perform Self-Surgery
            health.current -= 30.0; // Major damage
            addiction.craving = 50.0; // Partial relief

            // Consume item logic here
            // Add "Self-Mutilated" trait or memory
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Addiction Source**: Non-transhumanists should also get addicted if they have TOO MANY augs (e.g. > 3).
- **Utility AI**: Add `SelfSurgery` as a desperate ActionType in Utility AI, rather than a forced system check. This allows the AI to weigh "Die of Infection" vs "Die of Mood Failure".
- **Medical Treatment**: Doctors can perform "Maintenance Surgery" (using just time/medicine, no new augs) to reset Craving without adding new hardware.

## Acceptance Criteria

- [ ] `SurgicalAddiction` component exists.
- [ ] Transhumanists with augs develop addiction.
- [ ] Addiction decays over time.
- [ ] Low addiction (Withdrawal) reduces Mood.
- [ ] Self-Surgery happens at critical withdrawal (Action or System), causing damage.
- [ ] Tests pass.

## Technical Guidance

- Use `Spec 151` for Augmentation checks.
- Self-Surgery should likely be an `ActionType` in `utility_ai_system` to play nice with the simulation loop, but for GREEN phase, a direct system is acceptable if simpler.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
