# 367 Gravity Inversion Mining

## 1. Overview
A high-tech localized gravity generator that flips gravity for a designated chunk of terrain. This allows Pops to "fall" upwards to mine stalactites or previously unreachable cavern ceilings. However, a power failure while pops are "up" results in a lethal drop. This introduces verticality and extreme danger into the mining loop.

## 2. Dependencies
- `042-energy-system.md` (for power)
- `018-mining-resources.md` (for mining mechanics)
- `153-geological-instability.md` (for terrain height/z-levels)

## 3. RED Phase: Tests First

```rust
// tests/integration/gravity_inversion_test.rs

use crate::layer1::tech::gravity_inversion::*;
use crate::layer1::power::PowerGrid;
use crate::layer1::health::Health;

#[test]
fn test_gravity_generator_flips_gravity_when_powered() {
    let mut app = setup_test_app();
    let gen = app.world_mut().spawn((
        GravityGenerator { radius: 5.0, is_active: true },
        PowerRequirement { amount: 100.0 },
        Transform::default(),
    )).id();

    // Simulate powered state
    app.world_mut().resource_mut::<PowerGrid>().available_power = 200.0;
    app.update();

    // Generator should flip gravity for nearby pops
    let pop = app.world_mut().spawn((
        Pop,
        GravityState::Normal,
        Transform { translation: Vec3::new(2.0, 0.0, 0.0), ..Default::default() },
    )).id();

    app.update();

    let pop_state = app.world().get::<GravityState>(pop).unwrap();
    assert_eq!(*pop_state, GravityState::Inverted);
}

#[test]
fn test_power_failure_causes_lethal_drop() {
    let mut app = setup_test_app();

    let pop = app.world_mut().spawn((
        Pop,
        GravityState::Inverted, // Currently on the ceiling
        Health { current: 100.0, max: 100.0 },
        Transform { translation: Vec3::new(0.0, 10.0, 0.0), ..Default::default() },
    )).id();

    // Power fails!
    app.world_mut().resource_mut::<PowerGrid>().available_power = 0.0;

    // Trigger the fall
    app.update();

    let pop_health = app.world().get::<Health>(pop).unwrap();
    assert!(pop_health.current < 100.0); // Took fall damage

    let pop_state = app.world().get::<GravityState>(pop).unwrap();
    assert_eq!(*pop_state, GravityState::Falling);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/gravity_inversion.rs
use bevy::prelude::*;
use crate::layer1::health::Health;
use crate::layer1::power::{PowerGrid, PowerRequirement};

#[derive(Component)]
pub struct GravityGenerator {
    pub radius: f32,
    pub is_active: bool,
}

#[derive(Component, PartialEq, Debug)]
pub enum GravityState {
    Normal,
    Inverted,
    Falling,
}

pub fn update_gravity_zones(
    power_grid: Res<PowerGrid>,
    mut generators: Query<(&mut GravityGenerator, &PowerRequirement, &Transform)>,
    mut pops: Query<(&mut GravityState, &Transform, &mut Health)>,
) {
    let has_power = power_grid.available_power > 0.0;

    for (mut gen, req, gen_transform) in generators.iter_mut() {
        gen.is_active = has_power && power_grid.available_power >= req.amount;

        for (mut state, pop_transform, mut health) in pops.iter_mut() {
            let distance = gen_transform.translation.distance(pop_transform.translation);

            if distance <= gen.radius {
                if gen.is_active {
                    *state = GravityState::Inverted;
                } else if *state == GravityState::Inverted {
                    // Power failed while inverted
                    *state = GravityState::Falling;
                    health.current = (health.current - 50.0).max(0.0); // Apply fall damage
                }
            } else if *state == GravityState::Inverted {
                // Walked out of range while inverted
                *state = GravityState::Falling;
                health.current = (health.current - 50.0).max(0.0);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding:** Update A* to understand inverted surfaces as walkable terrain when a generator is active.
- **Fall Mechanics:** Calculate fall damage based on `Z-level` differences instead of a flat 50.0.
- **Visuals:** Inverse sprite rendering or UI markers to clearly indicate inverted Pops.

## 6. Acceptance Criteria
- [ ] Tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Coverage >= 85%.
- [ ] Generator correctly inverts Pops in radius when powered.
- [ ] Power loss while inverted applies falling damage.

## 7. Technical Guidance
- Integrate falling damage securely so it accounts for actual distances in Layer 1.
- Pathfinding modifications will require a custom cost closure or bridging logic.

## 8. Questions
*Builder: Add any questions here.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
