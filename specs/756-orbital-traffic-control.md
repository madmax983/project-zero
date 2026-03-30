# Spec 756: Orbital Traffic Control

## 1. Overview
**Layer:** 2
**Fantasy:** The sky is crowded.
**Mechanic:** As trade/mining ships increase, collision risk in orbit rises. "Traffic Control" stations reduce risk but cap throughput. Collisions drop debris/burning wreckage on Layer 1.
**Emergence:** You maximize trade for a boom economy, ignoring safety. Two freighters collide, raining burning electronics on your primary hospital.
**Tension:** Economic Throughput vs. Safety.

## 2. Dependencies
- Layer 1 `GridPosition` and `TerrainGrid` (for debris impact).
- Layer 2 `Fleet` and `Node` (for traffic).
- Event system (`AddChronicleEvent` for significant collisions).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::chronicle::AddChronicleEvent;

    #[test]
    fn test_orbital_traffic_collision_chance_increases_with_fleets() {
        // Arrange: Setup App with Layer 2 Orbit tracking
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.add_event::<OrbitalCollisionEvent>();

        let orbit_entity = app.world_mut().spawn(Orbit { traffic_count: 5 }).id();

        // Act: Run traffic evaluation system
        app.add_systems(Update, evaluate_orbital_traffic_system);
        app.update();

        // Assert: High traffic should probabilistically spawn a collision event
        // We can force RNG seed in real test, but here we expect at least an attempt
        // to calculate collision risk.
        let events = app.world().resource::<Events<OrbitalCollisionEvent>>();
        assert!(!events.is_empty(), "Collision event should be generated under high traffic");
    }

    #[test]
    fn test_traffic_control_station_reduces_collision_risk() {
        let mut app = App::new();
        app.add_event::<AddChronicleEvent>();
        app.add_event::<OrbitalCollisionEvent>();

        // Spawn a traffic control station to mitigate risk
        app.world_mut().spawn((
            TrafficControlStation { efficiency: 0.8 },
            Orbit { traffic_count: 5 }
        ));

        app.add_systems(Update, evaluate_orbital_traffic_system);
        app.update();

        let events = app.world().resource::<Events<OrbitalCollisionEvent>>();
        assert!(events.is_empty(), "Traffic control should have prevented the collision");
    }

    #[test]
    fn test_collision_spawns_debris_on_layer_1() {
        let mut app = App::new();
        app.add_event::<OrbitalCollisionEvent>();
        app.add_event::<SpawnDebrisEvent>();

        // Simulate a collision
        app.world_mut().resource_mut::<Events<OrbitalCollisionEvent>>()
            .send(OrbitalCollisionEvent { intensity: 10 });

        app.add_systems(Update, process_orbital_collisions_system);
        app.update();

        let spawn_events = app.world().resource::<Events<SpawnDebrisEvent>>();
        assert!(!spawn_events.is_empty(), "Debris should be spawned on the surface after a collision");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct Orbit {
    pub traffic_count: u32,
}

#[derive(Component)]
pub struct TrafficControlStation {
    pub efficiency: f32, // 0.0 to 1.0
}

#[derive(Event)]
pub struct OrbitalCollisionEvent {
    pub intensity: u32,
}

#[derive(Event)]
pub struct SpawnDebrisEvent {
    pub amount: u32,
}

pub fn evaluate_orbital_traffic_system(
    query: Query<(&Orbit, Option<&TrafficControlStation>)>,
    mut collision_writer: EventWriter<OrbitalCollisionEvent>,
) {
    let mut rng = rand::thread_rng();

    for (orbit, control_station) in query.iter() {
        let mut risk_factor = orbit.traffic_count as f32 * 0.1;

        if let Some(station) = control_station {
            risk_factor *= 1.0 - station.efficiency;
        }

        if risk_factor > 0.5 && rng.gen_bool(risk_factor.clamp(0.0, 1.0) as f64) {
            collision_writer.send(OrbitalCollisionEvent {
                intensity: orbit.traffic_count,
            });
        }
    }
}

pub fn process_orbital_collisions_system(
    mut collision_reader: EventReader<OrbitalCollisionEvent>,
    mut debris_writer: EventWriter<SpawnDebrisEvent>,
) {
    for event in collision_reader.read() {
        debris_writer.send(SpawnDebrisEvent {
            amount: event.intensity * 2,
        });
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Constants**: Extract the `0.1` traffic-to-risk multiplier and `0.5` threshold into configurable constants or global `GameRules` resources.
- **RNG**: Use a seeded RNG resource from Bevy rather than `rand::thread_rng()` directly to ensure deterministic simulation for tests and gameplay.
- **Debris placement**: The `SpawnDebrisEvent` bridge will need to interact with the Layer 1 `TerrainGrid` and `GridPosition` to accurately place debris without overwriting critical infrastructure.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >= 85% for new code
- [ ] `OrbitalCollisionEvent` properly triggers `AddChronicleEvent` for lore integration.

## 7. Technical Guidance
- Register the systems in `Layer2SystemSet::Economy` or a dedicated orbital traffic set.
- A bridge system (e.g., in `src/layer1/integration.rs`) should listen for `SpawnDebrisEvent` and handle the Layer 1 physical placement.

## 8. Questions
*Builder: add questions here if spec is unclear.*
