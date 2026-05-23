use bevy_ecs::prelude::*;
use crate::layer2::fleet::{Fleet, InOrbit};
use crate::layer2::system::SystemBody;
use rand::seq::SliceRandom;

#[derive(Component)]
pub struct PhantomSignal {
    pub pinned: bool,
    pub location: Entity, // Represents the system node/entity the signal is at
}

#[derive(Component)]
pub struct SensorProbe;

#[derive(Event)]
pub struct SignalRevealEvent {
    pub is_ambush: bool,
}

pub fn process_phantom_signal_evasion_system(
    mut signal_query: Query<&mut PhantomSignal>,
    fleet_query: Query<&InOrbit, With<Fleet>>,
    nodes_query: Query<Entity, With<SystemBody>>, // Get actual system nodes
) {
    let nodes: Vec<Entity> = nodes_query.iter().collect();
    let mut rng = rand::thread_rng();

    for mut signal in signal_query.iter_mut() {
        if !signal.pinned {
            for fleet_orbit in fleet_query.iter() {
                if signal.location == fleet_orbit.parent {
                    // Try to move to a different node if possible
                    let other_nodes: Vec<Entity> = nodes.iter().filter(|&&n| n != signal.location).copied().collect();
                    if !other_nodes.is_empty() {
                        if let Some(&new_node) = other_nodes.choose(&mut rng) {
                            signal.location = new_node;
                        }
                    } else if let Some(&fallback_node) = nodes.first() {
                        signal.location = fallback_node;
                    } else {
                        // Very extreme edge case, no SystemBody exists.
                        // We do nothing instead of creating a dangling entity.
                    }
                }
            }
        }
    }
}

pub fn apply_sensor_probes_system(
    probe_query: Query<&InOrbit, With<SensorProbe>>,
    mut signal_query: Query<&mut PhantomSignal>,
) {
    for probe_orbit in probe_query.iter() {
        for mut signal in signal_query.iter_mut() {
            if probe_orbit.parent == signal.location {
                signal.pinned = true;
            }
        }
    }
}

pub fn reveal_phantom_signal_nature_system(
    fleet_query: Query<&InOrbit, With<Fleet>>,
    signal_query: Query<(Entity, &PhantomSignal)>,
    mut commands: Commands,
    mut events: EventWriter<SignalRevealEvent>,
) {
    for (entity, signal) in signal_query.iter() {
        if signal.pinned {
            let mut revealed = false;
            for fleet_orbit in fleet_query.iter() {
                if fleet_orbit.parent == signal.location {
                    revealed = true;
                    break;
                }
            }
            if revealed {
                events.send(SignalRevealEvent { is_ambush: true });
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_phantom_signal_moves_when_approached_without_probes() {
        let mut app = App::new();
        app.add_systems(Update, process_phantom_signal_evasion_system);

        let node_1 = app.world_mut().spawn(SystemBody).id();
        let node_2 = app.world_mut().spawn(SystemBody).id();

        let _fleet = app.world_mut().spawn((Fleet, InOrbit { parent: node_1 })).id();
        let signal = app.world_mut().spawn(PhantomSignal { pinned: false, location: node_1 }).id();

        app.update();

        let updated_signal = app.world().get::<PhantomSignal>(signal).unwrap();
        assert_ne!(updated_signal.location, node_1, "Unpinned signal should move when fleet arrives");
        assert_eq!(updated_signal.location, node_2, "Signal should move to the other node");
    }

    #[test]
    fn test_probe_pins_phantom_signal() {
        let mut app = App::new();
        app.add_systems(Update, apply_sensor_probes_system);

        let node_2 = app.world_mut().spawn(SystemBody).id();

        let signal = app.world_mut().spawn(PhantomSignal { pinned: false, location: node_2 }).id();
        app.world_mut().spawn((SensorProbe, InOrbit { parent: node_2 }));

        app.update();

        let pinned_signal = app.world().get::<PhantomSignal>(signal).unwrap();
        assert!(pinned_signal.pinned, "Probe should pin the signal in place");
    }

    #[test]
    fn test_pinned_signal_reveals_true_nature() {
        let mut app = App::new();
        app.add_systems(Update, reveal_phantom_signal_nature_system);
        app.add_event::<SignalRevealEvent>();

        let node_3 = app.world_mut().spawn(SystemBody).id();

        let _signal = app.world_mut().spawn(PhantomSignal { pinned: true, location: node_3 }).id();
        let _fleet = app.world_mut().spawn((Fleet, InOrbit { parent: node_3 }));

        app.update();

        let events = app.world().resource::<Events<SignalRevealEvent>>();
        #[allow(deprecated)]
        let mut reader = events.get_reader();
        assert!(reader.read(events).count() > 0, "Pinned signal should reveal its nature upon fleet arrival");
    }
}
