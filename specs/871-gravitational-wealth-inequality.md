# Specification 871: Gravitational Wealth Inequality

## 1. Overview
**Layer:** Cross-layer (Layer 1 & Layer 2)
**Feature:** Gravitational Wealth Inequality
**Fantasy:** The richest Pops literally live above the rest, reaping the health benefits of lower gravity.
**Mechanic:** Pops living on orbital platforms or low-gravity upper spires age slower and require less food, while Pops deep in the gravity well age faster and suffer joint damage. This creates a literal caste system based on altitude/gravity.

## 2. Dependencies
- Needs `bevy_ecs` setup.
- Layer 1 `Pop` with `Metabolism` and `Age` components.
- Layer 1 `Position` or altitude tracking for gravity modifiers.

## 3. RED Phase: Tests First

```rust
// tests/gravitational_wealth_tests.rs
use bevy::prelude::*;
use scale::layer1::pop::{Pop, Age, Metabolism};
use scale::layer1::gravity::{GravityZone, apply_gravity_effects_system};

#[test]
fn test_low_gravity_slows_aging_and_metabolism() {
    let mut app = App::new();
    app.add_systems(Update, apply_gravity_effects_system);

    let high_born = app.world_mut().spawn((
        Pop,
        Age { value: 30.0, aging_rate: 1.0 },
        Metabolism { food_consumption: 1.0 },
        GravityZone { gravity_multiplier: 0.5 }, // Low G
    )).id();

    app.update();

    let age = app.world().get::<Age>(high_born).unwrap();
    let metabolism = app.world().get::<Metabolism>(high_born).unwrap();

    assert!(age.aging_rate < 1.0);
    assert!(metabolism.food_consumption < 1.0);
}

#[test]
fn test_high_gravity_accelerates_aging_and_causes_damage() {
    let mut app = App::new();
    app.add_systems(Update, apply_gravity_effects_system);

    let low_born = app.world_mut().spawn((
        Pop,
        Age { value: 30.0, aging_rate: 1.0 },
        Metabolism { food_consumption: 1.0 },
        GravityZone { gravity_multiplier: 2.0 }, // High G
    )).id();

    app.update();

    let age = app.world().get::<Age>(low_born).unwrap();
    let metabolism = app.world().get::<Metabolism>(low_born).unwrap();

    assert!(age.aging_rate > 1.0);
    assert!(metabolism.food_consumption > 1.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/gravity.rs
use bevy::prelude::*;
use crate::layer1::pop::{Age, Metabolism};

#[derive(Component)]
pub struct GravityZone {
    pub gravity_multiplier: f32,
}

pub fn apply_gravity_effects_system(
    mut query: Query<(&GravityZone, &mut Age, &mut Metabolism)>,
) {
    for (gravity, mut age, mut metabolism) in query.iter_mut() {
        // Base rate is 1.0. Adjust based on gravity.
        age.aging_rate = 1.0 * gravity.gravity_multiplier;
        metabolism.food_consumption = 1.0 * gravity.gravity_multiplier;

        // High gravity causes exponential strain
        if gravity.gravity_multiplier > 1.5 {
            age.aging_rate *= 1.2;
            metabolism.food_consumption *= 1.2;
        }

        // Low gravity provides exponential relief
        if gravity.gravity_multiplier < 0.8 {
            age.aging_rate *= 0.8;
            metabolism.food_consumption *= 0.8;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Social Impact**: Add logic to generate "Class Friction" Unrest events when Pops with vastly different `aging_rate` modifiers interact or reside in the same sector.
- **Visuals**: Hook into the UI to display the "Gravity Strain" or "Gravity Privilege" status effect on Pop inspection.
- **Chronicle**: Emit an event when a Pop dies of "Gravity Strain" (premature aging).

## 6. Acceptance Criteria
- [ ] Tests pass (`cargo test`).
- [ ] Coverage for new module >= 85%.
- [ ] `apply_gravity_effects_system` correctly modulates aging and food consumption based on the `GravityZone` multiplier.

## 7. Technical Guidance
- The `GravityZone` component should ideally be assigned to tiles or habitats, and Pops inherit its value based on their location. For the minimal implementation, attaching it directly to the Pop is fine for testing.

## 8. Questions
*Builder: add questions here if spec is unclear.*
