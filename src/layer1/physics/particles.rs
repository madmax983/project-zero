//! Sub-Grid Particle Effects and "Juice".
//!
//! This module handles purely visual, temporary entities called `Particle`s.
//! Particles are used to add "Juice" to the game—explosions, confetti, sparks, and debris.
//!
//! Unlike standard grid-locked entities, particles support sub-grid movement. They accumulate
//! fractional velocity (`dx`, `dy`) and only move across the integer grid when the accumulator
//! reaches `1.0`.
//!
//! # Physics
//! - **Gravity:** Particles with a `ParticleVelocity` are constantly pulled downwards.
//! - **Friction:** Particles naturally slow down over time, simulating air resistance.
//! - **Lifetime:** Every particle has a `lifetime` in ticks. When it hits `0`, it is despawned.
//!
//! # Examples
//!
//! ```
//! use bevy_ecs::prelude::*;
//! use scale::layer1::physics::particles::{Particle, ParticleVelocity, ParticleAccumulator, particle_physics_system, particle_system, spawn_moving_particle};
//! use scale::layer1::map::GridPosition;
//! use ratatui::style::Color;
//!
//! let mut world = World::new();
//!
//! // 1. Spawn a moving particle (e.g., a spark)
//! spawn_moving_particle(
//!     &mut world,
//!     GridPosition { x: 5, y: 5 },
//!     '*',
//!     Color::Yellow,
//!     10,   // Lifetime of 10 ticks
//!     2.0,  // Move 2 tiles right per tick
//!     0.0,  // No initial vertical movement
//! );
//!
//! // 2. Run the physics and lifetime systems
//! let mut schedule = Schedule::default();
//! schedule.add_systems((particle_physics_system, particle_system));
//! schedule.run(&mut world);
//!
//! // 3. The particle moved right, slowed down via friction, and lost 1 lifetime tick
//! let mut query = world.query::<(&GridPosition, &Particle)>();
//! let (pos, particle) = query.single(&world);
//!
//! assert_eq!(pos.x, 7); // Moved 2 tiles
//! assert_eq!(particle.lifetime, 9); // Lost 1 tick
//! ```
#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;
use rand::prelude::SliceRandom;
use rand::Rng;
use ratatui::style::Color;

/// Ludwig's Tuning Constants
/// Gravity applied to particles per tick to pull them downwards.
const GRAVITY: f32 = 0.05;

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

        // Apply gravity
        vel.dy += GRAVITY;

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
        let color = *colors.choose(&mut rng).unwrap_or(&Color::White);
        let char = *chars.choose(&mut rng).unwrap_or(&'*');
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
        world.run_system_once(particle_physics_system).expect("Component should exist or System should run");

        let pos = world.get::<GridPosition>(entity).expect("Component should exist or System should run");
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);

        let acc = world.get::<ParticleAccumulator>(entity).expect("Component should exist or System should run");
        assert!((acc.x - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_particle_system_despawns_expired_particles() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Particle {
                    char: '.',
                    color: Color::White,
                    lifetime: 1,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Tick 1: lifetime 1 -> 0
        world.run_system_once(particle_system).expect("Component should exist or System should run");

        let particle = world.get::<Particle>(entity).expect("Component should exist or System should run");
        assert_eq!(particle.lifetime, 0);

        // Tick 2: lifetime 0 -> despawn
        world.run_system_once(particle_system).expect("Component should exist or System should run");

        assert!(world.get::<Particle>(entity).is_none());
    }

    #[test]
    fn test_spawn_confetti_no_panic() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };

        spawn_confetti(&mut world, pos);

        let mut query = world.query::<&Particle>();
        let particle_count = query.iter(&world).count();

        assert_eq!(particle_count, 30);
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
        world.run_system_once(particle_physics_system).expect("Component should exist or System should run");

        let vel = world.get::<ParticleVelocity>(entity).expect("Component should exist or System should run");
        assert!((vel.dx - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_particle_physics_gravity() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Particle {
                    char: '.',
                    color: Color::White,
                    lifetime: 10,
                },
                GridPosition { x: 0, y: 0 },
                ParticleVelocity { dx: 0.0, dy: 0.0 },
                ParticleAccumulator::default(),
            ))
            .id();

        world.run_system_once(particle_physics_system).expect("Component should exist or System should run");

        let vel = world.get::<ParticleVelocity>(entity).expect("Component should exist or System should run");
        assert!((vel.dy - 0.045).abs() < 0.001); // (0.0 + GRAVITY(0.05)) * 0.9 = 0.045
    }
}
