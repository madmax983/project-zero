# 1187: Pneumatic Tubes

## 1. Overview
**Layer:** 1

**Fantasy:** The thwump of a canister arriving. Futurama-style logistics.

**Mechanic:** Expensive piping that instantly moves small items (Food, Mail, Samples) between buildings using pressure. Can clog if overused or leak pressure if damaged.

**Emergence:** You use tubes to feed the high-security prison block. A rebel stuffs a homemade bomb in the tube. *Thwump*. Boom. The kitchen explodes.

**Tension:** Instant transport (High Cost/Risk) vs. Haulers (Slow/Reliable).

---

## 2. Dependencies
- Base ECS system
- Event bus

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pneumatic_tube_transport() {
    // Arrange: Create two connected tube nodes and an item
    let mut app = App::new();
    app.add_systems(Update, process_pneumatic_tubes);
    let node_a = app.world_mut().spawn(TubeNode).id();
    let node_b = app.world_mut().spawn(TubeNode).id();
    app.world_mut().spawn((Item, InTube { from: node_a, to: node_b, progress: 0.0 }));

    // Act: Advance time
    let mut time = Time::default(); time.update(); app.insert_resource(time); // Note: instantiate properly in tests
    app.update();

    // Assert: Item progress increases
    let item = app.world().query::<&InTube>().single(app.world());
    assert!(item.progress > 0.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
fn process_pneumatic_tubes(
    mut query: Query<&mut InTube>,
    time: Res<Time>,
) {
    for mut tube in query.iter_mut() {
        tube.progress += time.delta_seconds() * 10.0;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Refactor into a proper physics/movement layer instead of raw progress floats if tube routing becomes complex.

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
