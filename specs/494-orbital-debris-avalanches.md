# Specification 494: Orbital Debris Avalanches

## 1. Overview
Intense fleet combat or the destruction of massive orbital structures in Layer 2 orbit generates an unstable, hyper-dense debris cloud. This cloud periodically causes massive "Avalanches" of flaming shrapnel to pour down onto specific Layer 1 sectors, instantly destroying anything on the surface.

## 2. Dependencies
- Layer 2 Fleet Combat / Orbital Structures
- Layer 1 Map Destruction System
- Layer 1/2 Event Bridge System

## 3. RED Phase: Tests First
```rust
#[test]
fn test_orbital_combat_generates_debris_cloud() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer2Plugin);

    // Arrange: Simulate ship destruction on L2
    app.world_mut().send_event(ShipDestroyedEvent {
        location: SystemLocation::Orbit(PlanetId(1)),
        mass: 5000.0,
    });

    // Act
    app.update();

    // Assert: Debris cloud should exist in orbit
    let debris_query = app.world_mut().query::<&DebrisCloud>().iter(app.world()).next();
    assert!(debris_query.is_some());
    assert!(debris_query.unwrap().mass >= 5000.0);
}

#[test]
fn test_debris_avalanche_destroys_layer_1_surface() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn a surface building and a subterranean building
    let surface_building = app.world_mut().spawn((
        Building::new(BuildingType::SolarPanel),
        SurfaceTile,
    )).id();

    let sub_building = app.world_mut().spawn((
        Building::new(BuildingType::Bunker),
        SubterraneanTile,
    )).id();

    // Act: Trigger avalanche
    app.world_mut().send_event(DebrisAvalancheEvent {
        target_sector: SectorId(1),
        damage: 1000.0,
    });
    app.update();

    // Assert: Surface building is destroyed, subterranean is safe
    assert!(app.world().get_entity(surface_building).is_none());
    assert!(app.world().get_entity(sub_building).is_some());
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// Accumulate debris mass from `ShipDestroyedEvent`.
// When mass > threshold, roll chance for `DebrisAvalancheEvent`.
// Delete entities with `SurfaceTile` caught in the avalanche radius.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure the bridging event between L2 debris and L1 avalanches handles probability correctly.
- Implement an alert/warning system before the avalanche hits to allow minor evacuation.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Layer 2 combat debris eventually triggers destructive events on Layer 1.

## 7. Technical Guidance
- Create a `DebrisCloud` resource/entity on L2.
- Have a system on L2 periodically roll for `AvalancheEvent`.
- Map the `AvalancheEvent` to an L1 `MeteorStrikeEvent` (or similar existing destructive system) but localized and massive.
- Subterranean structures should be explicitly immune to this event.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
