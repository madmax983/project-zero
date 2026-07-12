use bevy::prelude::*;
use crate::layer2::fleet::{Fleet, InTransit};

/// Event emitted when a fleet tries to enter or leave a quarantined system
#[derive(Event, Debug)]
pub struct QuarantineBounceEvent {
    pub fleet: Entity,
    pub system: Entity,
}

/// A component marking a star system as under quarantine.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Quarantine {
    pub is_active: bool,
}

pub fn enforce_quarantine_system(
    systems: Query<(Entity, &Quarantine)>,
    mut fleets: Query<(Entity, &mut InTransit), With<Fleet>>,
    mut bounce_events: EventWriter<QuarantineBounceEvent>,
) {
    for (fleet_entity, mut transit) in fleets.iter_mut() {
        if transit.origin == transit.destination {
            continue; // Already bounced or not moving
        }

        let mut bounced = false;

        if let Ok((system_entity, quarantine)) = systems.get(transit.destination) {
            if quarantine.is_active {
                // Bounce the fleet back to its origin
                transit.destination = transit.origin;
                bounced = true;

                bounce_events.send(QuarantineBounceEvent {
                    fleet: fleet_entity,
                    system: system_entity,
                });
            }
        }

        if !bounced {
            // Also check if fleet is inside a quarantined system attempting to leave
            if let Ok((system_entity, quarantine)) = systems.get(transit.origin) {
                if quarantine.is_active {
                    // Bounce the fleet back to its origin (which is the quarantined system)
                    transit.destination = transit.origin;

                    bounce_events.send(QuarantineBounceEvent {
                        fleet: fleet_entity,
                        system: system_entity,
                    });
                }
            }
        }
    }
}

/// Returns the pathfinding weight for a given node (StarSystem).
/// Returns `f32::INFINITY` if the node is quarantined.
pub fn get_system_pathfinding_weight(world: &World, system_entity: Entity) -> f32 {
    if let Some(quarantine) = world.get::<Quarantine>(system_entity) {
        if quarantine.is_active {
            return f32::INFINITY;
        }
    }
    1.0 // Base weight for normal systems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<QuarantineBounceEvent>>();
        world
    }

    #[test]
    fn test_fleet_cannot_enter_quarantined_system() {
        let mut world = setup_world();

        let target_system = world.spawn(Quarantine { is_active: true }).id();
        let origin_system = world.spawn(Quarantine { is_active: false }).id();

        let fleet_entity = world.spawn((
            Fleet,
            InTransit {
                origin: origin_system,
                destination: target_system,
                progress: 0.5,
                duration: 10.0,
            }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_quarantine_system);
        schedule.run(&mut world);

        let fleet = world.get::<InTransit>(fleet_entity).unwrap();
        assert_eq!(fleet.destination, origin_system, "Fleet should be bounced back to its origin");

        let events = world.resource::<Events<QuarantineBounceEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1, "Should emit one QuarantineBounceEvent");
    }

    #[test]
    fn test_fleet_cannot_leave_quarantined_system() {
        let mut world = setup_world();

        let target_system = world.spawn(Quarantine { is_active: false }).id();
        let origin_system = world.spawn(Quarantine { is_active: true }).id();

        let fleet_entity = world.spawn((
            Fleet,
            InTransit {
                origin: origin_system,
                destination: target_system,
                progress: 0.5,
                duration: 10.0,
            }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_quarantine_system);
        schedule.run(&mut world);

        let fleet = world.get::<InTransit>(fleet_entity).unwrap();
        assert_eq!(fleet.destination, origin_system, "Fleet should be bounced back to its quarantined origin");

        let events = world.resource::<Events<QuarantineBounceEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1, "Should emit one QuarantineBounceEvent");
    }

    #[test]
    fn test_pathfinding_graph_weight_is_infinity_for_quarantined_nodes() {
        let mut world = setup_world();
        let q_system = world.spawn(Quarantine { is_active: true }).id();
        let normal_system = world.spawn(Quarantine { is_active: false }).id();
        let no_comp_system = world.spawn_empty().id();

        assert_eq!(get_system_pathfinding_weight(&world, q_system), f32::INFINITY);
        assert_eq!(get_system_pathfinding_weight(&world, normal_system), 1.0);
        assert_eq!(get_system_pathfinding_weight(&world, no_comp_system), 1.0);
    }
}
