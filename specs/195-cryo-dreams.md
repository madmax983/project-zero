# 195: Cryo-Dreams

## Overview

Cryo-stasis was meant to be a dreamless sleep, but the human mind is stubborn. Even at absolute zero, the subconscious fires.

**Cryo-Dreams** introduces a mechanic where Pops in Cryo-Stasis are not chemically inert. They generate **Knowledge** (Research Points) representing the "processing" of their subconscious experiences. However, this comes with a risk: **Nightmares**. A Pop suffering a Cryo-Nightmare may wake up with permanent psychological scars or severe debuffs.

## Dependencies

- `139` — Cryo-Stasis Vaults
- `084` — Pop Traits
- `029` — Knowledge System (Resource)

## RED Phase: Tests First

Write these tests in `src/layer1/cryo_dreams_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::cryo::CryoStasis;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::traits::{Traits, Trait};
    use crate::layer1::cryo_dreams::{CryoDreamState, cryo_dream_system, CryoTrauma};
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_cryo_pop_generates_knowledge() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn a frozen pop
        world.spawn((
            Pop,
            CryoStasis,
            CryoDreamState::default(),
            Traits::default(),
        ));

        // Run system
        world.run_system_once(cryo_dream_system);

        let res = world.resource::<ColonyResources>();
        assert!(res.knowledge > 0.0, "Frozen pop should generate knowledge");
    }

    #[test]
    fn test_creative_trait_boosts_dreams() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Normal pop
        world.spawn((
            Pop,
            CryoStasis,
            CryoDreamState::default(),
            Traits::default(),
        ));

        // Creative pop
        let mut traits = Traits::default();
        traits.add(Trait::Creative);
        world.spawn((
            Pop,
            CryoStasis,
            CryoDreamState::default(),
            traits,
        ));

        // We need to track individual generation, but the system aggregates to global resource.
        // For testing, we might inspect the accumulator in CryoDreamState if we design it that way.
        // Or we spawn them in separate worlds/runs.
    }

    #[test]
    fn test_nightmare_acquisition() {
        let mut world = World::new();
        // Force a nightmare by setting high risk or mocking RNG (if possible)
        // For this test, we assume a specific function or state can trigger it.
        let pop = world.spawn((
            Pop,
            CryoStasis,
            CryoDreamState::default(),
            Traits::default(), // Maybe add "Anxious" to increase chance
        )).id();

        // Run system enough times or with forced RNG to trigger nightmare
        // (Implementation detail: maybe expose a config resource for test probability)

        // Assert CryoTrauma is added
        // assert!(world.get::<CryoTrauma>(pop).is_some());
    }

    #[test]
    fn test_trauma_affects_waking() {
        // Test that waking a pop with CryoTrauma applies extra penalties
        // This requires modifying exit_cryo_system from Spec 139
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. New Components (`src/layer1/cryo_dreams.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Default)]
pub struct CryoDreamState {
    pub accumulator: f32,
}

#[derive(Component)]
pub struct CryoTrauma {
    pub severity: f32,
}
```

### 2. The Dream System

```rust
pub fn cryo_dream_system(
    mut query: Query<(&mut CryoDreamState, &crate::layer1::traits::Traits), With<crate::layer1::cryo::CryoStasis>>,
    mut resources: ResMut<crate::layer1::resources::ColonyResources>,
) {
    let mut total_knowledge = 0.0;

    for (mut dream, traits) in query.iter_mut() {
        let mut rate = 0.01; // Base rate per tick

        if traits.has(crate::layer1::traits::Trait::Creative) {
            rate *= 1.5;
        }

        dream.accumulator += rate;

        if dream.accumulator >= 1.0 {
            total_knowledge += 1.0;
            dream.accumulator -= 1.0;

            // Roll for Nightmare here (e.g. 1% chance per knowledge point generated)
        }
    }

    if total_knowledge > 0.0 {
        resources.add_knowledge(total_knowledge);
    }
}
```

### 3. Integration with Cryo Stasis (`src/layer1/cryo.rs`)

Modify `enter_cryo_system` to add `CryoDreamState`.
Modify `exit_cryo_system` to check for `CryoTrauma`.

```rust
// in exit_cryo_system
if let Some(trauma) = commands.entity(entity).get::<CryoTrauma>() {
    // Apply permanent trait or massive mood debuff
    // Remove CryoTrauma
}
```

## REFACTOR Phase: Quality & Design

- **Visual Feedback**: Spawn floating text or particles over pods when they generate research.
- **UI**: Show "Dreaming..." status in Inspector.
- **Balancing**: Ensure the knowledge gain isn't an exploit to just freeze everyone. It should be slower than active research.
- **Event Logs**: "A Pop is having a nightmare!"

## Acceptance Criteria

- [ ] Frozen pops generate Knowledge over time.
- [ ] Traits (`Creative`, `Intellectual`) buff generation rate.
- [ ] There is a small chance of "Nightmare" events.
- [ ] Nightmares leave a persistent `CryoTrauma` component or effect upon waking.
- [ ] Test coverage > 85%.

## Technical Guidance

- Use `rand::Rng` for nightmare chance.
- Ensure `CryoDreamState` is added when entering cryo.
- Ensure `CryoDreamState` and `CryoTrauma` are removed when exiting cryo (after applying effects).
