# Specification 496: Sub-Orbital Piracy

## 1. Overview
Pirates deploy stealthy "Sub-Orbital Skiffs" that hover just above Layer 1, out of range of ground turrets. They cast grappling lines down to "fish" for high-value cargo from outdoor stockpiles or even snatch unwary Pops.

## 2. Dependencies
- Layer 1 Map & Entities
- Layer 2 Pirate Encounters
- Layer 1/2 Event Bridge
- Resource/Storage System (Outdoor vs Indoor)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_sub_orbital_skiff_steals_outdoor_cargo() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn an outdoor stockpile with 50 FusionCores
    let stockpile = app.world_mut().spawn((
        Stockpile { capacity: 100 },
        Inventory { items: vec![Item::new(ItemType::FusionCore, 50)] },
        Outdoor, // No roof component
    )).id();

    // Act: Trigger Sub-Orbital raid on the stockpile
    app.world_mut().send_event(SubOrbitalRaidEvent {
        target: stockpile,
        thief_capacity: 10,
    });
    app.update();

    // Assert: 10 FusionCores were stolen
    let inv = app.world().get::<Inventory>(stockpile).unwrap();
    let cores = inv.get_count(ItemType::FusionCore);
    assert_eq!(cores, 40);
}

#[test]
fn test_indoor_cargo_is_safe_from_skiffs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn an indoor stockpile with 50 FusionCores
    let stockpile = app.world_mut().spawn((
        Stockpile { capacity: 100 },
        Inventory { items: vec![Item::new(ItemType::FusionCore, 50)] },
        Roof, // Indoor
    )).id();

    // Act: Trigger raid
    app.world_mut().send_event(SubOrbitalRaidEvent {
        target: stockpile,
        thief_capacity: 10,
    });
    app.update();

    // Assert: No cargo was stolen
    let inv = app.world().get::<Inventory>(stockpile).unwrap();
    let cores = inv.get_count(ItemType::FusionCore);
    assert_eq!(cores, 50); // Unchanged
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// Process `SubOrbitalRaidEvent`. If `target` lacks `Roof` component, deduct `thief_capacity` from inventory.
// If `target` has `Roof`, ignore event.
```

## 5. REFACTOR Phase: Quality & Design
- Connect the Skiff event to L2 pirate presence.
- Ensure the event accurately discriminates between roofed/unroofed tiles.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Skiffs can steal from outdoor stockpiles but fail on indoor ones.

## 7. Technical Guidance
- Introduce a `SubOrbitalRaidEvent` spawned randomly when pirate activity is high in the sector.
- The system processes the event by selecting a valuable `Item` or `ResourcePile` on a tile without a `Roof`.
- The skiff simply "deletes" the item/resource after a short delay (visualizing the grappling hook).
- Future additions can include capturing unwary Pops in a similar manner.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
