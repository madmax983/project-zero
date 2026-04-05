# Sub-Zero Hibernation Sickness (Spec 797)

## 1. Overview
Pops transported over long distances in Cryo-Sleep have a chance to develop "Hibernation Sickness." They physically function normally, but their needs decay at half speed, and they cannot experience morale boosts. They act as emotionless automatons, terrifying their non-afflicted peers and creating severe social friction.

This creates tension: the incredibly high productivity of the afflicted pops vs. the severe collateral damage to the mental health of normal colonists working alongside them.

## 2. Dependencies
- `src/layer1/population/mod.rs` (Pop traits and needs)
- `src/layer1/needs.rs` (Needs decay systems)
- `src/layer1/morale.rs` (Morale modifiers)
- `src/layer1/social.rs` (Social interactions and stress radii)

## 3. RED Phase: Tests First

```rust
// tests/layer1/test_hibernation_sickness.rs

use bevy::prelude::*;
use scale::layer1::population::{Pop, TraitHibernationSickness};
use scale::layer1::needs::Needs;
use scale::layer1::morale::Morale;
use scale::layer1::social::SocialStressRadius;

#[test]
fn test_sickness_halves_need_decay() {
    let mut app = App::new();
    // ... setup needs system ...

    let healthy_pop = app.world_mut().spawn((Pop::new("Healthy"), Needs { hunger: 100.0, ..default() })).id();
    let sick_pop = app.world_mut().spawn((Pop::new("Sick"), Needs { hunger: 100.0, ..default() }, TraitHibernationSickness)).id();

    app.update(); // Advance metabolism

    let h_needs = app.world().get::<Needs>(healthy_pop).unwrap();
    let s_needs = app.world().get::<Needs>(sick_pop).unwrap();

    // Assert: Sick pop's hunger decayed slower
    assert!(h_needs.hunger < s_needs.hunger);
}

#[test]
fn test_sickness_blocks_positive_morale() {
    let mut app = App::new();
    // ... setup morale ...

    let sick_pop = app.world_mut().spawn((Pop::new("Sick"), Morale { current: 50.0, ..default() }, TraitHibernationSickness)).id();

    // Act: Give a massive positive boost (like a festival or good meal)
    app.world_mut().resource_mut::<Events<MoraleBoostEvent>>().send(MoraleBoostEvent { target: sick_pop, amount: 20.0 });
    app.update();

    let morale = app.world().get::<Morale>(sick_pop).unwrap();

    // Assert: Morale hasn't moved
    assert_eq!(morale.current, 50.0);
}

#[test]
fn test_sickness_emits_stress_to_healthy_neighbors() {
    let mut app = App::new();
    // ... setup spatial and social ...

    let sick_pop = app.world_mut().spawn((
        Pop::new("Sick"),
        Transform::from_translation(Vec3::ZERO),
        TraitHibernationSickness,
        SocialStressRadius { radius: 2.0, strength: 5.0 },
    )).id();

    let healthy_pop = app.world_mut().spawn((
        Pop::new("Healthy"),
        Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        Morale { current: 100.0, ..default() },
    )).id();

    app.update(); // Social systems run

    let h_morale = app.world().get::<Morale>(healthy_pop).unwrap();

    // Assert: Healthy pop took stress damage from proximity to the sick pop
    assert!(h_morale.current < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/hibernation_sickness.rs

use bevy::prelude::*;
use crate::layer1::needs::Needs;
use crate::layer1::morale::{Morale, MoraleBoostEvent};
use crate::layer1::social::SocialStressRadius;

#[derive(Component)]
pub struct TraitHibernationSickness;

pub fn sickness_needs_modifier_system(
    mut sick_pops: Query<&mut Needs, With<TraitHibernationSickness>>,
) {
    // In the real system, this would modify the decay rate, not hardcode the addition.
    // For minimal passing, just "refund" half the decay.
    for mut needs in sick_pops.iter_mut() {
        needs.hunger += 0.5; // Assuming decay was 1.0
        needs.rest += 0.5;
    }
}

pub fn block_sick_morale_boost_system(
    mut events: EventReader<MoraleBoostEvent>,
    mut queries: Query<&mut Morale, With<TraitHibernationSickness>>,
) {
    // Intercept event handling, or reset morale if handled upstream.
    for event in events.read() {
        if let Ok(mut morale) = queries.get_mut(event.target) {
            // Nullify the boost
            morale.current -= event.amount;
        }
    }
}

pub fn sick_stress_aura_system(
    sick_query: Query<(&Transform, &SocialStressRadius), With<TraitHibernationSickness>>,
    mut healthy_query: Query<(&Transform, &mut Morale), Without<TraitHibernationSickness>>,
) {
    for (sick_tf, aura) in sick_query.iter() {
        for (healthy_tf, mut morale) in healthy_query.iter_mut() {
            let dist = sick_tf.translation.distance(healthy_tf.translation);
            if dist <= aura.radius {
                morale.current -= aura.strength;
            }
        }
    }
}

pub struct HibernationSicknessPlugin;
impl Plugin for HibernationSicknessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            sickness_needs_modifier_system,
            block_sick_morale_boost_system,
            sick_stress_aura_system,
        ));
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
    - The `sickness_needs_modifier_system` is a hack that adds back needs. It should hook into the global `metabolism_system`'s multiplier calculation instead of overriding it manually.
    - Instead of reverting morale boosts after they happen (`block_sick_morale_boost_system`), the event handler itself should check for the trait before applying.
    - The O(N*M) proximity check in `sick_stress_aura_system` will scale poorly. It should use the spatial grid lookup (`GridPosition`) to find adjacent pops.
- **Code Smells**: Magic numbers for the refund amounts (0.5).
- **Performance**: Spatial querying for the aura needs spatial hashing to avoid bad scaling with large populations.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] `TraitHibernationSickness` correctly halves all need decays
- [ ] Pops with the trait cannot gain positive morale
- [ ] Nearby pops without the trait suffer constant morale decay

## 7. Technical Guidance
- **Integration Points**: Tie the trait generation to the Cryo-ship landing events or generation ship arrival events.
- **Gotchas**: Make sure negative morale events still apply to sick pops! They can't feel joy, but they can still break under pressure (or maybe they don't break at all, and just act as permanent machines until they die).

## 8. Questions
*Builder: add questions here if spec is unclear.*
