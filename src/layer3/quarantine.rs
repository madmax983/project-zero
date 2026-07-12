use bevy::prelude::*;
use crate::layer2::fleet::FleetOrder;

/// Marker component for star systems that are under quarantine
#[derive(Component)]
pub struct Quarantined;

/// Event emitted when a fleet is bounced from a quarantined system
#[derive(Event, Debug, Clone)]
pub struct QuarantineBounceEvent {
    pub fleet: Entity,
    pub system: Entity,
}

/// System to enforce interstellar quarantine fields
/// Fleets with orders to move to a quarantined system have their orders revoked
pub fn enforce_quarantine_system(
    systems: Query<(), With<Quarantined>>,
    mut commands: Commands,
    fleets: Query<(Entity, &FleetOrder)>,
    mut bounce_events: EventWriter<QuarantineBounceEvent>,
) {
    for (fleet_entity, fleet_order) in fleets.iter() {
        if let FleetOrder::MoveTo(dest_entity) = fleet_order {
            if systems.get(*dest_entity).is_ok() {
                // If the target is quarantined, we stop the fleet
                commands.entity(fleet_entity).remove::<FleetOrder>();
                bounce_events.send(QuarantineBounceEvent {
                    fleet: fleet_entity,
                    system: *dest_entity,
                });
            }
        }
    }
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

        let target_system = world.spawn(Quarantined).id();
        let fleet_entity = world.spawn(FleetOrder::MoveTo(target_system)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_quarantine_system);
        schedule.run(&mut world);

        // Fleet order should be removed
        assert!(
            world.get::<FleetOrder>(fleet_entity).is_none(),
            "Fleet should be bounced from a quarantined system and lose its MoveTo order"
        );

        // Event should be emitted
        let events = world.resource::<Events<QuarantineBounceEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.len(events), 1);
        let event = reader.read(events).next().unwrap();
        assert_eq!(event.fleet, fleet_entity);
        assert_eq!(event.system, target_system);
    }
}
