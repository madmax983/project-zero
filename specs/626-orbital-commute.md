# 626: Orbital Commute

## 1. Overview
The "Orbital Commute" feature allows Pops whose homes are located on the planetary surface (Layer 1) to be assigned jobs on orbital stations or platforms (Layer 2), establishing cross-layer commuting behavior via Shuttle Routes. This mechanic ties together the planetary map and orbit, highlighting the logistical and fuel constraints of relying on distributed infrastructure while emphasizing the "daily grind in space."

## 2. Dependencies
- `004` Basic Building
- `009` Job Assignment System
- `152` Orbital Stations
- Layer 2 nodes and connection handling

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_pop_can_be_assigned_orbital_job() {
    // Arrange: Setup test data
    let mut app = App::new();
    let pop = app.world.spawn((Pop, Home { layer: 1 })).id();
    let station = app.world.spawn(OrbitalStation).id();
    let orbital_job = app.world.spawn(Job { location: station, layer: 2 }).id();

    // Act: Call the feature
    assign_job(&mut app.world, pop, orbital_job);

    // Assert: Verify expected behavior
    let assignment = app.world.get::<JobAssignment>(pop).unwrap();
    assert_eq!(assignment.job_id, orbital_job);
    assert_eq!(assignment.layer, 2);
}

#[test]
fn test_orbital_commute_consumes_fuel_and_time() {
    // Arrange: Setup test data
    let mut app = App::new();
    let pop = app.world.spawn((Pop, CommutingState::default())).id();
    let route = app.world.spawn(ShuttleRoute { fuel_cost: 10, time_cost: 2.0 }).id();
    app.insert_resource(FuelReserve(100));

    // Act: Call the feature
    process_commute(&mut app.world, pop, route);

    // Assert: Verify expected behavior
    let fuel = app.world.get_resource::<FuelReserve>().unwrap();
    assert_eq!(fuel.0, 90);

    let state = app.world.get::<CommutingState>(pop).unwrap();
    assert_eq!(state.time_remaining, 2.0);
}

#[test]
fn test_commute_fails_when_fuel_is_insufficient() {
    // Test boundary conditions
    let mut app = App::new();
    let pop = app.world.spawn((Pop, CommutingState::default())).id();
    let route = app.world.spawn(ShuttleRoute { fuel_cost: 10, time_cost: 2.0 }).id();
    app.insert_resource(FuelReserve(5)); // Not enough fuel

    // Act
    process_commute(&mut app.world, pop, route);

    // Assert
    let state = app.world.get::<CommutingState>(pop).unwrap();
    assert!(state.is_stranded);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED → GREEN

pub struct JobAssignment {
    pub job_id: Entity,
    pub layer: u8,
}

pub fn assign_job(world: &mut World, pop: Entity, job: Entity) {
    let job_component = world.get::<Job>(job).unwrap();
    world.entity_mut(pop).insert(JobAssignment {
        job_id: job,
        layer: job_component.layer,
    });
}

pub fn process_commute(world: &mut World, pop: Entity, route: Entity) {
    let route_comp = world.get::<ShuttleRoute>(route).unwrap();
    let mut fuel = world.get_resource_mut::<FuelReserve>().unwrap();

    let mut state = world.get_mut::<CommutingState>(pop).unwrap();

    if fuel.0 >= route_comp.fuel_cost {
        fuel.0 -= route_comp.fuel_cost;
        state.time_remaining = route_comp.time_cost;
        state.is_stranded = false;
    } else {
        state.is_stranded = true;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring:** Extract commute verification logic into a `ShuttleSystem` that checks global availability of shuttles and fuel.
- **Code Smells:** Direct manipulation of `FuelReserve` resource in systems handling individual pops might be a bottleneck; consider using events (`CommuteRequestedEvent`, `CommuteApprovedEvent`) for batching.
- **Design:** Ensure that Pops who are stranded log a memory and drop their morale. Adjust the UI to reflect a pop's transit status.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Integrate with existing `Utility AI` by adding a large negative utility to jobs on a different layer if fuel is low or shuttles are unavailable.
- Introduce `CommutingState` as an active component to lock out other behaviors while in transit.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
