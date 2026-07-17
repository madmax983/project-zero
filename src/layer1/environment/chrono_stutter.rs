use bevy::prelude::*;

#[derive(Component)]
pub struct ChronoStutterAnomaly {
    pub radius: f32,
    pub time_multiplier: f32,
}

#[derive(Component)]
pub struct TimeModifier {
    pub multiplier: f32,
}

impl Default for TimeModifier {
    fn default() -> Self {
        Self { multiplier: 1.0 }
    }
}

pub fn apply_chrono_anomaly_system(
    anomaly_query: Query<(&ChronoStutterAnomaly, &Transform)>,
    mut entity_query: Query<(&mut TimeModifier, &Transform)>,
) {
    // Reset all modifiers first
    for (mut modifier, _) in entity_query.iter_mut() {
        if modifier.multiplier != 1.0 {
            modifier.multiplier = 1.0;
        }
    }

    // Apply highest anomaly modifier
    for (anomaly, anomaly_tf) in anomaly_query.iter() {
        for (mut modifier, tf) in entity_query.iter_mut() {
            let dist = tf.translation.distance(anomaly_tf.translation);

            if dist <= anomaly.radius && anomaly.time_multiplier > modifier.multiplier {
                modifier.multiplier = anomaly.time_multiplier;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_chrono_anomaly_applies_time_modifier_to_entities() {
        let mut world = setup_world();

        // Spawn Anomaly at origin
        world.spawn((
            ChronoStutterAnomaly { radius: 5.0, time_multiplier: 10.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Spawn Pop inside anomaly
        let pop_in = world.spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            TimeModifier::default(),
        )).id();

        // Spawn Pop outside anomaly
        let pop_out = world.spawn((
            Transform::from_xyz(10.0, 0.0, 0.0),
            TimeModifier::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_chrono_anomaly_system);
        schedule.run(&mut world);

        assert_eq!(world.get::<TimeModifier>(pop_in).unwrap().multiplier, 10.0);
        assert_eq!(world.get::<TimeModifier>(pop_out).unwrap().multiplier, 1.0);
    }

    #[test]
    fn test_overlapping_chrono_anomalies_applies_highest_multiplier() {
        let mut world = setup_world();

        // Spawn first Anomaly
        world.spawn((
            ChronoStutterAnomaly { radius: 5.0, time_multiplier: 2.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Spawn second Anomaly
        world.spawn((
            ChronoStutterAnomaly { radius: 5.0, time_multiplier: 10.0 },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Spawn Pop inside both anomalies
        let pop_in = world.spawn((
            Transform::from_xyz(2.0, 0.0, 0.0),
            TimeModifier::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_chrono_anomaly_system);
        schedule.run(&mut world);

        assert_eq!(world.get::<TimeModifier>(pop_in).unwrap().multiplier, 10.0);
    }
}
