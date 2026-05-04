//! Echoing Footsteps (Nova Feature).
//!
//! # The Spark
//! Pops moving around the base don't generate noise, only buildings do. What if busy hallways became naturally noisy?
//!
//! # The Feature
//! The `EchoingFootsteps` system tracks Pop movement. When a Pop moves to a new tile, they spawn an `AcousticEcho` (a temporary `NoiseSource`) on their previous tile. These echoes fade over time.
//! This creates dynamic, emergent noise pollution based on colony traffic patterns, forcing players to design wider corridors or route traffic away from sleeping areas.

use crate::layer1::entities::pop::Pop;
use crate::layer1::map::GridPosition;
use crate::layer1::physics::acoustic::NoiseSource;
use bevy_ecs::prelude::*;

/// Component tracking the last known position of a Pop to detect movement.
#[derive(Component)]
pub struct LastPosition(pub GridPosition);

/// Component representing a fading sound echo left by movement.
#[derive(Component)]
pub struct AcousticEcho {
    pub life_remaining: u32,
}

const ECHO_STARTING_LIFE: u32 = 5;
const ECHO_STARTING_INTENSITY: f32 = 0.5;
const ECHO_RADIUS: f32 = 3.0;

/// System that detects movement and spawns acoustic echoes.
pub fn spawn_echoes_system(
    mut commands: Commands,
    mut query: Query<(Entity, &GridPosition, Option<&mut LastPosition>), With<Pop>>,
) {
    for (entity, current_pos, mut opt_last_pos) in query.iter_mut() {
        if let Some(ref mut last_pos) = opt_last_pos {
            if *current_pos != last_pos.0 {
                // Pop moved! Spawn an echo at the old position.
                commands.spawn((
                    AcousticEcho {
                        life_remaining: ECHO_STARTING_LIFE,
                    },
                    last_pos.0,
                    NoiseSource {
                        radius: ECHO_RADIUS,
                        intensity: ECHO_STARTING_INTENSITY,
                    },
                ));

                // Update last position
                last_pos.0 = *current_pos;
            }
        } else {
            // First time tracking this pop, just initialize the component.
            commands.entity(entity).insert(LastPosition(*current_pos));
        }
    }
}

/// System that fades existing echoes and despawns them when silent.
pub fn fade_echoes_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AcousticEcho, &mut NoiseSource)>,
) {
    for (entity, mut echo, mut noise) in query.iter_mut() {
        if echo.life_remaining > 0 {
            echo.life_remaining -= 1;
            // Linearly fade the intensity
            #[allow(clippy::cast_precision_loss)]
            let fraction = echo.life_remaining as f32 / ECHO_STARTING_LIFE as f32;
            noise.intensity = ECHO_STARTING_INTENSITY * fraction;
        }

        if echo.life_remaining == 0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((spawn_echoes_system, fade_echoes_system));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_echo_spawns_on_movement() {
        let mut world = World::new();

        // Spawn a pop at 0,0
        let pop = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();

        // Run once to initialize LastPosition
        world.run_system_once(spawn_echoes_system).unwrap();

        assert!(world.get::<LastPosition>(pop).is_some());
        assert_eq!(world.query::<&AcousticEcho>().iter(&world).count(), 0);

        // Move the pop
        world.get_mut::<GridPosition>(pop).unwrap().x = 1;

        // Run again to spawn echo
        world.run_system_once(spawn_echoes_system).unwrap();

        // Verify echo exists at old position
        let mut echo_query = world.query::<(&AcousticEcho, &GridPosition, &NoiseSource)>();
        let mut echo_count = 0;
        for (echo, pos, noise) in echo_query.iter(&world) {
            echo_count += 1;
            assert_eq!(pos.x, 0); // Old pos
            assert_eq!(pos.y, 0);
            assert_eq!(echo.life_remaining, ECHO_STARTING_LIFE);
            assert!((noise.intensity - ECHO_STARTING_INTENSITY).abs() < f32::EPSILON);
        }
        assert_eq!(echo_count, 1);

        // Verify LastPosition updated
        assert_eq!(world.get::<LastPosition>(pop).unwrap().0.x, 1);
    }

    #[test]
    fn test_echo_fades_and_despawns() {
        let mut world = World::new();

        let echo_entity = world
            .spawn((
                AcousticEcho {
                    life_remaining: 1, // Will fade to 0
                },
                NoiseSource {
                    radius: ECHO_RADIUS,
                    intensity: 0.1,
                },
            ))
            .id();

        world.run_system_once(fade_echoes_system).unwrap();

        // Should be despawned
        assert!(world.get_entity(echo_entity).is_err());
    }
}
