# Void Sickness

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Humans weren't meant to live in the black. The stars change you.
**Mechanic:** Pops assigned to off-world duties (Pilots, Station Crew) accumulate "Void Exposure" over time. High exposure leads to the "Void Touched" trait (High Intellect/Perception, Low Empathy/Social). They become strange to surface-dwellers.
**Emergence:** A "Spacer Caste" emerges that refuses to sleep on the planet surface, demanding orbital habitats. Returning pilots struggle to reintegrate with their families.
**Tension:** Rotate crews frequently (logistics headache) to keep them human, or embrace the Void (specialized but alien workforce)?

## 2. Dependencies
- Layer 1 Pop Traits system
- Layer 1/2 Integration (assignment to off-world duties)
- Layer 1 Needs system (specifically Sleep/Rest and Social)

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, PopTraits, Trait};
use crate::layer1::needs::Needs;
use crate::layer2::ship::OffWorldDuty;
// Assume appropriate imports for VoidExposure and related systems

#[test]
fn test_void_exposure_accumulation() {
    let mut app = App::new();
    app.add_systems(Update, process_void_exposure_system);

    let pop = app.world_mut().spawn((
        Pop,
        VoidExposure(0.0),
        OffWorldDuty,
    )).id();

    app.update();

    let exposure = app.world().get::<VoidExposure>(pop).unwrap();
    assert!(exposure.0 > 0.0, "Exposure should increase when on off-world duty");
}

#[test]
fn test_void_touched_trait_application() {
    let mut app = App::new();
    app.add_systems(Update, apply_void_touched_trait_system);

    let pop = app.world_mut().spawn((
        Pop,
        PopTraits::default(),
        VoidExposure(100.0), // High exposure
    )).id();

    app.update();

    let traits = app.world().get::<PopTraits>(pop).unwrap();
    assert!(traits.has(Trait::VoidTouched), "High exposure should apply the VoidTouched trait");
}

#[test]
fn test_void_touched_stat_modifiers() {
    let mut app = App::new();
    // Assuming stat calculation uses traits to apply modifiers
    app.add_systems(Update, apply_trait_stat_modifiers_system);

    let mut base_traits = PopTraits::default();
    base_traits.add(Trait::VoidTouched);

    let pop = app.world_mut().spawn((
        Pop,
        base_traits,
        PopStats::default(),
    )).id();

    app.update();

    let stats = app.world().get::<PopStats>(pop).unwrap();
    assert!(stats.intellect > PopStats::default().intellect, "VoidTouched should boost Intellect");
    assert!(stats.empathy < PopStats::default().empathy, "VoidTouched should penalize Empathy");
}

#[test]
fn test_void_touched_refuses_surface_sleep() {
    let mut app = App::new();
    app.add_systems(Update, utility_eval_system);

    let mut traits = PopTraits::default();
    traits.add(Trait::VoidTouched);

    let mut needs = Needs::default();
    needs.rest = 0.1; // Highly tired

    // Pop on surface (Layer 1)
    let pop = app.world_mut().spawn((
        Pop,
        traits,
        needs,
        Location::Surface,
    )).id();

    app.update();

    let utility_scores = app.world().get::<UtilityScores>(pop).unwrap();
    let sleep_score = utility_scores.get_score(ActionType::Sleep);

    // Normal pop would have a high sleep score here
    assert!(sleep_score < 0.2, "VoidTouched Pop on surface should refuse/heavily penalize sleep");
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

#[derive(Component, Default)]
pub struct VoidExposure(pub f32);

pub fn process_void_exposure_system(
    mut query: Query<&mut VoidExposure, With<OffWorldDuty>>,
    time: Res<Time>,
) {
    for mut exposure in query.iter_mut() {
        exposure.0 += time.delta_secs() * 0.1; // Some base rate
    }
}

pub fn apply_void_touched_trait_system(
    mut query: Query<(&VoidExposure, &mut PopTraits)>,
) {
    for (exposure, mut traits) in query.iter_mut() {
        if exposure.0 >= 100.0 && !traits.has(Trait::VoidTouched) {
            traits.add(Trait::VoidTouched);
        }
    }
}

// Ensure stat modifier logic handles Trait::VoidTouched correctly
// Ensure utility AI for ActionType::Sleep checks for Trait::VoidTouched and Location::Surface
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Ensure `Void Exposure` doesn't leak into unrelated systems. Ensure the trait application is event-driven or systematically clean, avoiding every-tick checks.
- **Performance**: Use Bevy `Changed<VoidExposure>` filters or event triggers (`PopOffWorldEvent`) rather than polling all Pops every frame to calculate exposure.
- **API Improvements**: The stats system might need a more generic modifier framework if `VoidTouched` introduces complex multi-stat changes.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Pops assigned off-world correctly accumulate Void Exposure
- [ ] The `VoidTouched` trait correctly modifies stats and Utility AI weights

## 7. Technical Guidance
- **Gotchas**: Beware of edge cases where a Pop is unassigned from off-world duty while still processing exposure; ensure cleanup is precise.
- **Integration**: The sleep refusal behavior requires carefully modifying `utility_types.rs` or `utility_eval_types.rs` to penalize the "Sleep" action if the Pop is on the surface. You may need a `GridPosition` or `Location` check in the utility scoring function.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Builder: `PopTraits` does not exist (it's `Traits`). `PopStats` does not exist. `ActionType::Sleep` is actually `ActionType::SatisfyRest`. Will need to adjust the RED phase.*
