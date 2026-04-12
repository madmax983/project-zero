use crate::layer1::administration::edicts::{ColonyPolicies, Policy};
use crate::layer1::core::integration::PirateAmnestyEvent;
use crate::layer2::fleet::{Fleet, FleetFaction, InOrbit};
use bevy::prelude::*;

/// Disbands Pirate Fleets in orbit if the Amnesty Visa policy is active.
#[allow(clippy::type_complexity)]
pub fn evaluate_pirate_amnesty_system(
    mut commands: Commands,
    policies: Res<ColonyPolicies>,
    fleets: Query<(Entity, &FleetFaction), (With<Fleet>, With<InOrbit>)>,
    mut amnesty_events: Option<ResMut<'_, bevy::prelude::Events<PirateAmnestyEvent>>>,
) {
    if !policies.is_active(Policy::AmnestyVisa) {
        return;
    }

    for (entity, faction) in fleets.iter() {
        if *faction == FleetFaction::Pirate {
            commands.entity(entity).despawn_recursive();
            if let Some(ref mut events) = amnesty_events {
                events.send(PirateAmnestyEvent { fleet: entity });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_pirate_amnesty() {
        let mut app = App::new();
        app.add_event::<PirateAmnestyEvent>();

        let mut policies = ColonyPolicies::default();
        policies.active_policies.insert(Policy::AmnestyVisa);
        app.insert_resource(policies);

        let fleet_entity = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Pirate,
                InOrbit {
                    parent: Entity::PLACEHOLDER,
                },
            ))
            .id();

        let other_fleet = app
            .world_mut()
            .spawn((
                Fleet,
                FleetFaction::Player,
                InOrbit {
                    parent: Entity::PLACEHOLDER,
                },
            ))
            .id();

        app.add_systems(Update, evaluate_pirate_amnesty_system);
        app.update();

        // Verify the pirate fleet is despawned
        assert!(
            app.world().get_entity(fleet_entity).is_err()
                || app.world().get::<Fleet>(fleet_entity).is_none()
        );

        // Verify the other fleet is unaffected
        assert!(app.world().get_entity(other_fleet).is_ok());
        assert!(app.world().get::<Fleet>(other_fleet).is_some());

        // Verify the event was sent
        let events = app.world().resource::<Events<PirateAmnestyEvent>>();
        let mut cursor = events.get_cursor();
        let mut event_count = 0;
        for ev in cursor.read(events) {
            assert_eq!(ev.fleet, fleet_entity);
            event_count += 1;
        }
        assert_eq!(event_count, 1);
    }
}
