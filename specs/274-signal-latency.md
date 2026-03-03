# 274: Signal Latency

## 1. Overview

**Layer:** Cross-layer

**Fantasy:** Ruling a galactic empire means dealing with the speed of light. You are an Emperor, not a god.

**Mechanic:** Orders issued to Layer 2/3 entities take time to arrive based on distance. Intel reports are similarly delayed.

## 2. Dependencies
- 146 (Command Center), 094 (System View Architecture)

## 3. RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_order_latency_based_on_distance() {
    let mut world = App::new();
    // Setup a command center and a distant fleet
    let fleet_entity = world.world_mut().spawn(Fleet { position: Vec2::new(100.0, 0.0) }).id();

    // Issue order
    issue_order_to_fleet(&mut world, fleet_entity, Order::Move);

    // Assert order is pending, not applied immediately
    assert!(world.world().get::<PendingOrder>(fleet_entity).is_some());
    assert_eq!(world.world().get::<PendingOrder>(fleet_entity).unwrap().ticks_remaining, 100);
}

#[test]
fn test_order_applied_after_latency() {
    let mut world = App::new();
    let fleet_entity = world.world_mut().spawn((
        Fleet { position: Vec2::new(10.0, 0.0) },
        PendingOrder { order: Order::Move, ticks_remaining: 1 }
    )).id();

    // Tick simulation
    world.update();

    // Assert order is applied
    assert!(world.world().get::<PendingOrder>(fleet_entity).is_none());
    assert!(world.world().get::<ActiveOrder>(fleet_entity).is_some());
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
