# Spec 630: Biocompatibility

## 1. Overview
The planet's biology actively rejects the colonists. Pops have a "Biocompatibility" rating with the local flora and atmosphere. A low rating causes sickness or slower work speeds when operating in unsealed areas (e.g., outside of protected habitats). The rating can be improved through adaptation (gene-modding, drugs) or mitigated through terraforming the planet itself.

## 2. Dependencies
- `src/layer1/pop.rs` (Pop components)
- `src/layer1/needs.rs` (Pop needs and statuses, specifically health/sickness)
- `src/layer1/environment.rs` or equivalent (to determine if an area is unsealed/exposed to the planet's biology)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Health;

    #[test]
    fn test_biocompatibility_reduces_health_in_unsealed_areas() {
        let mut app = App::new();
        app.add_systems(Update, biocompatibility_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Biocompatibility { rating: 0.1 }, // Low rating
            Health { value: 100.0 },
            ExposedToEnvironment, // Marker component for being in an unsealed area
        )).id();

        app.update();

        let health = app.world().get::<Health>(pop_entity).unwrap();
        assert!(health.value < 100.0, "Health should decrease when exposed with low biocompatibility");
    }

    #[test]
    fn test_biocompatibility_does_not_affect_sealed_areas() {
        let mut app = App::new();
        app.add_systems(Update, biocompatibility_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Biocompatibility { rating: 0.1 }, // Low rating
            Health { value: 100.0 },
            // Missing ExposedToEnvironment
        )).id();

        app.update();

        let health = app.world().get::<Health>(pop_entity).unwrap();
        assert_eq!(health.value, 100.0, "Health should not decrease when in a sealed area");
    }

    #[test]
    fn test_high_biocompatibility_prevents_health_loss() {
        let mut app = App::new();
        app.add_systems(Update, biocompatibility_system);

        let pop_entity = app.world_mut().spawn((
            Pop,
            Biocompatibility { rating: 1.0 }, // Perfect rating
            Health { value: 100.0 },
            ExposedToEnvironment,
        )).id();

        app.update();

        let health = app.world().get::<Health>(pop_entity).unwrap();
        assert_eq!(health.value, 100.0, "Health should not decrease with perfect biocompatibility");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Health;

/// Component indicating a Pop's compatibility with the planet's biology (0.0 to 1.0).
#[derive(Component)]
pub struct Biocompatibility {
    pub rating: f32,
}

impl Default for Biocompatibility {
    fn default() -> self {
        Self { rating: 0.5 }
    }
}

/// Marker component indicating the Pop is in an unsealed area.
#[derive(Component)]
pub struct ExposedToEnvironment;

pub fn biocompatibility_system(
    mut query: Query<(&mut Health, &Biocompatibility), With<ExposedToEnvironment>>
) {
    for (mut health, compatibility) in query.iter_mut() {
        // Base damage for zero compatibility
        let base_damage = 1.0;

        // Damage scales inversely with compatibility rating
        let damage = base_damage * (1.0 - compatibility.rating);

        health.value -= damage;
        if health.value < 0.0 {
            health.value = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Currently uses a flat damage value. Instead of just damaging Health, this could apply a `Sick` status effect or debuff working speed (e.g., adding an `AllergicReaction` component that slows action timers).
- Ensure `ExposedToEnvironment` is dynamically added/removed based on the Pop's `GridPosition` and the building they are in or if they are wearing a hazmat suit.
- `Biocompatibility` could be influenced by a global `PlanetaryToxicity` resource, which decreases as terraforming progresses.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Exposed Pops with low biocompatibility lose health over time.
- [ ] Pops with 1.0 biocompatibility or in sealed areas do not lose health from the environment.

## 7. Technical Guidance
- `ExposedToEnvironment` should ideally be managed by a separate system that checks if a Pop's location is inside a sealed building.
- Register `biocompatibility_system` in the main simulation update schedule, potentially running on a fixed timer (e.g., every in-game hour) rather than every tick to balance damage output.

## 8. Questions
*Builder: add questions here if spec is unclear.*
