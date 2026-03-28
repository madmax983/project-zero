//! Sensor Ambiguity
//!
//! Mechanic: Unidentified objects on the System Map appear as generic "Contacts"
//! with a "Signal Strength". High-tech sensors identify contacts at longer ranges.

use crate::layer2::fleet::{Fleet, InOrbit};
use bevy_ecs::prelude::*;

/// Component holding sensor data for a contact on the system map.
#[derive(Component, Debug, Clone)]
pub struct SensorContact {
    /// The strength of the sensor return.
    pub signal_strength: f32,
    /// The actual entity this contact represents, if known.
    pub resolved_entity: Option<Entity>,
}

/// Marker component for an unidentified contact.
#[derive(Component, Debug, Clone)]
pub struct UnidentifiedContact;

/// Component indicating an entity has sensors with a given range.
#[derive(Component, Debug, Clone)]
pub struct Sensors {
    /// Sensor range. For MVP, we treat range conceptually or abstractly.
    pub range: f32,
}

/// System to evaluate sensor contacts and resolve identities.
/// For MVP: fleets at the same `InOrbit` parent are fully identified.
/// Fleets at different `InOrbit` parents are `UnidentifiedContact`s.
#[allow(clippy::type_complexity)]
pub fn resolve_sensors_system(
    mut commands: Commands,
    sensor_query: Query<(&Sensors, Option<&InOrbit>)>,
    mut target_query: Query<(Entity, Option<&InOrbit>, Option<&mut SensorContact>), With<Fleet>>,
) {
    // Collect all observer locations
    let mut observer_locations = Vec::new();
    for (_, maybe_orbit) in &sensor_query {
        if let Some(orbit) = maybe_orbit {
            observer_locations.push(orbit.parent);
        }
    }

    // Evaluate all target fleets
    for (target_entity, maybe_orbit, mut maybe_contact) in target_query.iter_mut() {
        let mut is_close = false;

        if let Some(target_orbit) = maybe_orbit {
            if observer_locations.contains(&target_orbit.parent) {
                is_close = true;
            }
        }

        if is_close {
            // Remove UnidentifiedContact, update/create SensorContact
            commands
                .entity(target_entity)
                .remove::<UnidentifiedContact>();

            if let Some(ref mut contact) = maybe_contact {
                contact.resolved_entity = Some(target_entity);
            } else {
                commands.entity(target_entity).insert(SensorContact {
                    signal_strength: 100.0,
                    resolved_entity: Some(target_entity),
                });
            }
        } else {
            // Far away: Add UnidentifiedContact, update/create SensorContact without resolved_entity
            commands.entity(target_entity).insert(UnidentifiedContact);

            if let Some(ref mut contact) = maybe_contact {
                contact.resolved_entity = None;
            } else {
                commands.entity(target_entity).insert(SensorContact {
                    signal_strength: 10.0, // Basic signal strength
                    resolved_entity: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::fleet::{Fleet, InOrbit};

    #[test]
    fn test_distant_fleet_appears_as_unidentified_contact() {
        // Arrange: Fleet A with basic sensors, Fleet B far away
        let mut world = World::new();

        let planet_a = world.spawn_empty().id();
        let planet_b = world.spawn_empty().id();

        let _fleet_a = world
            .spawn((Fleet, Sensors { range: 10.0 }, InOrbit { parent: planet_a }))
            .id();

        let fleet_b = world.spawn((Fleet, InOrbit { parent: planet_b })).id();

        // Act: Evaluate sensor contacts for Fleet A
        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_sensors_system);
        schedule.run(&mut world);

        // Assert: Fleet B is detected as an `UnidentifiedContact` with basic signal strength, not a full `Fleet` entity
        assert!(
            world.get::<UnidentifiedContact>(fleet_b).is_some(),
            "Fleet B should be marked as UnidentifiedContact"
        );

        let contact = world
            .get::<SensorContact>(fleet_b)
            .expect("Fleet B should have a SensorContact");
        assert!(
            contact.resolved_entity.is_none(),
            "Distant contact should not be resolved"
        );
        assert!(
            contact.signal_strength > 0.0,
            "Contact should have some signal strength"
        );
    }

    #[test]
    fn test_close_proximity_reveals_contact_identity() {
        // Arrange: Fleet A moves close to an `UnidentifiedContact`
        let mut world = World::new();

        let planet = world.spawn_empty().id();

        let _fleet_a = world
            .spawn((Fleet, Sensors { range: 10.0 }, InOrbit { parent: planet }))
            .id();

        // Fleet B is at the same planet (close proximity)
        let fleet_b = world
            .spawn((
                Fleet,
                UnidentifiedContact, // Initially unidentified
                SensorContact {
                    signal_strength: 10.0,
                    resolved_entity: None,
                },
                InOrbit { parent: planet },
            ))
            .id();

        // Act: Evaluate sensor contacts
        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_sensors_system);
        schedule.run(&mut world);

        // Assert: Contact is resolved into its true identity
        assert!(
            world.get::<UnidentifiedContact>(fleet_b).is_none(),
            "UnidentifiedContact should be removed in close proximity"
        );

        let contact = world
            .get::<SensorContact>(fleet_b)
            .expect("Fleet B should still have a SensorContact");
        assert_eq!(
            contact.resolved_entity,
            Some(fleet_b),
            "Contact should be resolved to true identity"
        );
    }
}
