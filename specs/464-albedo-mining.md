# 464 - Albedo Mining

## 1. Overview
Changing the climate of your world not through pollution, but by literally stealing its shade from orbit. A planet with an "Ice Ring" provides incredible, easy water extraction on Layer 2. However, the ring naturally reflects sunlight. As you mine the ring and thin it out, the "Albedo" of the planet drops. The Layer 1 colony experiences a permanent, slowly rising global temperature increase.

## 2. Dependencies
- Layer 1 Temperature System (`src/layer1/weather/temperature.rs` or similar)
- Layer 2 System Resources and Logistics (Extracting resources)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    #[derive(Component)]
    struct IceRing {
        total_mass: f32,
        current_mass: f32,
    }

    #[derive(Component)]
    struct Albedo {
        value: f32,
    }

    #[derive(Event)]
    struct MineIceRingEvent {
        planet: Entity,
        amount: f32,
    }

    #[derive(Resource, Default)]
    struct GlobalTemperature {
        base: f32,
        current: f32,
    }

    fn mine_ice_ring_system(
        mut events: EventReader<MineIceRingEvent>,
        mut query: Query<(&mut IceRing, &mut Albedo)>,
    ) {
        for event in events.read() {
            if let Ok((mut ice_ring, mut albedo)) = query.get_mut(event.planet) {
                ice_ring.current_mass = (ice_ring.current_mass - event.amount).max(0.0);

                let ratio = ice_ring.current_mass / ice_ring.total_mass;
                albedo.value = 0.2 + (0.6 * ratio);
            }
        }
    }

    fn sync_albedo_temperature_system(
        query: Query<&Albedo>,
        mut global_temp: ResMut<GlobalTemperature>,
    ) {
        for albedo in query.iter() {
            global_temp.current = global_temp.base + ((1.0 - albedo.value) * 50.0);
        }
    }

    #[test]
    fn test_ice_ring_mining_reduces_albedo() {
        let mut world = World::new();

        let planet_entity = world.spawn((
            IceRing { total_mass: 1000.0, current_mass: 1000.0 },
            Albedo { value: 0.8 },
        )).id();

        let mut schedule = Schedule::default();
        world.insert_resource(Events::<MineIceRingEvent>::default());
        schedule.add_systems(mine_ice_ring_system);

        world.send_event(MineIceRingEvent {
            planet: planet_entity,
            amount: 100.0,
        });

        schedule.run(&mut world);

        let albedo = world.get::<Albedo>(planet_entity).unwrap();
        assert!(albedo.value < 0.8);
    }

    #[test]
    fn test_reduced_albedo_increases_global_temperature() {
        let mut world = World::new();

        world.insert_resource(GlobalTemperature { base: 20.0, current: 20.0 });

        let planet_entity = world.spawn(Albedo { value: 0.8 }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(sync_albedo_temperature_system);

        schedule.run(&mut world);

        let initial_temp = world.resource::<GlobalTemperature>().current;

        world.get_mut::<Albedo>(planet_entity).unwrap().value = 0.2;

        schedule.run(&mut world);

        let final_temp = world.resource::<GlobalTemperature>().current;

        assert!(final_temp > initial_temp);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct Planet {
    pub name: String,
}

#[derive(Component)]
pub struct IceRing {
    pub total_mass: f32,
    pub current_mass: f32,
}

#[derive(Component)]
pub struct Albedo {
    pub value: f32,
}

#[derive(Event)]
pub struct MineIceRingEvent {
    pub planet: Entity,
    pub amount: f32,
}

#[derive(Resource)]
pub struct GlobalTemperature {
    pub base: f32,
    pub current: f32,
}

pub fn mine_ice_ring_system(
    mut events: EventReader<MineIceRingEvent>,
    mut query: Query<(&mut IceRing, &mut Albedo)>,
) {
    for event in events.read() {
        if let Ok((mut ice_ring, mut albedo)) = query.get_mut(event.planet) {
            ice_ring.current_mass = (ice_ring.current_mass - event.amount).max(0.0);

            // Simple proportional albedo reduction
            let ratio = ice_ring.current_mass / ice_ring.total_mass;
            albedo.value = 0.2 + (0.6 * ratio); // Scales from 0.2 (no ring) to 0.8 (full ring)
        }
    }
}

pub fn sync_albedo_temperature_system(
    query: Query<&Albedo, With<Planet>>,
    mut global_temp: ResMut<GlobalTemperature>,
) {
    for albedo in query.iter() {
        // Lower albedo means higher temperature absorption
        // Example: Base temp + (1.0 - albedo) * 50.0
        global_temp.current = global_temp.base + ((1.0 - albedo.value) * 50.0);
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration:** The `GlobalTemperature` resource needs to correctly sync with Layer 1's `TemperatureGrid` so that individual tiles feel the heat.
- **Resource Output:** The `MineIceRingEvent` should produce `Water` or `Ice` resources on Layer 1 or Layer 2 stockpiles to make it a viable, high-yield mining path.
- **Visuals:** Add visual feedback for Ratatui where the ring thinning or the planetary glow increases based on albedo loss.

## 6. Acceptance Criteria (Testable!)
- [ ] `IceRing` mass reduction correctly lowers the `Albedo` component on the planet.
- [ ] Reduced `Albedo` results in a higher `GlobalTemperature`.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- Ensure that the calculation for `Albedo` reduction properly balances the heat increase over the expected length of a playthrough to avoid instant catastrophic temperature spikes.
- A non-linear reduction model may be more realistic and mechanically interesting.

## 8. Questions
*Builder: add questions here if spec is unclear.*
