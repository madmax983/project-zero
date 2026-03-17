# Specification 493: Ghost Town Phobia

## 1. Overview
Buildings that have been unpowered or unstaffed for a long duration, or areas where significant numbers of Pops died, generate a localized "Dread" aura. Pops pathfinding through these abandoned sectors suffer an immediate, sharp spike in Stress and a temporary decrease in movement speed as they hesitate in the dark.

## 2. Dependencies
- Needs System (Stress)
- Memory System (Death events)
- Designation/Zoning System (Pathfinding)
- Grid System (Auras/Fields)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_abandoned_building_generates_dread_aura() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn an unpowered building
    let building_entity = app.world_mut().spawn((
        Building::new(BuildingType::Factory),
        PowerStatus::Unpowered,
        TimeUnpowered(Duration::from_days(30)),
    )).id();

    // Act: Run system
    app.update();

    // Assert: The building should now have a DreadAura component
    assert!(app.world().get::<DreadAura>(building_entity).is_some());
}

#[test]
fn test_pop_pathing_through_dread_gains_stress_and_slows_down() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(Layer1Plugin);

    // Arrange: Spawn a dread aura tile and a pop moving onto it
    let pop_entity = app.world_mut().spawn((
        PopBundle::default(),
        Position { x: 5, y: 5 },
        BaseSpeed(1.0),
        Stress(0.0),
    )).id();

    app.world_mut().spawn((
        GridPosition { x: 5, y: 5 },
        DreadAura { intensity: 10.0 },
    ));

    // Act: Run systems to apply aura effects
    app.update();

    // Assert: Stress should increase, and a speed debuff should be applied
    let stress = app.world().get::<Stress>(pop_entity).unwrap();
    let speed_modifier = app.world().get::<SpeedModifier>(pop_entity).unwrap();

    assert!(stress.0 > 0.0);
    assert!(speed_modifier.multiplier < 1.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// Add `DreadAura` component to buildings unpowered for > threshold.
// Query Pops intersecting `DreadAura` tiles, apply `Stress` increment and `SpeedModifier`.
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the dread aura calculation (use a dirty flag for building state changes).
- Consider making the dread aura dissipate slowly once a building is re-occupied.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] Pops traversing abandoned areas experience a noticeable stress increase and speed penalty.

## 7. Technical Guidance
- Implement a `DreadAura` component for buildings and tiles.
- The pathfinding logic should factor in the `DreadAura` to avoid it if a reasonably fast alternative path exists, or suffer the penalties if it must traverse.
- The penalty to movement speed should be temporary (`RecentAction` or similar status effect).

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
