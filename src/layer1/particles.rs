use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;
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

/// Helper to spawn a new particle.
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
