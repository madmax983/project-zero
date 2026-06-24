use crate::layer2::ship::logistics::Position;
use crate::layer2::ship::Ship;
use bevy::prelude::*;
use std::f32::consts::PI;

/// A celestial pulsar that rotates and emits a deadly radiation beam.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Pulsar {
    /// Time in seconds for a full 360 degree rotation.
    pub rotation_period: f32,
    /// Width of the radiation beam in radians.
    pub beam_width: f32,
    /// Damage applied per second to ships caught in the beam.
    pub damage_per_second: f32,
}

/// The current rotational facing of an entity, in radians.
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct Facing(pub f32);

/// Rotates pulsars based on their rotation period.
pub fn pulsar_rotation_system(time: Res<Time>, mut query: Query<(&Pulsar, &mut Facing)>) {
    let delta = time.delta_secs();
    for (pulsar, mut facing) in query.iter_mut() {
        let rotation_speed = (2.0 * PI) / pulsar.rotation_period;
        facing.0 += rotation_speed * delta;
        facing.0 %= 2.0 * PI;
    }
}

/// Applies damage to ships that are caught within a pulsar's radiation beam.
pub fn pulsar_radiation_damage_system(
    time: Res<Time>,
    pulsars: Query<(&Position, &Facing, &Pulsar)>,
    mut ships: Query<(&Position, &mut Ship)>,
) {
    let delta = time.delta_secs();
    for (pulsar_pos, facing, pulsar) in pulsars.iter() {
        for (ship_pos, mut ship) in ships.iter_mut() {
            let dx = ship_pos.0.x - pulsar_pos.0.x;
            let dy = ship_pos.0.y - pulsar_pos.0.y;
            let angle_to_ship = dy.atan2(dx);

            let mut angle_diff = (angle_to_ship - facing.0).abs();
            // Normalize angle diff to [0, PI]
            while angle_diff > PI {
                angle_diff -= 2.0 * PI;
            }
            angle_diff = angle_diff.abs();

            if angle_diff <= pulsar.beam_width / 2.0 {
                ship.health -= pulsar.damage_per_second * delta;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::ship::ShipType;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn test_pulsar_beam_rotation() {
        let mut world = World::new();
        // Arrange
        let pulsar = world
            .spawn((
                Pulsar {
                    rotation_period: 10.0,
                    beam_width: PI / 4.0, // 45 degrees
                    damage_per_second: 10.0,
                },
                Facing(0.0), // Starts pointing at 0 radians
            ))
            .id();

        // Act - advance time by 2.5 seconds (1/4 period)
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(2500));
        world.insert_resource(time);

        world.run_system_once(pulsar_rotation_system).unwrap();

        // Assert - should rotate by PI/2 radians (90 degrees)
        let facing = world.get::<Facing>(pulsar).unwrap();
        assert!((facing.0 - PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_ship_takes_damage_in_beam() {
        let mut world = World::new();
        // Arrange
        world.spawn((
            Pulsar {
                rotation_period: 10.0,
                beam_width: PI / 4.0,
                damage_per_second: 10.0,
            },
            Position(Vec2::new(0.0, 0.0)),
            Facing(0.0), // Pointing right
        ));

        let ship_in_beam = world
            .spawn((
                Ship::new(ShipType::Scout),     // Will have 20.0 max health
                Position(Vec2::new(10.0, 0.0)), // Right
            ))
            .id();

        let ship_out_beam = world
            .spawn((
                Ship::new(ShipType::Scout),     // Will have 20.0 max health
                Position(Vec2::new(0.0, 10.0)), // Up
            ))
            .id();

        // Act
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(1));
        world.insert_resource(time);

        world
            .run_system_once(pulsar_radiation_damage_system)
            .unwrap();

        // Assert
        let ship_in = world.get::<Ship>(ship_in_beam).unwrap();
        assert!(ship_in.health < 20.0, "Ship in beam should take damage");
        // Initial health is 20.0. Takes 10.0 damage.
        assert!((ship_in.health - 10.0).abs() < 0.001);

        let ship_out = world.get::<Ship>(ship_out_beam).unwrap();
        assert_eq!(
            ship_out.health, 20.0,
            "Ship out of beam should not take damage"
        );
    }
}
