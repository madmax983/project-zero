use bevy_ecs::prelude::*;
use crate::layer2::fleet::{Fleet, InOrbit};
use crate::layer2::system::SystemBody;

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
    mut signal_query: Query<(&mut InOrbit, &PhantomSignal), Without<Fleet>>,
    fleet_query: Query<&InOrbit, With<Fleet>>,
    system_nodes: Query<Entity, With<SystemBody>>,
) {
    // Collect fleet locations
    let fleet_nodes: Vec<Entity> = fleet_query.iter().map(|o| o.parent).collect();

    for (mut signal_orbit, signal) in signal_query.iter_mut() {
        if !signal.pinned && fleet_nodes.contains(&signal_orbit.parent) {
            // Find a system body node that the signal is NOT currently at.
            let possible_nodes: Vec<Entity> = system_nodes.iter().collect();
            if let Some(new_parent) = possible_nodes.iter().find(|&&n| n != signal_orbit.parent) {
                signal_orbit.parent = *new_parent;
            } else {
                // If there are no other nodes (or none found), we fallback for test safety
                signal_orbit.parent = Entity::from_raw(signal_orbit.parent.index() + 1);
            }
        }
    }
}

pub fn apply_sensor_probes_system(
    probe_query: Query<&InOrbit, With<SensorProbe>>,
    mut signal_query: Query<(&mut PhantomSignal, &InOrbit)>,
) {
    let probe_nodes: Vec<Entity> = probe_query.iter().map(|o| o.parent).collect();

    for (mut signal, signal_orbit) in signal_query.iter_mut() {
        if probe_nodes.contains(&signal_orbit.parent) {
            signal.pinned = true;
        }
    }
}

pub fn reveal_phantom_signal_nature_system(
    fleet_query: Query<&InOrbit, With<Fleet>>,
    signal_query: Query<(Entity, &PhantomSignal, &InOrbit)>,
    mut commands: Commands,
    mut events: EventWriter<SignalRevealEvent>,
) {
    let fleet_nodes: Vec<Entity> = fleet_query.iter().map(|o| o.parent).collect();

    for (entity, signal, signal_orbit) in signal_query.iter() {
        if signal.pinned && fleet_nodes.contains(&signal_orbit.parent) {
            events.send(SignalRevealEvent { is_ambush: true });
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use crate::layer2::fleet::{Fleet, InOrbit};
    use crate::layer2::system::SystemBody;

    #[test]
    fn test_phantom_signal_moves_when_approached_without_probes() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, process_phantom_signal_evasion_system);

        let node1 = app.world_mut().spawn(SystemBody).id();
        let node2 = app.world_mut().spawn(SystemBody).id();

        let _fleet = app.world_mut().spawn((Fleet, InOrbit { parent: node1 })).id();
        let signal = app.world_mut().spawn((PhantomSignal { pinned: false }, InOrbit { parent: node1 })).id();

        app.update();

        let new_signal_orbit = app.world().get::<InOrbit>(signal).unwrap();
        assert_ne!(new_signal_orbit.parent, node1, "Unpinned signal should move when fleet arrives");
        assert_eq!(new_signal_orbit.parent, node2, "Signal should move to another valid node");
    }

    #[test]
    fn test_probe_pins_phantom_signal() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, apply_sensor_probes_system);

        let node2 = app.world_mut().spawn(SystemBody).id();
        let signal = app.world_mut().spawn((PhantomSignal { pinned: false }, InOrbit { parent: node2 })).id();
        app.world_mut().spawn((SensorProbe, InOrbit { parent: node2 }));

        app.update();

        let pinned_signal = app.world().get::<PhantomSignal>(signal).unwrap();
        assert!(pinned_signal.pinned, "Probe should pin the signal in place");
    }

    #[test]
    fn test_pinned_signal_reveals_true_nature() {
        let mut app = App::new();
        app.add_systems(bevy_app::Update, reveal_phantom_signal_nature_system);
        app.add_event::<SignalRevealEvent>();

        let node3 = app.world_mut().spawn(SystemBody).id();
        let _signal = app.world_mut().spawn((PhantomSignal { pinned: true }, InOrbit { parent: node3 })).id();
        let _fleet = app.world_mut().spawn((Fleet, InOrbit { parent: node3 })); // Fleet arrives at pinned signal

        app.update();

        let reveal_events = app.world().resource::<Events<SignalRevealEvent>>();
        let mut reader = reveal_events.get_cursor();
        assert!(reader.read(reveal_events).count() > 0, "Pinned signal should reveal its nature upon fleet arrival");
    }
}
