# 229: Station Keeping

## 1. Overview

**Gravity is a relentless creditor.**

In Layer 2 (System Simulation), objects in orbit are not statically pinned to the sky. They must constantly expend energy (Fuel) to correct for atmospheric drag, gravitational perturbations, and solar wind. This feature introduces **Station Keeping**, a mechanic where Orbital Stations and stationary Fleets consume a small amount of Fuel per tick.

If an entity runs out of Fuel, its **Orbital Stability** begins to decay. When Stability reaches 0%, the object's orbit collapses, crashing it onto the planet (Layer 1) or burning up in the atmosphere, spawning massive amounts of debris.

This adds a critical logistical pressure: You cannot just build a station and forget it. You must maintain a fuel supply chain.

## 2. Dependencies

- [x] `152` Orbital Stations (Defines `Station` entities)
- [x] `099` Fleet Movement (Defines `Fleet` and `InOrbit`)
- [x] `104` Fuel Industry (Defines `Fuel` resource)
- [x] `184` Orbital Debris (Defines debris mechanics)

## 3. RED Phase: Tests First

These tests verify the core mechanics of fuel consumption, stability decay, and catastrophic failure.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;

    // Mock components for testing
    #[derive(Component, Debug, Default)]
    struct StationKeeping {
        pub fuel_reserve: f32,
        pub max_fuel: f32,
        pub consumption_rate: f32,
        pub stability: f32, // 0.0 to 100.0
        pub decay_rate: f32,
    }

    #[derive(Component)]
    struct Station;

    #[derive(Component)]
    struct Fleet;

    #[derive(Event)]
    struct OrbitCrashEvent {
        pub entity: Entity,
        pub parent: Entity,
    }

    #[test]
    fn test_station_keeping_consumes_fuel() {
        // Arrange
        let mut world = World::new();
        let id = world.spawn(StationKeeping {
            fuel_reserve: 10.0,
            max_fuel: 100.0,
            consumption_rate: 1.0,
            stability: 100.0,
            decay_rate: 5.0,
        }).id();

        // Act: Run system for 1 tick
        // station_keeping_system(&mut world);

        // Assert
        let sk = world.get::<StationKeeping>(id).unwrap();
        assert_eq!(sk.fuel_reserve, 9.0);
        assert_eq!(sk.stability, 100.0);
    }

    #[test]
    fn test_stability_decays_without_fuel() {
        // Arrange
        let mut world = World::new();
        let id = world.spawn(StationKeeping {
            fuel_reserve: 0.0,
            consumption_rate: 1.0,
            stability: 100.0,
            decay_rate: 5.0,
            ..Default::default()
        }).id();

        // Act
        // station_keeping_system(&mut world);

        // Assert
        let sk = world.get::<StationKeeping>(id).unwrap();
        assert_eq!(sk.stability, 95.0);
    }

    #[test]
    fn test_refueling_stops_decay_and_recovers_stability() {
        // Arrange
        let mut world = World::new();
        let id = world.spawn(StationKeeping {
            fuel_reserve: 5.0, // Has fuel now
            consumption_rate: 1.0,
            stability: 50.0,   // Was decayed
            decay_rate: 5.0,
            ..Default::default()
        }).id();

        // Act
        // station_keeping_system(&mut world);

        // Assert
        let sk = world.get::<StationKeeping>(id).unwrap();
        assert_eq!(sk.fuel_reserve, 4.0);
        assert!(sk.stability > 50.0, "Stability should recover when fueled");
    }

    #[test]
    fn test_crash_at_zero_stability() {
        // Arrange
        let mut world = World::new();
        let planet = world.spawn_empty().id();
        let station = world.spawn((
            Station,
            StationKeeping {
                fuel_reserve: 0.0,
                stability: 0.0, // Critical
                decay_rate: 5.0,
                ..Default::default()
            },
            InOrbit { parent: planet },
        )).id();

        // Register event
        // world.add_event::<OrbitCrashEvent>();

        // Act
        // station_keeping_system(&mut world);

        // Assert
        // Check event fired
        // Check entity despawned or marked for destruction
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### Components

**`StationKeeping`**
Tracks the fuel status and structural integrity of the orbit.
- `fuel_reserve`: `f32` (Current fuel stored)
- `max_fuel`: `f32` (Capacity)
- `consumption_rate`: `f32` (Fuel consumed per tick, e.g., 0.1)
- `stability`: `f32` (Current stability %, starts at 100.0)
- `decay_rate`: `f32` (Stability loss per tick when empty, e.g., 0.5)

### Systems

**`station_keeping_system`**
Runs on `FixedUpdate` (or simulation tick).
Iterates all entities with `StationKeeping`.

Logic:
1. **Has Fuel?**
   - If `fuel_reserve >= consumption_rate`:
     - Deduct `consumption_rate`.
     - Increase `stability` by `decay_rate` (recover orbit) up to 100.0.
   - Else:
     - Fuel is empty (set to 0).
     - Decrease `stability` by `decay_rate`.

2. **Check Failure**
   - If `stability <= 0.0`:
     - Spawn `OrbitCrashEvent { entity, parent }`.
     - Despawn entity (or mark `DespawnRecursive`).
     - Spawn `OrbitalDebris` at location (Spec 184).

**Integration Points**
- **Fleets**: `Fleet` entities should gain `StationKeeping` component when `InOrbit`. When `InTransit`, consumption might differ (handled by `117` Fuel Consumption), but `StationKeeping` is specifically for holding orbit.
- **Stations**: All `Station` entities spawned by `build_station_system` (Spec 152) must initialize with `StationKeeping`.

## 5. REFACTOR Phase: Quality & Design

- **Fuel Transfer**: Ensure the `Trade` or `Logistics` systems can refill `StationKeeping.fuel_reserve`.
- **UI**: Display "Orbit Stability" and "Fuel Time Remaining" in the Station Inspector.
- **Scaling**: Larger stations (Shipyards) should consume more fuel than Outposts.
- **Grace Period**: Maybe stability shouldn't decay immediately, or decay is non-linear? (Keep linear for MVP).
- **Notifications**: Warn player when fuel < 10% or Stability < 50%.

## 6. Acceptance Criteria

- [ ] `StationKeeping` component defined.
- [ ] `station_keeping_system` implements consumption/decay logic.
- [ ] Stations spawn with the component initialized.
- [ ] Empty fuel leads to stability loss.
- [ ] Zero stability triggers destruction and event.
- [ ] Tests pass.

## 7. Technical Guidance

- Use `bevy_ecs` events for the crash to decouple the destruction logic from the simulation loop.
- Register the system in `SimulationSystemSet` in `src/simulation.rs`.
- Update `build_station_system` in `src/layer2/station.rs` to add the component.
- Remember `OrbitalDebris` from Spec 184 - the crash should generate a *lot* of it.

## 8. Questions

- Should Fleets consume fuel while in orbit?
  - *Yes, "Station Keeping" applies to anything maintaining a specific geostationary or parking orbit.*
- How do we refuel?
  - *For MVP, assume manual transfer or abstract "docking" logic. Future spec will handle automated tankers.*
