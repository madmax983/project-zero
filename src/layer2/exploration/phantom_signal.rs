use bevy_ecs::prelude::*;
use crate::layer2::fleet::InOrbit;

#[derive(Component)]
pub struct PhantomSignal {
    pub pinned: bool,
}

#[derive(Component)]
pub struct SensorProbe;

#[derive(Event)]
pub struct SignalRevealEvent {
    pub is_ambush: bool,
}

pub fn process_phantom_signal_evasion_system(
    mut signal_query: Query<(Entity, &mut InOrbit, &PhantomSignal), Without<crate::layer2::fleet::Fleet>>,
    fleet_query: Query<&InOrbit, With<crate::layer2::fleet::Fleet>>,
) {
    for (_entity, mut signal_orbit, signal) in signal_query.iter_mut() {
        if !signal.pinned {
            for fleet_orbit in fleet_query.iter() {
                if signal_orbit.parent == fleet_orbit.parent {
                    // Evade - simulate moving by spawning a dummy new destination node or just re-assigning parent
                    // Here we'll just set it to a placeholder Entity to simulate evasion
                    signal_orbit.parent = Entity::PLACEHOLDER;
                }
            }
        }
    }
}

pub fn apply_sensor_probes_system(
    probe_query: Query<&InOrbit, With<SensorProbe>>,
    mut signal_query: Query<(&mut PhantomSignal, &InOrbit)>,
) {
    for probe_orbit in probe_query.iter() {
        for (mut signal, signal_orbit) in signal_query.iter_mut() {
            if probe_orbit.parent == signal_orbit.parent {
                signal.pinned = true;
            }
        }
    }
}

pub fn reveal_phantom_signal_nature_system(
    fleet_query: Query<&InOrbit, With<crate::layer2::fleet::Fleet>>,
    signal_query: Query<(Entity, &PhantomSignal, &InOrbit)>,
    mut commands: Commands,
    mut events: EventWriter<SignalRevealEvent>,
) {
    for fleet_orbit in fleet_query.iter() {
        for (entity, signal, signal_orbit) in signal_query.iter() {
            if signal.pinned && fleet_orbit.parent == signal_orbit.parent {
                events.send(SignalRevealEvent { is_ambush: true }); // Assume ambush for now
                commands.entity(entity).despawn(); // Remove the signal
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;
    use bevy_app::{App, Update};
    use crate::layer2::fleet::Fleet;

    #[test]
    fn test_phantom_signal_moves_when_approached_without_probes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, process_phantom_signal_evasion_system);

        let node1 = Entity::from_raw(1);
        let _fleet = app.world_mut().spawn((Fleet, InOrbit { parent: node1 })).id();
        let signal = app.world_mut().spawn((PhantomSignal { pinned: false }, InOrbit { parent: node1 })).id();

        app.update();

        let new_signal_orbit = app.world().get::<InOrbit>(signal).unwrap();
        assert_ne!(new_signal_orbit.parent, node1, "Unpinned signal should move when fleet arrives");
    }

    #[test]
    fn test_probe_pins_phantom_signal() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, apply_sensor_probes_system);

        let node2 = Entity::from_raw(2);
        let signal = app.world_mut().spawn((PhantomSignal { pinned: false }, InOrbit { parent: node2 })).id();
        app.world_mut().spawn((SensorProbe, InOrbit { parent: node2 }));

        app.update();

        let pinned_signal = app.world().get::<PhantomSignal>(signal).unwrap();
        assert!(pinned_signal.pinned, "Probe should pin the signal in place");
    }

    #[test]
    fn test_pinned_signal_reveals_true_nature() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, reveal_phantom_signal_nature_system);
        app.add_event::<SignalRevealEvent>();

        let node3 = Entity::from_raw(3);
        let _signal = app.world_mut().spawn((PhantomSignal { pinned: true }, InOrbit { parent: node3 })).id();
        let _fleet = app.world_mut().spawn((Fleet, InOrbit { parent: node3 })); // Fleet arrives at pinned signal

        app.update();

        let reveal_events = app.world().resource::<Events<SignalRevealEvent>>();
        let mut reader = reveal_events.get_cursor();
        assert!(reader.read(reveal_events).len() > 0, "Pinned signal should reveal its nature upon fleet arrival");
    }
}
