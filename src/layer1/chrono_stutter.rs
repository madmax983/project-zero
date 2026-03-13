use crate::layer1::map::GridPosition;
use crate::layer1::pop::Speed;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ChronoAnomaly {
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

pub fn move_chrono_anomaly_system(
    mut anomaly_query: Query<&mut GridPosition, With<ChronoAnomaly>>,
) {
    for mut pos in anomaly_query.iter_mut() {
        if rand::random::<bool>() {
            pos.x = pos.x.saturating_add(1);
        }
    }
}

pub fn apply_chrono_speed_system(mut query: Query<(&TimeModifier, &mut Speed)>) {
    for (modifier, mut speed) in query.iter_mut() {
        speed.current *= modifier.multiplier;
    }
}

pub fn apply_chrono_anomaly_system(
    anomaly_query: Query<(&ChronoAnomaly, &GridPosition)>,
    mut entity_query: Query<(&mut TimeModifier, &GridPosition)>,
) {
    for (mut modifier, entity_pos) in entity_query.iter_mut() {
        let mut max_multiplier = 1.0_f32;

        for (anomaly, anomaly_pos) in anomaly_query.iter() {
            let dist = crate::layer1::utility_types::manhattan_distance(entity_pos, anomaly_pos);

            if dist as f32 <= anomaly.radius && anomaly.time_multiplier > max_multiplier {
                max_multiplier = anomaly.time_multiplier;
            }
        }

        if (modifier.multiplier - max_multiplier).abs() > f32::EPSILON {
            modifier.multiplier = max_multiplier;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_chrono_anomaly_applies_time_modifier_to_entities() {
        let mut world = setup_world();

        // Spawn Anomaly at origin
        world.spawn((
            ChronoAnomaly {
                radius: 5.0,
                time_multiplier: 10.0,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn Pop inside anomaly
        let pop_in = world
            .spawn((Pop, GridPosition { x: 2, y: 0 }, TimeModifier::default()))
            .id();

        // Spawn Pop outside anomaly
        let pop_out = world
            .spawn((Pop, GridPosition { x: 10, y: 0 }, TimeModifier::default()))
            .id();

        let _ = world.run_system_once(apply_chrono_anomaly_system);

        assert_eq!(world.get::<TimeModifier>(pop_in).unwrap().multiplier, 10.0);
        assert_eq!(world.get::<TimeModifier>(pop_out).unwrap().multiplier, 1.0);
    }
}
