# Haunted Assembly Lines (Spec 817)

## 1. Overview
When a Pop dies in a tragic industrial accident (e.g., crushed by machinery, burned in a fusion core), the surrounding factory or infrastructure becomes "Haunted." This building gains a permanent "Echo of the Fallen" trait. It operates at 150% efficiency (as if the ghost is still working), but living Pops assigned to work there suffer massive stress and paranoia penalties, eventually refusing to enter.

## 2. Dependencies
- Layer 1 core systems (Pops, Buildings, Efficiency, Stress)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_haunted_assembly_lines_basic_behavior() {
    // Arrange
    let mut app = App::new();

    // Setup a factory and a worker
    let factory = app.world_mut().spawn((Building, Efficiency(1.0))).id();
    let pop = app.world_mut().spawn((Pop, Stress(0.0), AssignedTo(factory))).id();

    // Act
    // Simulate industrial accident death
    app.world_mut().send_event(PopDiedInAccidentEvent { pop, location: factory });
    app.update();

    // Assert
    // Factory should gain Echo of the Fallen and 150% efficiency
    assert!(app.world().entity(factory).contains::<EchoOfTheFallen>());
    assert_eq!(app.world().get::<Efficiency>(factory).unwrap().0, 1.5);

    // New worker assigned should gain massive stress over time
    let new_worker = app.world_mut().spawn((Pop, Stress(0.0), AssignedTo(factory))).id();
    app.update(); // Tick time
    assert!(app.world().get::<Stress>(new_worker).unwrap().0 > 0.5);
}

#[test]
fn test_haunted_assembly_lines_edge_cases() {
    // Arrange
    let mut app = App::new();

    // Setup an already haunted factory
    let factory = app.world_mut().spawn((Building, Efficiency(1.5), EchoOfTheFallen)).id();
    let pop = app.world_mut().spawn((Pop, Stress(0.0), AssignedTo(factory))).id();

    // Act
    // Simulate another industrial accident death in the same haunted factory
    app.world_mut().send_event(PopDiedInAccidentEvent { pop, location: factory });
    app.update();

    // Assert
    // Efficiency should not stack beyond 150%
    assert_eq!(app.world().get::<Efficiency>(factory).unwrap().0, 1.5);

    // Worker refusing to enter at max stress
    let stressed_worker = app.world_mut().spawn((Pop, Stress(1.0), AssignedTo(factory))).id();
    app.update();
    assert!(app.world().get::<AssignedTo>(stressed_worker).is_none());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- Abstract the "haunting" mechanic so it can be extended to other building types.
- Ensure the stress gain formula is tunable and balanced with other stress factors.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Listen for `PopDiedInAccidentEvent` to trigger the haunting logic.
- Use an `Update` system to gradually apply stress to living Pops working in haunted buildings.
- Use a component `EchoOfTheFallen` to track haunted status.

## 8. Questions
*Builder: add questions here if spec is unclear.*
