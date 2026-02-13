# 107: Biocompatibility

## Overview

The planet is alien. Not all colonists can tolerate its atmosphere, pollen, or microbes.
This spec introduces a **Biocompatibility** system:
- Each Pop has a `Biocompatibility` stat (0.0 to 1.0).
- Tiles (based on Atmosphere/Flora) have a "Bio-Hazard" level.
- If Bio-Hazard > Biocompatibility, the Pop takes damage or works slower.

This adds survival pressure: some colonists are naturally suited for outdoor work, while others must stay in sealed habitats or wear suits (future).

## Dependencies

- `041` — Atmosphere Simulation (for pollution/hazard grid)
- `034` — Pop Health (for taking damage)
- `084` — Pop Traits (for trait-based modifiers)

## RED Phase: Tests First

Write these tests in `src/layer1/biocompatibility_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Biocompatibility};
    use crate::layer1::health::Health;
    use crate::layer1::atmosphere::AtmosphereGrid;
    use crate::layer1::map::GridPosition;
    use crate::layer1::biocompatibility::biocompatibility_system;

    #[test]
    fn test_biocompatibility_component_default() {
        let bio = Biocompatibility::default();
        // Default should be average, e.g., 0.5
        assert_eq!(bio.value, 0.5);
    }

    #[test]
    fn test_high_biocompatibility_resists_hazard() {
        let mut world = World::new();
        // Setup Hazard Grid (Atmosphere)
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(0, 0, 0.4); // Moderate hazard
        world.insert_resource(grid);

        // Spawn Pop with High Bio (0.8) > Hazard (0.4)
        let pop = world.spawn((
            Pop,
            Biocompatibility { value: 0.8 },
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run system
        biocompatibility_system(&mut world);

        // Assert: No damage
        let health = world.get::<Health>(pop).unwrap();
        assert_eq!(health.current, 100.0);
    }

    #[test]
    fn test_low_biocompatibility_takes_damage() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(0, 0, 0.6); // High hazard
        world.insert_resource(grid);

        // Spawn Pop with Low Bio (0.2) < Hazard (0.6)
        let pop = world.spawn((
            Pop,
            Biocompatibility { value: 0.2 },
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        biocompatibility_system(&mut world);

        // Assert: Damage taken
        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_traits_modify_biocompatibility() {
        // Verify NativeBorn boosts resistance
        let mut world = World::new();
        use crate::layer1::traits::{Trait, Traits};
        use std::collections::HashSet;

        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(0, 0, 0.7); // High hazard
        world.insert_resource(grid);

        // Pop with Base 0.5 + NativeBorn (+0.3) = 0.8 > 0.7
        let pop = world.spawn((
            Pop,
            Biocompatibility { value: 0.5 },
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
            Traits(HashSet::from([Trait::NativeBorn])),
        )).id();

        biocompatibility_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert_eq!(health.current, 100.0, "NativeBorn should resist hazard");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `Trait` Enum

Modify `src/layer1/traits.rs` to include:
- `NativeBorn`
- `WeakImmunity`

Update the `random` generation logic to include these new traits in the pool.

### 2. Define Component

In `src/layer1/pop.rs` (or new module `src/layer1/biocompatibility.rs`):

```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct Biocompatibility {
    pub value: f32, // 0.0 to 1.0
}

impl Default for Biocompatibility {
    fn default() -> Self {
        Self { value: 0.5 }
    }
}
```

### 3. Implement System

In `src/layer1/biocompatibility.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::atmosphere::AtmosphereGrid;
use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Biocompatibility;
use crate::layer1::traits::{Trait, Traits};

pub fn biocompatibility_system(world: &mut World) {
    let mut damages = Vec::new();

    // 1. Read Phase
    {
        let grid = world.resource::<AtmosphereGrid>();
        // Query for pops with potential hazard exposure
        let mut query = world.query::<(Entity, &GridPosition, &Biocompatibility, Option<&Traits>)>();

        for (entity, pos, bio, traits) in query.iter(world) {
            let hazard_level = grid.get(pos.x as i32, pos.y as i32);

            // Calculate effective bio
            let mut effective_bio = bio.value;
            if let Some(t) = traits {
                if t.0.contains(&Trait::NativeBorn) {
                    effective_bio += 0.3;
                }
                if t.0.contains(&Trait::WeakImmunity) {
                    effective_bio -= 0.2;
                }
            }

            // Check threshold
            // Tolerance buffer: damage only if hazard significantly exceeds bio
            if hazard_level > effective_bio {
                let delta = hazard_level - effective_bio;
                let damage = delta * 5.0; // Scaling factor
                damages.push((entity, damage));
            }
        }
    }

    // 2. Write Phase
    for (entity, damage) in damages {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.take_damage(damage);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **UI**: Display "Bio-Comp" in Pop Inspector.
- **Feedback**: Show "Coughing" particle/icon when taking bio-damage.
- **Work Speed**: Also apply a speed penalty (`WorkSpeedModifier`) based on `(hazard - bio).max(0.0)`.
- **Optimization**: Don't check every pop every tick if performance lags; check on a slower schedule (e.g. every 10 ticks).

## Acceptance Criteria

- [ ] `Biocompatibility` component exists.
- [ ] `Trait` enum includes `NativeBorn` and `WeakImmunity`.
- [ ] System compares `AtmosphereGrid` vs `Biocompatibility`.
- [ ] Damage is applied if Hazard > Bio.
- [ ] Traits influence the effective Bio value.
- [ ] Tests pass.

## Technical Guidance

- Ensure `biocompatibility_system` runs *after* `update_atmosphere_system`.
- Reuse `AtmosphereGrid` for "Hazard". Later, we can add a separate `PollenGrid` if needed.
