# Specification 475: Epidemic Denial

## 1. Overview
A plague is ravaging the colony, but half the population thinks it's a hoax by the governor. During an outbreak, Pops with "Rebellious" or "Paranoid" traits have a chance to gain the `Condition::Denial` condition. They refuse to visit the hospital, take medicine, or respect quarantine zones, actively spreading the disease and reducing the overall health rating. This introduces an unpredictable element into medical triage and adds tension between enforcing quarantines (causing unrest) versus letting the infection spread.

## 2. Dependencies
- Core disease/outbreak systems (Condition/Infection components).
- Pop Trait system (`Trait::Rebellious`, `Trait::Paranoid`).
- Quarantine zoning and medical behavior AI (Layer 1 utility AI).

## 3. RED Phase: Tests First

```rust
#[test]
fn test_epidemic_denial_affects_paranoid_and_rebellious_pops() {
    // Arrange: Create a world with an active outbreak event/status.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // ... add required custom plugins

    // Spawn a pop with the Paranoid trait and an infection
    let pop_paranoid = app.world_mut().spawn((
        PopBundle::default(),
        Traits(vec![Trait::Paranoid]),
        Infection { severity: 50.0 },
    )).id();

    // Spawn a pop with the Rebellious trait and an infection
    let pop_rebellious = app.world_mut().spawn((
        PopBundle::default(),
        Traits(vec![Trait::Rebellious]),
        Infection { severity: 50.0 },
    )).id();

    // Spawn a pop without these traits
    let pop_obedient = app.world_mut().spawn((
        PopBundle::default(),
        Traits(vec![Trait::Diligent]),
        Infection { severity: 50.0 },
    )).id();

    // Act: Run the denial condition generation system
    app.update();

    // Assert: Check that Paranoid/Rebellious pops gained the Denial condition
    assert!(app.world().entity(pop_paranoid).contains::<DenialCondition>());
    assert!(app.world().entity(pop_rebellious).contains::<DenialCondition>());
    assert!(!app.world().entity(pop_obedient).contains::<DenialCondition>());
}

#[test]
fn test_denial_pops_ignore_quarantine_zones() {
    // Arrange: Set up a Pop with DenialCondition and a movement intention towards a Quarantine Zone
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let pop = app.world_mut().spawn((
        PopBundle::default(),
        DenialCondition,
        PathingIntention { target: QUARANTINE_ZONE_POS },
    )).id();

    // Act: Run the quarantine enforcement system
    app.update();

    // Assert: Pop's intention is NOT blocked/redirected because they deny the quarantine
    let intention = app.world().get::<PathingIntention>(pop).unwrap();
    assert_eq!(intention.target, QUARANTINE_ZONE_POS);
}

#[test]
fn test_denial_pops_refuse_medical_treatment() {
    // Arrange: Spawn a sick pop with DenialCondition
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let pop = app.world_mut().spawn((
        PopBundle::default(),
        DenialCondition,
        Infection { severity: 50.0 },
        Need::Medical(100.0), // Need high medical attention
    )).id();

    // Act: Run Utility AI to decide next action
    app.update();

    // Assert: The pop should NOT have a `SeekTreatment` action despite high medical need
    assert!(!app.world().entity(pop).contains::<SeekTreatmentAction>());
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct DenialCondition;

pub fn epidemic_denial_system(
    mut commands: Commands,
    query: Query<(Entity, &Traits, &Infection), Without<DenialCondition>>,
) {
    for (entity, traits, infection) in query.iter() {
        // Simple minimal threshold: if infected and has specific traits, they gain Denial
        if traits.0.contains(&Trait::Paranoid) || traits.0.contains(&Trait::Rebellious) {
            // In a real implementation, this might have a probability factor.
            // For minimal pass, we just insert it.
            commands.entity(entity).insert(DenialCondition);
        }
    }
}

// In the Utility AI medical scoring system:
pub fn score_medical_need_system(
    mut query: Query<(&mut ActionScore, &Need), With<DenialCondition>>,
) {
    for (mut score, need) in query.iter_mut() {
        // If they have DenialCondition, they score medical treatment extremely low
        if let Need::Medical(_) = need {
            score.value = 0.0;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Probabilistic Denial**: Refactor the assignment of `DenialCondition` to use a probability based on the severity of the outbreak and the exact traits, rather than a guaranteed 100% chance.
- **Lore Integration**: Ensure the Lexicon terms "the Hoax" and "false-plague" are used when creating notification events or logs related to pops breaching quarantine.
- **Unrest Mechanic**: Integrate the consequences of forced quarantine. If the player forcibly locks down a sector, Pops with the `DenialCondition` should generate massive Unrest.
- **Contagion radius**: Ensure Pops with `DenialCondition` have an active "spread radius" since they refuse to wear masks or stay isolated.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Pops with `Paranoid` or `Rebellious` traits have a chance to gain `DenialCondition` during an outbreak.
- [ ] Pops with `DenialCondition` actively ignore quarantine movement restrictions.
- [ ] Pops with `DenialCondition` refuse to execute `SeekTreatmentAction`.

## 7. Technical Guidance
- Integrate into the `Layer1SystemSet::Observation` or `Decision` schedules.
- `DenialCondition` should act as a strong modifier to the Utility AI, overriding the otherwise overwhelming urge to seek medical care when health is low.
- For movement, you will likely need to intercept the `PathingIntention` and bypass the standard `QuarantineZone` collision checks for entities carrying `DenialCondition`.
- Be mindful of Bevy ECS performance when checking for `Trait` arrays.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
