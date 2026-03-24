# 567: The Gravity Well Graveyard

## Overview

Living at the bottom of a cosmic sinkhole where the galaxy's trash—and treasures—inevitably fall. Settling a colony on a hyper-dense world (or near a black hole) creates a passive "Tractor Effect" on Layer 2. Passing derelicts, asteroid fragments, and occasionally active hostile ships are slowly pulled into your orbit and eventually crash onto your Layer 1 map.

## Dependencies

- Layer 2 Orbits and Gravity Well (Spec X).
- Event generation (Spec Y).


- A planetary trait `GravitySinkhole` that triggers periodic `OrbitalImpactEvent`.
- `OrbitalImpactEvent` brings either resources, derelict tech, or hostile events (e.g. crashing unexploded ordnance).
- A planetary shield or deflector building to prevent damage from the impacts.

- Completely random impacts; they should be scaled to local Layer 3 wars.

## RED Phase: Tests First

```rust
// tests/integration/gravity_well_graveyard.rs

#[test]
fn test_gravity_sinkhole_trait_triggers_impact_events() {
    let mut app = setup_test_app_with_trait(PlanetaryTrait::GravitySinkhole);

    // Simulate time passing
    app.update_n_ticks(100);

    // Assert an impact event was queued
    let impacts = app.world().resource::<Events<OrbitalImpactEvent>>();
    assert!(!impacts.is_empty());
}

#[test]
fn test_impact_event_damages_colony_without_shield() {
    let mut app = setup_test_app_with_trait(PlanetaryTrait::GravitySinkhole);
    let building = spawn_test_building(&mut app);

    // Trigger impact directly on building
    trigger_impact_on(&mut app, building);
    app.update();

    // Assert building is damaged
    let health = app.world().get::<Health>(building).unwrap();
    assert!(health.current < health.max);
}

#[test]
fn test_impact_event_blocked_by_planetary_shield() {
    let mut app = setup_test_app_with_trait(PlanetaryTrait::GravitySinkhole);
    let shield = spawn_planetary_shield(&mut app);
    let building = spawn_test_building(&mut app);

    // Trigger impact directly on building
    trigger_impact_on(&mut app, building);
    app.update();

    // Assert building is NOT damaged
    let health = app.world().get::<Health>(building).unwrap();
    assert_eq!(health.current, health.max);
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer2/events.rs
pub struct OrbitalImpactEvent {
    pub location: Vec2,
    pub impact_type: ImpactType,
}

// src/layer2/systems/impact_systems.rs
pub fn trigger_gravity_sinkhole_impacts(
    query: Query<&PlanetaryTrait>,
    mut events: EventWriter<OrbitalImpactEvent>,
    time: Res<Time>,
    mut timer: ResMut<ImpactTimer>,
) {
    if timer.tick(time.delta()).just_finished() {
        for traits in query.iter() {
            if traits.contains(&PlanetaryTrait::GravitySinkhole) {
                events.send(OrbitalImpactEvent {
                    location: random_location(),
                    impact_type: ImpactType::Derelict,
                });
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Refactoring:** The impact timer should scale based on Layer 3 war activity instead of a static timer.
- **Code Smells:** Avoid global static timers; attach `ImpactTimer` to the planet entity.
- **Performance:** Optimize random location generation to avoid placing it inside solid rock.

## Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Gravity sinkhole planets experience regular impacts.
- [ ] Impacts can be mitigated by shields.

## Technical Guidance

### Components
```rust
#[derive(Component)]
pub struct GravitySinkhole;

pub enum ImpactType {
    ResourceDebris,
    DerelictTech,
    HostileOrdnance,
}
```

### Systems
```rust
pub fn resolve_orbital_impacts_system(...) {}
pub fn scale_impact_frequency_with_layer3_system(...) {}
```

### Integration Points
Connect `OrbitalImpactEvent` to the Layer 1 event system so that crashing objects instantiate actual resources or cause damage in the game world.

## Questions

*Builder: add questions here if spec is unclear. Architect will address.*
