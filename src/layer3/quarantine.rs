//! Interstellar Quarantine Fields
//!
//! Quarantines physical systems to prevent FTL travel in and out, isolating them.

use bevy::prelude::*;
use crate::layer2::fleet::{Fleet, InTransit, InOrbit};

/// Component indicating a star system is currently under quarantine.
/// This physically prevents FTL travel in and out of the system.
#[derive(Component)]
pub struct Quarantined;

/// Event emitted when a fleet attempts to enter or leave a quarantined system
/// and is bounced back.
#[derive(Event)]
pub struct QuarantineBounceEvent {
    /// The fleet that was bounced.
    pub fleet: Entity,
    /// The system that blocked the travel.
    pub system: Entity,
}

/// System that enforces interstellar quarantine fields.
/// It intercepts fleets `InTransit` and bounces them if their destination
/// or origin is quarantined.
pub fn enforce_quarantine_system(
    systems: Query<&Quarantined>,
    fleets: Query<(Entity, &InTransit), With<Fleet>>,
    mut bounce_events: EventWriter<QuarantineBounceEvent>,
    mut commands: Commands,
) {
    for (entity, transit) in fleets.iter() {
        let dest_entity = transit.destination;
        let origin_entity = transit.origin;

        let mut bounced_system = None;

        // Check destination
        if systems.get(dest_entity).is_ok() {
            bounced_system = Some(dest_entity);
        } else if systems.get(origin_entity).is_ok() {
            // Check current system (prevent leaving)
            bounced_system = Some(origin_entity);
        }

        if let Some(system) = bounced_system {
            bounce_events.send(QuarantineBounceEvent {
                fleet: entity,
                system,
            });

            // To make it bounce immediately, we put it back in orbit
            commands.entity(entity)
                .remove::<InTransit>()
                .insert(InOrbit { parent: origin_entity });
        }
    }
}


/// Plugin that registers the quarantine systems and events.
pub struct QuarantinePlugin;

impl Plugin for QuarantinePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<QuarantineBounceEvent>()
           .add_systems(Update, enforce_quarantine_system);
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

        let origin_system = world.spawn_empty().id();
        let target_system = world.spawn(Quarantined).id();

        let fleet_entity = world.spawn((Fleet, InTransit {
            origin: origin_system,
            destination: target_system,
            progress: 0.5,
            duration: 100.0,
        })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_quarantine_system);
        schedule.run(&mut world);

        // Fleet should be back in orbit at origin
        assert!(world.get::<InOrbit>(fleet_entity).is_some());
        assert_eq!(world.get::<InOrbit>(fleet_entity).unwrap().parent, origin_system);
        assert!(world.get::<InTransit>(fleet_entity).is_none());

        let events = world.resource::<Events<QuarantineBounceEvent>>();
        assert_eq!(events.get_cursor().len(events), 1, "Should emit a QuarantineBounceEvent");
    }

    #[test]
    fn test_fleet_cannot_leave_quarantined_system() {
        let mut world = setup_world();

        let origin_system = world.spawn(Quarantined).id();
        let target_system = world.spawn_empty().id();

        let fleet_entity = world.spawn((Fleet, InTransit {
            origin: origin_system,
            destination: target_system,
            progress: 0.1,
            duration: 100.0,
        })).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(enforce_quarantine_system);
        schedule.run(&mut world);

        // Fleet should be back in orbit at origin
        assert!(world.get::<InOrbit>(fleet_entity).is_some());
        assert_eq!(world.get::<InOrbit>(fleet_entity).unwrap().parent, origin_system);
        assert!(world.get::<InTransit>(fleet_entity).is_none());

        let events = world.resource::<Events<QuarantineBounceEvent>>();
        assert_eq!(events.get_cursor().len(events), 1, "Should emit a QuarantineBounceEvent");
    }
}
