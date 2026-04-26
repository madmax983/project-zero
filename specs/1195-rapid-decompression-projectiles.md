# 1195: Rapid Decompression Projectiles

## 1. Overview
**Layer:** 1

**Fantasy:** The wind is a weapon. A hull breach turns a screwdriver into a bullet.

**Mechanic:** When a room decompresses violently, unsecured items (Tools, Debris) become projectiles traveling towards the breach. They deal damage to Pops and fragile buildings (Glass, Screens) in their path.

**Emergence:** You open the airlock to vent a fire. A wrench flies out, shattering the control panel for the *inner* door, locking you in the vacuum with the fire.

**Tension:** Secure loose items (Storage time) vs. Emergency venting (Speed).

## 2. Dependencies
- Base ECS system (`src/layer1/mod.rs` or relevant system)
- Layer 1 core modules

## 3. RED Phase: Tests First
```rust
// specs/1195-rapid-decompression-projectiles.md - doctest for TDD
// These tests should fail until implementation is complete.

#[test]
fn test_unsecured_items_become_projectiles_on_decompression() {
    let mut world = World::new();
    let room = world.spawn(Room { pressure: 1.0 }).id();
    let wrench = world.spawn((ItemType::Wrench, InRoom(room))).id();

    // Act: Run the atmospheric/physics system
    world.send_event(DecompressionEvent { room, target_pressure: 0.0 });
    run_decompression_physics_system(&mut world);

    // Assert: Verify items receive a velocity component directed towards the breach
    let velocity = world.get::<Velocity>(wrench).unwrap();
    assert!(velocity.magnitude() > 10.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal code to turn tests green.
// Define required components and resources.
// Update relevant systems to process the new data.
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Avoid tight coupling.
- **Performance**: Ensure systems are optimized.
- **Design**: Consider integration points.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Basic logic and edge cases are verified.

## 7. Technical Guidance
- Register all new systems, events, and resources in the main app.
- Check relevant integration points in `src/`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
