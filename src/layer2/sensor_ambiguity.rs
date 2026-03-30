use bevy_ecs::prelude::*;
use crate::layer2::fleet::{Fleet, InOrbit, InTransit};
use crate::layer2::system::Orbit;

/// A component that represents a contact detected by sensors.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SensorContact {
    /// Signal strength of the contact, from 0.0 (weak) to 1.0 (strong).
    pub signal_strength: f32,
    /// If resolved, the entity that the contact represents.
    pub resolved_entity: Option<Entity>,
}

/// A component that gives an entity the ability to detect other entities within a certain range.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct SensorRange {
    /// The maximum distance at which this entity can detect others.
    pub range: f32,
}

/// System that evaluates sensor contacts for all entities with a `SensorRange`.
#[allow(clippy::type_complexity)]
pub fn resolve_sensors_system(
    mut commands: Commands,
    sensor_query: Query<(Entity, &SensorRange, Option<&InOrbit>, Option<&InTransit>), With<Fleet>>,
    target_query: Query<(Entity, Option<&InOrbit>, Option<&InTransit>), With<Fleet>>,
    orbit_query: Query<&Orbit>,
) {
    // Recursive function to get absolute position (handling nested orbits like moons)
    fn get_absolute_pos(entity_id: Entity, orbit_query: &Query<&Orbit>) -> (f32, f32) {
        if let Ok(orbit) = orbit_query.get(entity_id) {
            // Calculate local offset
            // In system view, X is scaled by 2.0 to look circular in terminal (2:1 char aspect)
            let local_x = orbit.radius * 2.0 * orbit.angle.cos();
            let local_y = orbit.radius * orbit.angle.sin();

            // Get parent absolute position
            let (parent_x, parent_y) = get_absolute_pos(orbit.parent, orbit_query);

            (parent_x + local_x, parent_y + local_y)
        } else {
            // Base case: no orbit (e.g., the central Star), so (0, 0) relative
            (0.0, 0.0)
        }
    }

    let get_entity_pos = |in_orbit: Option<&InOrbit>, in_transit: Option<&InTransit>| -> Option<(f32, f32)> {
        if let Some(orbit) = in_orbit {
            Some(get_absolute_pos(orbit.parent, &orbit_query))
        } else if let Some(transit) = in_transit {
            let start = get_absolute_pos(transit.origin, &orbit_query);
            let end = get_absolute_pos(transit.destination, &orbit_query);
            let x = start.0 + (end.0 - start.0) * transit.progress;
            let y = start.1 + (end.1 - start.1) * transit.progress;
            Some((x, y))
        } else {
            None
        }
    };

    // Keep track of the highest signal strength for each target
    let mut target_contacts: std::collections::HashMap<Entity, SensorContact> = std::collections::HashMap::new();

    for (sensor_entity, sensor_range, sensor_orbit, sensor_transit) in &sensor_query {
        let sensor_pos = get_entity_pos(sensor_orbit, sensor_transit).unwrap_or((0.0, 0.0));

        for (target_entity, target_orbit, target_transit) in &target_query {
            // A sensor does not detect itself
            if sensor_entity == target_entity {
                continue;
            }

            let target_pos = get_entity_pos(target_orbit, target_transit).unwrap_or((0.0, 0.0));

            let distance = ((sensor_pos.0 - target_pos.0).powi(2) + (sensor_pos.1 - target_pos.1).powi(2)).sqrt();

            // Additional distance check: if both are at the same body, distance is 0.
            let adjusted_distance = if let (Some(so), Some(to)) = (sensor_orbit, target_orbit) {
                if so.parent == to.parent { 0.0 } else { distance }
            } else {
                distance
            };

            if adjusted_distance <= sensor_range.range {
                let signal_strength = (1.0 - (adjusted_distance / sensor_range.range)).clamp(0.0, 1.0);
                let resolved_entity = if adjusted_distance <= sensor_range.range * 0.5 {
                    Some(target_entity)
                } else {
                    None
                };

                let contact = SensorContact {
                    signal_strength,
                    resolved_entity,
                };

                // Update best contact
                target_contacts.entry(target_entity)
                    .and_modify(|existing| {
                        if contact.signal_strength > existing.signal_strength {
                            *existing = contact;
                        }
                    })
                    .or_insert(contact);
            }
        }
    }

    // Apply the contacts, removing old ones if no longer seen
    for (target_entity, _, _) in &target_query {
        if let Some(contact) = target_contacts.get(&target_entity) {
            commands.entity(target_entity).insert(*contact);
        } else {
            commands.entity(target_entity).remove::<SensorContact>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distant_fleet_appears_as_unidentified_contact() {
        let mut world = World::new();
        let planet_a = world.spawn(Orbit { parent: Entity::PLACEHOLDER, radius: 10.0, speed: 0.0, angle: 0.0 }).id();
        let planet_b = world.spawn(Orbit { parent: Entity::PLACEHOLDER, radius: 100.0, speed: 0.0, angle: 0.0 }).id();

        let _ = world.spawn((
            Fleet,
            SensorRange { range: 200.0 }, // Can see planet B (distance is 90)
            InOrbit { parent: planet_a }
        ));

        let fleet_b = world.spawn((
            Fleet,
            InOrbit { parent: planet_b }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_sensors_system);
        schedule.run(&mut world);

        let contact = world.get::<SensorContact>(fleet_b).expect("Target should have a SensorContact");
        assert!(contact.signal_strength > 0.0 && contact.signal_strength < 1.0);
        assert_eq!(contact.resolved_entity, None);
    }

    #[test]
    fn test_close_proximity_reveals_contact_identity() {
        let mut world = World::new();
        let planet_a = world.spawn(Orbit { parent: Entity::PLACEHOLDER, radius: 10.0, speed: 0.0, angle: 0.0 }).id();

        let _ = world.spawn((
            Fleet,
            SensorRange { range: 100.0 },
            InOrbit { parent: planet_a }
        ));

        let fleet_b = world.spawn((
            Fleet,
            InOrbit { parent: planet_a }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_sensors_system);
        schedule.run(&mut world);

        let contact = world.get::<SensorContact>(fleet_b).expect("Target should have a SensorContact");
        assert_eq!(contact.signal_strength, 1.0);
        assert_eq!(contact.resolved_entity, Some(fleet_b));
    }
}
