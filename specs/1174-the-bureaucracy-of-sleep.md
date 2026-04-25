# 1174: The Bureaucracy of Sleep

## 1. Overview
In highly regulated colonies, even the act of sleeping requires filing forms. "Sleep Permits" are issued by the Administration. If a Pop is tired but cannot secure a Sleep Permit (due to bureaucratic backlog or lost paperwork), they are forced to stay awake. This leads to extreme exhaustion, hallucination events, and potentially the Pop collapsing on the job.

## 2. Dependencies
- Layer 1 `Pop` needs system (specifically `Tiredness` / `Energy`).
- Layer 1 `Utility AI` (the `Sleep` action).
- Administration / Bureaucracy job output tracking.

## 3. RED Phase: Tests First

```rust
#[test]
fn test_sleep_requires_permit() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, evaluate_sleep_action_system);

    // Pop is very tired, but has no permit
    let pop = app.world_mut().spawn((
        Pop,
        Energy(10.0),
        ActionScore::default()
    )).id();

    // Act
    app.update();

    // Assert: Sleep score is 0 because no permit exists
    let score = app.world().get::<ActionScore>(pop).unwrap().sleep_score;
    assert_eq!(score, 0.0, "Sleep score should be 0 without a permit");
}

#[test]
fn test_permit_allows_sleep() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, evaluate_sleep_action_system);

    // Pop is tired AND has a permit
    let pop = app.world_mut().spawn((
        Pop,
        Energy(10.0),
        SleepPermit,
        ActionScore::default()
    )).id();

    // Act
    app.update();

    // Assert: Sleep score is > 0
    let score = app.world().get::<ActionScore>(pop).unwrap().sleep_score;
    assert!(score > 0.0, "Sleep score should be > 0 with a permit");
}

#[test]
fn test_exhaustion_collapse() {
    // Arrange
    let mut app = App::new();
    app.add_systems(Update, exhaustion_collapse_system);

    // Pop has 0 energy and no permit
    let pop = app.world_mut().spawn((
        Pop,
        Energy(0.0),
    )).id();

    // Act
    app.update();

    // Assert: Pop gains Collapsed state
    assert!(app.world().get::<Collapsed>(pop).is_some(), "Pop should collapse at 0 energy");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Pop;

#[derive(Component)]
pub struct Energy(pub f32);

#[derive(Component)]
pub struct SleepPermit;

#[derive(Component, Default)]
pub struct ActionScore {
    pub sleep_score: f32,
    pub work_score: f32,
}

#[derive(Component)]
pub struct Collapsed;

pub fn evaluate_sleep_action_system(
    mut pops: Query<(&Energy, &mut ActionScore, Option<&SleepPermit>), With<Pop>>,
) {
    for (energy, mut score, permit) in pops.iter_mut() {
        if permit.is_some() && energy.0 < 50.0 {
            // Simplified scoring: lower energy = higher score
            score.sleep_score = 100.0 - energy.0;
        } else {
            score.sleep_score = 0.0;
        }
    }
}

pub fn exhaustion_collapse_system(
    mut commands: Commands,
    pops: Query<(Entity, &Energy), (With<Pop>, Without<Collapsed>)>,
) {
    for (entity, energy) in pops.iter() {
        if energy.0 <= 0.0 {
            commands.entity(entity).insert(Collapsed);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Permit Generation**: Needs a system where Administrative buildings actually generate `SleepPermit` resources or components and distribute them to the most tired Pops.
- **Hallucinations**: Pops near 0 energy should gain a `Hallucinating` component before collapsing, which could cause them to perform erratic actions or fight invisible enemies.
- **Policy Toggle**: This entire mechanic should probably be gated behind a `Policy` (e.g., "Strict Sleep Scheduling") so it doesn't punish early game colonies.

## 6. Acceptance Criteria (Testable!)
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage $\ge$ 85% for new code.
- [ ] Pops cannot score the sleep action without a SleepPermit, leading to collapse at 0 energy.

## 7. Technical Guidance
- Integrate with `evaluate_actions_system` to ensure `SleepPermit` is a hard requirement *only* if the relevant policy is active.
- Collapsed pops should probably drop whatever they are carrying and require rescue or slow recovery.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* Implement the simplest possible version for the MVP. Advanced behaviors and edge cases will be deferred to future specifications.
