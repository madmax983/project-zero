use bevy::prelude::*;
use crate::layer2::ship::Ship;
use crate::layer2::ship::logistics::Position;
use crate::shared::time::{SimulationTime, SimSpeed};

#[derive(Component)]
pub struct Pulsar {
    pub rotation_period: f32, // Ticks for a full 360 degree rotation
    pub beam_width: f32,      // Radians
}

#[derive(Component)]
pub struct Facing(pub f32); // Radians

pub fn pulsar_rotation_system(
    time: Res<SimulationTime>,
    mut query: Query<(&Pulsar, &mut Facing)>,
) {
    if time.speed == SimSpeed::Paused {
        return;
    }

    // Using ticks. E.g. rotation_period is 100 ticks.
    for (pulsar, mut facing) in query.iter_mut() {
        let rotation_speed = (2.0 * std::f32::consts::PI) / pulsar.rotation_period;
        facing.0 += rotation_speed;
        facing.0 = facing.0.rem_euclid(2.0 * std::f32::consts::PI);
    }
}

pub fn pulsar_radiation_damage_system(
    time: Res<SimulationTime>,
    pulsars: Query<(&Position, &Facing, &Pulsar)>,
    mut ships: Query<(&Position, &mut Ship)>,
) {
    if time.speed == SimSpeed::Paused {
        return;
    }

    let damage_per_tick = 1.0; // Flat damage per tick

    for (pulsar_pos, facing, pulsar) in pulsars.iter() {
        for (ship_pos, mut ship) in ships.iter_mut() {
            let dx = ship_pos.0.x - pulsar_pos.0.x;
            let dy = ship_pos.0.y - pulsar_pos.0.y;
            let angle_to_ship = dy.atan2(dx);

            // Normalize angle relative to facing using rem_euclid for safety
            let mut angle_diff = (angle_to_ship - facing.0).abs();
            if angle_diff > std::f32::consts::PI {
                angle_diff = 2.0 * std::f32::consts::PI - angle_diff;
            }

            if angle_diff <= pulsar.beam_width / 2.0 {
                ship.health -= damage_per_tick;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::ship::ShipType;

    #[test]
    fn test_pulsar_beam_rotation() {
        let mut app = App::new();
        // Arrange
        let pulsar = app.world_mut().spawn((
            Pulsar {
                rotation_period: 100.0, // 100 ticks for a full rotation
                beam_width: std::f32::consts::PI / 4.0, // 45 degrees
            },
            Facing(0.0), // Starts pointing at 0 radians
        )).id();

        app.insert_resource(SimulationTime { speed: SimSpeed::Normal, ..Default::default() });
        app.add_systems(Update, pulsar_rotation_system);

        // Act - advance time by 25 ticks (1/4 period)
        for _ in 0..25 {
            app.update();
        }

        // Assert - should rotate by PI/2 radians (90 degrees)
        let facing = app.world().get::<Facing>(pulsar).unwrap();
        assert!((facing.0 - std::f32::consts::PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_ship_takes_damage_in_beam() {
        let mut app = App::new();

        // Arrange
        let _pulsar = app.world_mut().spawn((
            Pulsar {
                rotation_period: 100.0,
                beam_width: std::f32::consts::PI / 4.0,
            },
            Position(Vec2::new(0.0, 0.0)),
            Facing(0.0), // Pointing right
        )).id();

        let mut ship1 = Ship::new(ShipType::Scout);
        ship1.health = 100.0;
        let ship_in_beam = app.world_mut().spawn((
            ship1,
            Position(Vec2::new(10.0, 0.0)), // Right
        )).id();

        let mut ship2 = Ship::new(ShipType::Scout);
        ship2.health = 100.0;
        let ship_out_beam = app.world_mut().spawn((
            ship2,
            Position(Vec2::new(0.0, 10.0)), // Up
        )).id();

        app.insert_resource(SimulationTime { speed: SimSpeed::Normal, ..Default::default() });
        app.add_systems(Update, pulsar_radiation_damage_system);

        // Act - 10 ticks
        for _ in 0..10 {
            app.update();
        }

        // Assert
        let ship_in = app.world().get::<Ship>(ship_in_beam).unwrap();
        assert!(ship_in.health < 100.0, "Ship in beam should take damage");

        let ship_out = app.world().get::<Ship>(ship_out_beam).unwrap();
        assert_eq!(ship_out.health, 100.0, "Ship out of beam should not take damage");
    }
}
