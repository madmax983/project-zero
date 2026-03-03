# 277: Cultural Drift

## 1. Overview

**Layer:** Cross-layer

**Fantasy:** Colonies isolated from the homeworld develop their own culture. Space is big, and maintaining a unified empire is hard.

**Mechanic:** Distance + Time = Divergence. Colonies far away drift in ethics/loyalty. Regular communication (expensive) reduces drift.

## 2. Dependencies
- 197 (Civic Ideology), 209 (Planetary Governance), 146 (Command Center)

## 3. RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_colony_cultural_drift_over_time() {
    let mut world = App::new();
    let colony = world.world_mut().spawn((Colony, DistanceToHomeworld(100.0), CulturalAlignment(1.0))).id();

    // Tick simulation for significant time
    for _ in 0..100 {
        world.update();
    }

    // Assert alignment has drifted
    let alignment = world.world().get::<CulturalAlignment>(colony).unwrap().0;
    assert!(alignment < 1.0);
}

#[test]
fn test_communication_reduces_drift() {
    let mut world = App::new();
    let colony = world.world_mut().spawn((Colony, DistanceToHomeworld(100.0), CulturalAlignment(0.5))).id();

    // Send communication event
    world.send_event(CommunicationReceived { colony });
    world.update();

    // Assert alignment is restored
    let alignment = world.world().get::<CulturalAlignment>(colony).unwrap().0;
    assert!(alignment > 0.5);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- Identify refactoring opportunities
- Extract magic numbers into `layer1::constants` or appropriate configuration
- Improve system performance using optimized queries
- Improve API for better modularity

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Implement feature in a separate module.
- Register the system in the `ScheduleBuilder` or Bevy App.
- Use the correct event dispatch patterns if interacting with ECS.
- Ensure that we isolate complex logic into helper functions instead of large systems.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
