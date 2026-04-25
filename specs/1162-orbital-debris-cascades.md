# 1162: Orbital Debris Cascades

## Overview
Every ship destroyed or orbital station demolished generates "Debris Clouds" in Layer 2 orbit. High debris density increases the chance that incoming ships (traders, refugees, or military) are damaged or destroyed upon entering the system, generating even *more* debris (Kessler Syndrome). Extremely dense debris clouds block sunlight to the Layer 1 surface, dropping temperature and destroying solar power generation. This creates tension between fighting enemies close to your worlds for defense advantages versus risking the long-term doom of the planet due to debris.

## Dependencies
- Layer 2 Fleet Combat & Destruction
- Layer 1 Atmosphere/Temperature/Power systems
- Layer 2 Orbit/Movement Pathfinding

## RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_ship_destruction_creates_debris() {
    // Arrange: Spawn a ship and a planet it's orbiting
    // Act: Destroy the ship
    // Assert: A DebrisCloud component is spawned in the planet's orbit with mass proportional to the ship
}

#[test]
fn test_debris_damages_incoming_ships() {
    // Arrange: A high-density DebrisCloud exists. A ship moves through the orbit.
    // Act: Advance simulation
    // Assert: The ship takes damage and potentially generates more debris
}

#[test]
fn test_dense_debris_blocks_sunlight() {
    // Arrange: A DebrisCloud reaches extreme density
    // Act: Advance simulation to trigger Layer 1 effects
    // Assert: The planet's surface temperature and solar power efficiency drop
}
```

## GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN

pub fn generate_debris_on_destruction(
    mut commands: Commands,
    mut ship_destroyed_events: EventReader<ShipDestroyedEvent>,
    mut existing_debris: Query<(Entity, &Orbiting, &mut DebrisCloud)>,
) {
    for event in ship_destroyed_events.read() {
        let mut added = false;
        // Find existing debris cloud in same orbit to add to
        for (_entity, orbiting, mut cloud) in existing_debris.iter_mut() {
            if orbiting.target == event.planet_entity {
                cloud.mass += event.ship_mass;
                added = true;
                break;
            }
        }

        // Spawn new cloud if none exists
        if !added {
            commands.spawn((
                DebrisCloud { mass: event.ship_mass },
                Orbiting { target: event.planet_entity },
            ));
        }
    }
}

pub fn debris_collision_system(
    mut commands: Commands,
    debris_clouds: Query<(&DebrisCloud, &Orbiting)>,
    mut ships: Query<(Entity, &Orbiting, &mut Hull), With<Ship>>,
) {
    for (cloud, cloud_orbit) in debris_clouds.iter() {
        if cloud.mass > 1000.0 { // threshold for collisions
            for (ship_entity, ship_orbit, mut hull) in ships.iter_mut() {
                if cloud_orbit.target == ship_orbit.target {
                    // Simple collision logic
                    hull.integrity -= cloud.mass * 0.01;
                    if hull.integrity <= 0.0 {
                        // Normally this would trigger ShipDestroyedEvent, simplified here
                        commands.entity(ship_entity).insert(Destroyed);
                    }
                }
            }
        }
    }
}

pub fn debris_shadow_effect_system(
    debris_clouds: Query<(&DebrisCloud, &Orbiting)>,
    mut planets: Query<(Entity, &mut Temperature, &mut SolarEfficiency)>,
) {
    for (cloud, orbiting) in debris_clouds.iter() {
        if cloud.mass > 10000.0 { // Extreme density
            if let Ok((_entity, mut temp, mut solar)) = planets.get_mut(orbiting.target) {
                temp.value -= 5.0; // Drop temperature
                solar.efficiency *= 0.5; // Halve solar efficiency
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design
- **Code Smell:** Hardcoded values like `1000.0` and `10000.0` for density thresholds. Use a defined resource or constants.
- **Optimization:** Use spatial partitioning or specific event triggers for collisions instead of checking all ships against all debris clouds every frame.
- **Integration:** The `ShipDestroyedEvent` needs to properly capture ship mass. Sunlight blocking should tie into the existing weather/climate systems more organically.
- **Refactoring:** Debris clouds should probably decay slowly over time naturally, or be clearable by specific "Sweeper" ships.

## Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code in `src/layer2/debris.rs`
- [ ] Extreme debris creates a noticeable, logged effect on the Layer 1 colony below.

## Technical Guidance
- **Components:** `DebrisCloud { pub mass: f32 }`.
- **System Placement:** `generate_debris_on_destruction` and `debris_collision_system` run in the Layer 2 schedule. `debris_shadow_effect_system` runs in the cross-layer update.
- **Events:** Ensure `ShipDestroyedEvent` is emitted by the combat systems.

## Questions
*Builder: add questions here if spec is unclear. Architect will address.*
