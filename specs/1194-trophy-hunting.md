# 1194: Trophy Hunting

## 1. Overview
**Layer:** 1

**Fantasy:** Proving dominance over the alien world.

**Mechanic:** Killing unique mega-fauna yields "Trophies" (Heads, Claws) that can be mounted. They provide a "Courage" aura to nearby troops but increase "Aggression" in local fauna (revenge spawns).

**Emergence:** You mount the head of a Hive Queen in the mess hall. The soldiers love it. The Hive swarms the mess hall specifically to retrieve it.

**Tension:** Morale boost (Trophy) vs. Ecological retaliation (Aggro).

## 2. Dependencies
- Base ECS system (`src/layer1/mod.rs` or relevant system)
- Layer 1 core modules

## 3. RED Phase: Tests First
```rust
// specs/1194-trophy-hunting.md - doctest for TDD
// These tests should fail until implementation is complete.

#[test]
fn test_trophy_generation_on_megafauna_kill() {
    let mut world = World::new();
    let fauna = world.spawn((Megafauna, EntityId(1))).id();

    // Act: Run the combat/loot system
    world.send_event(EntityDeathEvent { entity: fauna, killer: Some(Player) });
    run_loot_generation_system(&mut world);

    // Assert: Verify a mountable "Trophy" item (e.g. Head/Claw) is generated in the inventory
    let inventory = world.resource::<ColonyInventory>();
    assert!(inventory.contains(&ItemType::Trophy(TrophyType::HiveQueenHead)));
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
