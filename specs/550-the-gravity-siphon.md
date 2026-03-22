# Spec 550: The Gravity Siphon

## 1. Overview
The colony uncovers an ancient, experimental "Micro-Singularity Generator" (Layer 1). It provides infinite, clean energy, solving all power needs. However, running the generator increases the gravitational mass of the colony locally. Over time, this massive localized gravity anomaly disrupts the orbit of the planet on Layer 2, pulling dangerous asteroid belts closer, warping trade ship navigation, and eventually pulling a planet-killer asteroid out of orbit onto a collision course with the settlement.

## 2. Dependencies
- `042` Energy System
- `094` System View Architecture (Layer 2)
- `184` Orbital Debris (for asteroids)
- Event Bridge for L1 -> L2 communication.

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::energy::EnergyGrid;
    use crate::layer2::system_map::{Planet, AsteroidField};

    #[test]
    fn test_singularity_generator_provides_infinite_energy() {
        let mut app = App::new();
        app.add_systems(Update, process_singularity_energy_system);

        let mut grid = EnergyGrid { produced: 0.0, ..Default::default() };
        app.world_mut().insert_resource(grid);

        app.world_mut().spawn(SingularityGenerator { active: true, mass_accumulated: 0.0 });

        app.update();

        let updated_grid = app.world().resource::<EnergyGrid>();
        assert!(updated_grid.produced >= 999999.0, "Singularity should provide effectively infinite power");
    }

    #[test]
    fn test_active_generator_increases_gravitational_mass() {
        let mut app = App::new();
        app.add_systems(Update, process_singularity_mass_accumulation_system);
        app.insert_resource(Time::default());

        let generator = app.world_mut().spawn(SingularityGenerator { active: true, mass_accumulated: 0.0 }).id();

        // Advance time
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(1.0));
        app.update();

        let gen_state = app.world().get::<SingularityGenerator>(generator).unwrap();
        assert!(gen_state.mass_accumulated > 0.0, "Active generator must accumulate mass");
    }

    #[test]
    fn test_critical_mass_triggers_layer_2_orbital_decay() {
        let mut app = App::new();
        app.add_event::<OrbitalDecayEvent>();
        app.add_systems(Update, trigger_orbital_decay_system);

        app.world_mut().spawn(SingularityGenerator { active: true, mass_accumulated: 10000.0 }); // Critical mass

        app.update();

        let decay_events = app.world().resource::<Events<OrbitalDecayEvent>>();
        let mut reader = decay_events.get_reader();
        assert!(reader.read(decay_events).len() > 0, "Critical mass should trigger an orbital decay event");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;
use crate::layer1::energy::EnergyGrid;

#[derive(Component)]
pub struct SingularityGenerator {
    pub active: bool,
    pub mass_accumulated: f32,
}

#[derive(Event)]
pub struct OrbitalDecayEvent {
    pub anomaly_strength: f32,
}

pub fn process_singularity_energy_system(
    mut grid: ResMut<EnergyGrid>,
    query: Query<&SingularityGenerator>,
) {
    for generator in query.iter() {
        if generator.active {
            grid.produced += 1_000_000.0; // Infinite power
        }
    }
}

pub fn process_singularity_mass_accumulation_system(
    time: Res<Time>,
    mut query: Query<&mut SingularityGenerator>,
) {
    let dt = time.delta_secs();
    for mut generator in query.iter_mut() {
        if generator.active {
            generator.mass_accumulated += 10.0 * dt; // Accumulate mass
        }
    }
}

pub fn trigger_orbital_decay_system(
    query: Query<&SingularityGenerator>,
    mut decay_events: EventWriter<OrbitalDecayEvent>,
) {
    for generator in query.iter() {
        if generator.mass_accumulated >= 10000.0 { // Critical threshold
            decay_events.send(OrbitalDecayEvent { anomaly_strength: generator.mass_accumulated });
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Create a `SingularityConfig` resource to manage the mass accumulation rate, energy output, and the critical thresholds.
- `OrbitalDecayEvent` needs to be bridged to Layer 2 to physically shift the `AsteroidField` nodes closer to the `Planet` node, altering trade routes and increasing the probability of `OrbitalDebris` events.
- Implement UI warnings on Layer 1 when the gravitational mass begins altering local physics (e.g., movement speed penalties around the generator).

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for new code.
- [ ] An active `SingularityGenerator` provides massive energy to the `EnergyGrid`.
- [ ] An active `SingularityGenerator` constantly accumulates `mass_accumulated`.
- [ ] When `mass_accumulated` crosses a threshold, an `OrbitalDecayEvent` is emitted.

## 7. Technical Guidance
- The `SingularityGenerator` should be a unique, unbuildable `Artifact` discovered via `041` Field Science or deep mining.
- The player must have the option to deactivate it, halting the energy production and mass accumulation, but perhaps the accumulated mass never fully dissipates, serving as permanent structural debt.
- The ultimate consequence on Layer 2 should be the spawning of a `PlanetKillerAsteroid` entity homing in on the colony's planet node.

## 8. Questions
*Builder: add questions here if spec is unclear.*
