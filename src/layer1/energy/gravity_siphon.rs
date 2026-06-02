use crate::layer1::energy::PowerSource;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct SingularityGenerator {
    pub active: bool,
    pub mass_accumulated: f32,
}

#[derive(Event, Default)]
pub struct OrbitalDecayEvent {
    pub anomaly_strength: f32,
}

pub fn process_singularity_energy_system(
    mut query: Query<(&SingularityGenerator, &mut PowerSource)>,
) {
    for (generator, mut source) in query.iter_mut() {
        if generator.active {
            source.output = 1_000_000.0;
        } else {
            source.output = 0.0;
        }
    }
}

pub fn process_singularity_mass_accumulation_system(
    time: Res<bevy::time::Time>,
    mut query: Query<&mut SingularityGenerator>,
) {
    let dt = time.delta_secs();
    for mut generator in query.iter_mut() {
        if generator.active {
            generator.mass_accumulated += 10.0 * dt;
        }
    }
}

pub fn trigger_orbital_decay_system(
    mut query: Query<&mut SingularityGenerator>,
    mut decay_events: EventWriter<OrbitalDecayEvent>,
) {
    for mut generator in query.iter_mut() {
        if generator.mass_accumulated >= 10000.0 {
            decay_events.send(OrbitalDecayEvent {
                anomaly_strength: generator.mass_accumulated,
            });
            // Reset mass_accumulated so we don't spam the event bus
            generator.mass_accumulated = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::energy::PowerSource;
    use bevy::prelude::*;

    #[test]
    fn test_singularity_generator_provides_infinite_energy() {
        let mut app = App::new();
        app.add_systems(Update, process_singularity_energy_system);

        let entity = app
            .world_mut()
            .spawn((
                SingularityGenerator {
                    active: true,
                    mass_accumulated: 0.0,
                },
                PowerSource {
                    output: 0.0,
                    active: true,
                },
            ))
            .id();

        app.update();

        let source = app.world().get::<PowerSource>(entity).unwrap();
        assert!(
            source.output >= 999999.0,
            "Singularity should provide effectively infinite power"
        );
    }

    #[test]
    fn test_active_generator_increases_gravitational_mass() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin);
        app.add_systems(Update, process_singularity_mass_accumulation_system);

        let generator = app
            .world_mut()
            .spawn(SingularityGenerator {
                active: true,
                mass_accumulated: 0.0,
            })
            .id();

        // Initial update to initialize time and frame count
        app.update();

        // Manually advance the time resource and update
        let mut time = app.world_mut().resource_mut::<bevy::time::Time>();
        time.advance_by(std::time::Duration::from_secs_f32(1.0));
        app.update();

        let gen_state = app.world().get::<SingularityGenerator>(generator).unwrap();
        assert!(
            gen_state.mass_accumulated > 0.0,
            "Active generator must accumulate mass"
        );
    }

    #[test]
    fn test_critical_mass_triggers_layer_2_orbital_decay() {
        let mut app = App::new();
        app.add_event::<OrbitalDecayEvent>();
        app.add_systems(Update, trigger_orbital_decay_system);

        app.world_mut().spawn(SingularityGenerator {
            active: true,
            mass_accumulated: 10000.0,
        });

        app.update();

        let decay_events = app.world().resource::<Events<OrbitalDecayEvent>>();
        let mut cursor = decay_events.get_cursor();
        assert!(
            cursor.read(decay_events).len() > 0,
            "Critical mass should trigger an orbital decay event"
        );
    }
}
