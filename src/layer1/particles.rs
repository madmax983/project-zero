#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;
use rand::prelude::SliceRandom;
use rand::Rng;
use ratatui::style::Color;

/// Visual particle effect component.
///
/// Particles are temporary entities used for visual feedback ("Juice").
/// They have a character representation, color, and a lifetime in ticks.
/// Once the lifetime reaches zero, the particle is despawned.
#[derive(Component, Debug, Clone, Copy)]
pub struct Particle {
    /// The character to render.
    pub char: char,
    /// The color of the particle.
    pub color: Color,
    /// Remaining lifetime in ticks.
    pub lifetime: u32,
}

/// Velocity for moving particles.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ParticleVelocity {
    /// Velocity in X direction (tiles per tick).
    pub dx: f32,
    /// Velocity in Y direction (tiles per tick).
    pub dy: f32,
}

/// Accumulator for sub-grid movement.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ParticleAccumulator {
    /// Accumulated sub-grid movement in X.
    pub x: f32,
    /// Accumulated sub-grid movement in Y.
    pub y: f32,
}

/// System to update particle lifetimes and despawn expired ones.
pub fn particle_system(mut commands: Commands, mut query: Query<(Entity, &mut Particle)>) {
    for (entity, mut particle) in &mut query {
        if particle.lifetime > 0 {
            particle.lifetime -= 1;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

/// System to handle sub-grid movement for particles.
pub fn particle_physics_system(
    mut query: Query<(
        &mut GridPosition,
        &mut ParticleVelocity,
        &mut ParticleAccumulator,
    )>,
) {
    for (mut pos, mut vel, mut acc) in &mut query {
        acc.x += vel.dx;
        acc.y += vel.dy;

        // Apply friction to slow down particles naturally
        vel.dx *= 0.9;
        vel.dy *= 0.9;

        // X movement
        if acc.x.abs() >= 1.0 {
            // trunc() rounds towards zero (1.5 -> 1.0, -1.5 -> -1.0)
            let move_x = acc.x.trunc() as i32;
            pos.x += move_x;
            acc.x -= move_x as f32;
        }

        // Y movement
        if acc.y.abs() >= 1.0 {
            let move_y = acc.y.trunc() as i32;
            pos.y += move_y;
            acc.y -= move_y as f32;
        }
    }
}

/// Helper to spawn a new static particle.
pub fn spawn_particle(
    world: &mut World,
    pos: GridPosition,
    char: char,
    color: Color,
    lifetime: u32,
) {
    world.spawn((
        Particle {
            char,
            color,
            lifetime,
        },
        pos,
    ));
}

/// Helper to spawn a moving particle.
pub fn spawn_moving_particle(
    world: &mut World,
    pos: GridPosition,
    char: char,
    color: Color,
    lifetime: u32,
    dx: f32,
    dy: f32,
) {
    world.spawn((
        Particle {
            char,
            color,
            lifetime,
        },
        pos,
        ParticleVelocity { dx, dy },
        ParticleAccumulator::default(),
    ));
}

/// Helper to spawn confetti particles for celebrations.
pub fn spawn_confetti(world: &mut World, pos: GridPosition) {
    let mut rng = rand::thread_rng();
    let colors = [
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Yellow,
        Color::Magenta,
        Color::Cyan,
        Color::White,
    ];
    let chars = ['*', '.', '+', 'x', 'o'];

    for _ in 0..30 {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(0.5..1.5);
        let dx = angle.cos() * speed;
        let dy = angle.sin() * speed;
        let color = *colors.choose(&mut rng).unwrap();
        let char = *chars.choose(&mut rng).unwrap();
        let lifetime = rng.gen_range(20..40);

        spawn_moving_particle(world, pos, char, color, lifetime, dx, dy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_particle_physics_movement() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Particle {
                    char: '.',
                    color: Color::White,
                    lifetime: 10,
                },
                GridPosition { x: 0, y: 0 },
                ParticleVelocity { dx: 1.5, dy: 0.0 }, // Move 1.5 per tick
                ParticleAccumulator::default(),
            ))
            .id();

        // Tick 1
        // Acc += 1.5 -> 1.5. Pos += 1. Acc -> 0.5. Friction applied.
        world.run_system_once(particle_physics_system).unwrap();

        let pos = world.get::<GridPosition>(entity).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);

        let acc = world.get::<ParticleAccumulator>(entity).unwrap();
        assert!((acc.x - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_particle_physics_friction() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Particle {
                    char: '.',
                    color: Color::White,
                    lifetime: 10,
                },
                GridPosition { x: 0, y: 0 },
                ParticleVelocity { dx: 1.0, dy: 0.0 },
                ParticleAccumulator::default(),
            ))
            .id();

        // Tick 1
        // Acc += 1.0 -> 1.0. Pos += 1. Acc -> 0.0. Vel *= 0.9 -> 0.9.
        world.run_system_once(particle_physics_system).unwrap();

        let vel = world.get::<ParticleVelocity>(entity).unwrap();
        assert!((vel.dx - 0.9).abs() < 0.001);
    }
}
