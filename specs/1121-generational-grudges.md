# 1121: Generational Grudges

## 1. Overview

Feuds that outlast the people who started them. When a Pop is wronged by another, they form a "Grudge". If the Grudge is not resolved before death, it is inherited by their descendants. Pops with active Grudges will refuse to work in the same room and may sabotage the other family's efforts.

## 2. Dependencies

- None

## 3. RED Phase: Tests First

```rust
use bevy::prelude::*;
use crate::layer1::social::SocialPlugin;

#[test]
fn test_grudge_formation() {
    let mut app = App::new();
    app.add_plugins(SocialPlugin);

    // Arrange: Setup two pops
    let pop1 = app.world_mut().spawn_empty().id();
    let pop2 = app.world_mut().spawn_empty().id();

    // Act: Simulate a wrong (e.g. theft or assault)
    // Send event or trigger system to simulate wrong-doing

    // Assert: Verify that a Grudge component or relation is created
    // assert!(app.world().get::<Grudge>(pop1).is_some());
}

#[test]
fn test_grudge_inheritance() {
    let mut app = App::new();
    app.add_plugins(SocialPlugin);

    // Arrange: Setup a parent with a grudge and a child
    let target = app.world_mut().spawn_empty().id();
    let parent = app.world_mut().spawn((Grudge { target, intensity: 1.0 }, )).id();
    let child = app.world_mut().spawn_empty().id();

    // Relate child to parent

    // Act: Simulate parent death

    // Assert: Verify child inherits the grudge
    // assert!(app.world().get::<Grudge>(child).is_some());
}

#[test]
fn test_grudge_work_refusal() {
    let mut app = App::new();
    app.add_plugins(SocialPlugin);

    // Arrange: Setup two pops with a grudge against each other
    let target = app.world_mut().spawn_empty().id();
    let pop = app.world_mut().spawn((Grudge { target, intensity: 1.0 }, )).id();

    // Act: Attempt to assign them to the same workplace

    // Assert: Verify assignment fails or efficiency drops
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Grudge {
    pub target: Entity,
    pub intensity: f32,
}

// Add systems to form grudges based on negative social interactions,
// inherit them on death if unresolved, and
// prevent pops with grudges from working in the same location
```

## 5. REFACTOR Phase: Quality & Design

- Optimize lookup of grudges, possibly using Bevy's built-in relational mechanics if available.
- Integrate with existing social/memory systems to allow resolution of grudges over time.
- Identify code smells in the work assignment logic to ensure checking grudges does not bottleneck simulation ticks.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified (Grudge inheritance and work refusal)

## 7. Technical Guidance

- Consider using Bevy's relationship components or a custom component storing a list/set of targets if pops can hold multiple grudges.
- Ensure the death system propagates grudges *before* the parent entity is despawned.

## 8. Questions

*Builder: add questions here if spec is unclear.*
