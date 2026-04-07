# Station Keeping

## 1. Overview
**Layer:** 2
**Fantasy:** Gravity is a relentless creditor.
**Mechanic:** Orbital Stations and parked Fleets constantly consume minute amounts of "Fuel" to maintain orbit. If Fuel runs out, the orbit decays. The object eventually crashes onto the Layer 1 map, destroying itself and anything it hits.
**Emergence:** You forget to authorize a fuel shipment to the Starbase. It turns into a meteor that wipes out your industrial district.

## 2. Dependencies
- Base ECS system
- `Orbit` or `Fleet` components with resource storage
- Layer 1 Map interactions (for crash landing/destruction)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_orbital_station_consumes_fuel() {
        let mut app = App::new();
        app.add_systems(Update, station_keeping_system);

        let entity = app.world_mut().spawn((
            OrbitalStation,
            FuelStorage { current: 10.0, capacity: 100.0 },
            OrbitDecay { rate: 1.0, current_decay: 0.0, max_decay: 100.0 },
        )).id();

        app.update();

        let fuel = app.world().get::<FuelStorage>(entity).unwrap();
        assert!(fuel.current < 10.0, "Fuel should be consumed to maintain orbit");

        let decay = app.world().get::<OrbitDecay>(entity).unwrap();
        assert_eq!(decay.current_decay, 0.0, "Orbit should not decay while fuel is available");
    }

    #[test]
    fn test_orbit_decays_when_fuel_empty() {
        let mut app = App::new();
        app.add_systems(Update, station_keeping_system);

        let entity = app.world_mut().spawn((
            OrbitalStation,
            FuelStorage { current: 0.0, capacity: 100.0 },
            OrbitDecay { rate: 5.0, current_decay: 0.0, max_decay: 100.0 },
        )).id();

        app.update();

        let decay = app.world().get::<OrbitDecay>(entity).unwrap();
        assert!(decay.current_decay > 0.0, "Orbit should decay when out of fuel");
    }

    #[test]
    fn test_orbit_crash_when_decay_maxed() {
        let mut app = App::new();
        app.add_event::<OrbitalCrashEvent>();
        app.add_systems(Update, station_keeping_system);

        let entity = app.world_mut().spawn((
            OrbitalStation,
            FuelStorage { current: 0.0, capacity: 100.0 },
            OrbitDecay { rate: 5.0, current_decay: 98.0, max_decay: 100.0 }, // Near max
        )).id();

        app.update();

        let events = app.world().resource::<Events<OrbitalCrashEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.len(&events), 1, "Should emit a crash event when decay reaches max");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct OrbitalStation;

#[derive(Component)]
pub struct FuelStorage {
    pub current: f32,
    pub capacity: f32,
}

#[derive(Component)]
pub struct OrbitDecay {
    pub rate: f32,
    pub current_decay: f32,
    pub max_decay: f32,
}

#[derive(Event)]
pub struct OrbitalCrashEvent {
    pub entity: Entity,
}

pub fn station_keeping_system(
    mut query: Query<(Entity, &mut FuelStorage, &mut OrbitDecay)>,
    mut crash_events: EventWriter<OrbitalCrashEvent>,
) {
    for (entity, mut fuel, mut decay) in query.iter_mut() {
        let fuel_consumption_rate = 1.0;

        if fuel.current >= fuel_consumption_rate {
            fuel.current -= fuel_consumption_rate;
            decay.current_decay = 0.0; // Stabilize orbit
        } else {
            fuel.current = 0.0;
            decay.current_decay += decay.rate;

            if decay.current_decay >= decay.max_decay {
                crash_events.send(OrbitalCrashEvent { entity });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate fuel consumption based on mass/size of the station or fleet.
- Add warnings or notifications to the UI when fuel reaches critical levels.
- Integrate with `Time` so `fuel.current` and `decay` operate on `delta_seconds()`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Stations consume fuel, orbits decay when empty, crash event fires at max decay.

## 7. Technical Guidance
- Integration point: Layer 1 should listen for `OrbitalCrashEvent` to trigger explosions and destruction on the map.
- Consider what resources count as "Fuel".

## 8. Questions
*Builder: add questions here if spec is unclear.*
