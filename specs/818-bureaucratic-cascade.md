# Bureaucratic Cascade (Spec 818)

## 1. Overview
Layer 3 diplomatic or economic agreements generate "Administrative Debt" on Layer 1 colonies. This debt manifests as physical "Paperwork Tasks" that Pops must complete at Administrator Desks. If the tasks aren't completed in time, the Layer 3 agreement temporarily fails—trade routes halt, alliances fracture, or supply drops cease.

## 2. Dependencies
- Layer 3 Diplomatic/Economic Agreements
- Layer 1 core systems (Pops, Tasks, Administrator Desks)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_bureaucratic_cascade_basic_behavior() {
    // Arrange
    let mut app = App::new();

    // Setup Layer 3 agreement
    let agreement = app.world_mut().spawn((DiplomaticAgreement { active: true }, AdministrativeDebt(0.0))).id();

    // Setup Layer 1 colony with admin desk and pop
    let desk = app.world_mut().spawn(AdministratorDesk).id();
    let pop = app.world_mut().spawn((Pop, AssignedTo(desk))).id();

    // Act
    // Simulate generation of administrative debt from the agreement over time
    app.world_mut().send_event(TimePassedEvent(1.0));
    app.update();

    // Assert
    // Debt should increase
    assert!(app.world().get::<AdministrativeDebt>(agreement).unwrap().0 > 0.0);

    // Pop working at desk should reduce debt
    let initial_debt = app.world().get::<AdministrativeDebt>(agreement).unwrap().0;
    app.world_mut().send_event(TaskCompletedEvent { pop, task_type: TaskType::Paperwork });
    app.update();
    assert!(app.world().get::<AdministrativeDebt>(agreement).unwrap().0 < initial_debt);
}

#[test]
fn test_bureaucratic_cascade_edge_cases() {
    // Arrange
    let mut app = App::new();

    // Setup Layer 3 agreement with max debt threshold
    let agreement = app.world_mut().spawn((DiplomaticAgreement { active: true }, AdministrativeDebt(100.0))).id();

    // Act
    // Simulate debt exceeding threshold
    app.world_mut().send_event(TimePassedEvent(1.0));
    app.update(); // Let debt grow past 100

    // Assert
    // Agreement should become inactive
    assert!(!app.world().get::<DiplomaticAgreement>(agreement).unwrap().active);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN
```

## 5. REFACTOR Phase: Quality & Design
- Create a generalized bridge system to map Layer 3 agreements to Layer 1 debt/tasks.
- Ensure the rate of debt generation scales appropriately with the complexity of the agreement.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Listen for time ticks or specific events to increment `AdministrativeDebt`.
- Use a system to check debt thresholds and temporarily suspend `DiplomaticAgreement` components if exceeded.
- Create a `PaperworkTask` that Pops can perform at `AdministratorDesk` entities to reduce `AdministrativeDebt`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
