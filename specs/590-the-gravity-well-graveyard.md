# 590: The Gravity Well Graveyard

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Living at the bottom of a cosmic sinkhole where the galaxy's trash—and treasures—inevitably fall.
**Mechanic:** Settling a colony on a hyper-dense world (or near a black hole) creates a passive "Tractor Effect" on Layer 2. Passing derelicts, asteroid fragments, and occasionally active hostile ships are slowly pulled into your orbit and eventually crash onto your Layer 1 map.
**Emergence:** You enjoy a steady rain of free orbital salvage that builds your early economy. But as local Layer 3 wars escalate, the debris becomes massive warships and active unexploded ordnance. You are forced to build massive, outward-facing planetary shields just to stop the galaxy's garbage from leveling your city.
**Tension:** Free, passive resource acquisition vs. an ever-increasing risk of devastating, unguided orbital impacts.

## 2. Dependencies
- 100-orbital-mechanics
- 150-planetary-hazards

## 3. RED Phase: Tests First
```rust
#[test]
fn test_hyper_dense_planet_attracts_debris() {
    let mut app = setup_test_app();
    app.world.insert_resource(PlanetTraits { hyper_dense: true });

    let debris = app.world.spawn((OrbitalDebris, Position { x: 1000.0, y: 1000.0 })).id();
    app.update();

    let pos = app.world.get::<Position>(debris).unwrap();
    // Verify distance to planet (0,0) has decreased
    assert!(pos.x < 1000.0 && pos.y < 1000.0);
}

#[test]
fn test_debris_crash_damages_structures() {
    let mut app = setup_test_app();
    let structure = app.world.spawn((Structure, Health { current: 100.0 }, Position { x: 0.0, y: 0.0 })).id();

    // Simulate debris crash
    app.world.send_event(OrbitalCrashEvent { position: Position { x: 0.0, y: 0.0 }, damage: 50.0 });
    app.update();

    let health = app.world.get::<Health>(structure).unwrap();
    assert_eq!(health.current, 50.0);
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/orbital.rs
pub fn apply_tractor_effect_system(
    planet_traits: Res<PlanetTraits>,
    mut query: Query<&mut Position, With<OrbitalDebris>>,
) {
    if planet_traits.hyper_dense {
        for mut pos in query.iter_mut() {
            pos.x *= 0.99; // Simple pull towards origin
            pos.y *= 0.99;
        }
    }
}

// src/layer1/hazards.rs
pub fn process_orbital_crashes_system(
    mut events: EventReader<OrbitalCrashEvent>,
    mut query: Query<(&Position, &mut Health), With<Structure>>,
) {
    for event in events.iter() {
        for (pos, mut health) in query.iter_mut() {
            if pos.x == event.position.x && pos.y == event.position.y {
                health.current -= event.damage;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Implement spatial partitioning for crash impact detection instead of iterating all structures.
- Add varying debris types with different crash payloads (resources vs. damage).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Hyper-dense planets pull orbital objects closer over time.
- [ ] Crashing objects damage structures on Layer 1.

## 7. Technical Guidance
- Consider the performance impact of the tractor effect on many objects.
- Ensure shields can intercept `OrbitalCrashEvent`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
