# 1188: Corrosive Atmosphere

## 1. Overview
**Layer:** 1

**Fantasy:** The air hates you. Metal screams when it touches the wind.

**Mechanic:** On certain worlds, the atmosphere deals constant durability damage to external structures and items. Requires "Shielding" (Paint/Energy) or building underground/indoors to mitigate.

**Emergence:** You forget to re-paint the comms tower. It dissolves mid-transmission, cutting off your plea for help right as the storm hits.

**Tension:** Maintenance cost (Paint) vs. Subterranean constraints (Space).

---

## 2. Dependencies
- Base ECS system
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_corrosive_atmosphere_damage() {
    // Arrange: Set up corrosive atmosphere and an unshielded building
    let mut app = App::new();
    app.insert_resource(Atmosphere { corrosiveness: 5.0 });
    app.add_systems(Update, apply_corrosion);
    let building = app.world_mut().spawn((Building, Health { current: 100.0, max: 100.0 })).id();

    // Act: Run simulation tick
    let mut time = Time::default(); time.update(); app.insert_resource(time);
    app.update();

    // Assert: Building health decreases
    let health = app.world().get::<Health>(building).unwrap();
    assert!(health.current < 100.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn apply_corrosion(
    atmosphere: Res<Atmosphere>,
    mut query: Query<&mut Health, Without<Shield>>,
    time: Res<Time>,
) {
    for mut health in query.iter_mut() {
        health.current -= atmosphere.corrosiveness * time.delta_seconds();
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Consider tracking corrosion damage separately from health so Pops can repair the outer layer.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code

## 7. Technical Guidance
- Ensure events are processed correctly in the update schedule.
- Remember to explicitly register all new systems, events, and resources to the main game app.

## 8. Questions
*Builder: add questions here if spec is unclear.*
