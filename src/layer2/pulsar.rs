use crate::layer1::biology::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer2::ship::Ship;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Pulsar {
    pub rotation_period: f32, // Seconds for a full 360 degree rotation
    pub beam_width: f32,      // Radians
}

#[derive(Component)]
pub struct Facing(pub f32); // Radians

pub fn pulsar_rotation_system(
    _time: Res<SimulationTime>,
    mut query: Query<(&Pulsar, &mut Facing)>,
) {
    for (pulsar, mut facing) in query.iter_mut() {
        let rotation_speed = (2.0 * std::f32::consts::PI) / pulsar.rotation_period;
        let delta_seconds = 0.1; // 1 tick = 100ms
        facing.0 += rotation_speed * delta_seconds;
        // Normalize to 0..2PI
        facing.0 %= 2.0 * std::f32::consts::PI;
    }
}

pub fn pulsar_radiation_damage_system(
    pulsars: Query<(&GridPosition, &Facing, &Pulsar)>,
    mut ships: Query<(&GridPosition, &mut Health), With<Ship>>,
) {
    let damage_per_second = 10.0;
    let delta_seconds = 0.1;

    for (pulsar_pos, facing, pulsar) in pulsars.iter() {
        for (ship_pos, mut health) in ships.iter_mut() {
            let dx = (ship_pos.x - pulsar_pos.x) as f32;
            let dy = (ship_pos.y - pulsar_pos.y) as f32;
            let angle_to_ship = dy.atan2(dx);

            // Normalize angle relative to facing
            let mut angle_diff = (angle_to_ship - facing.0).abs();
            if angle_diff > std::f32::consts::PI {
                angle_diff = 2.0 * std::f32::consts::PI - angle_diff;
            }

            if angle_diff <= pulsar.beam_width / 2.0 {
                health.take_damage(damage_per_second * delta_seconds);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::ship::ShipType;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_pulsar_beam_rotation() {
        let mut world = World::new();
        // Arrange
        let pulsar = world
            .spawn((
                Pulsar {
                    rotation_period: 10.0,
                    beam_width: std::f32::consts::PI / 4.0, // 45 degrees
                },
                Facing(0.0), // Starts pointing at 0 radians
            ))
            .id();

        // Act - advance time by 2.5 seconds (1/4 period, 25 ticks)
        let time = SimulationTime {
            tick: 25,
            ..Default::default()
        };
        world.insert_resource(time);
        for _ in 0..25 {
            let _ = world.run_system_once(pulsar_rotation_system);
        }

        // Assert - should rotate by PI/2 radians (90 degrees)
        let facing = world.get::<Facing>(pulsar).unwrap();
        assert!((facing.0 - std::f32::consts::PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_ship_takes_damage_in_beam() {
        let mut world = World::new();
        // Arrange
        let _pulsar = world
            .spawn((
                Pulsar {
                    rotation_period: 10.0,
                    beam_width: std::f32::consts::PI / 4.0,
                },
                GridPosition { x: 0, y: 0 },
                Facing(0.0), // Pointing right
            ))
            .id();

        let ship_in_beam = world
            .spawn((
                Ship::new(ShipType::Scout),
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                GridPosition { x: 10, y: 0 }, // Right
            ))
            .id();

        let ship_out_beam = world
            .spawn((
                Ship::new(ShipType::Scout),
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                GridPosition { x: 0, y: 10 }, // Up
            ))
            .id();

        // Act
        let time = SimulationTime {
            tick: 10,
            ..Default::default()
        };
        world.insert_resource(time);
        for _ in 0..10 {
            let _ = world.run_system_once(pulsar_radiation_damage_system);
        }

        // Assert
        let health_in = world.get::<Health>(ship_in_beam).unwrap();
        assert!(health_in.current < 100.0, "Ship in beam should take damage");

        let health_out = world.get::<Health>(ship_out_beam).unwrap();
        assert_eq!(
            health_out.current, 100.0,
            "Ship out of beam should not take damage"
        );
    }
}
