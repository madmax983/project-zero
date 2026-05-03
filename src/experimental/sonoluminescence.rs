//! Sonoluminescence mechanic.
//!
//! Converts acoustic noise into visible light.

use crate::layer1::lighting::LightSource;
use crate::layer1::map::GridPosition;
use crate::layer1::physics::acoustic::NoiseMap;
use bevy_ecs::prelude::*;

/// Component designating an entity as a Sonoluminescent Node.
/// It converts ambient noise from the `NoiseMap` into light.
#[derive(Component, Default)]
pub struct SonoluminescentNode {
    /// The conversion efficiency. A value of 1.0 means 1.0 noise = 1.0 intensity.
    pub efficiency: f32,
}

/// System to update the light emitted by Sonoluminescent Nodes based on ambient noise.
pub fn sonoluminescence_system(
    noise_map: Res<NoiseMap>,
    mut nodes: Query<(&GridPosition, &SonoluminescentNode, &mut LightSource)>,
) {
    for (pos, node, mut light) in &mut nodes {
        let ambient_noise = noise_map.get(pos.x, pos.y);

        // Convert noise to light
        let new_intensity = ambient_noise * node.efficiency;

        light.intensity = new_intensity;

        // Dynamically scale radius based on intensity, base radius 1.0, max radius 5.0
        if new_intensity > 0.05 {
            light.radius = (new_intensity * 5.0).max(1.0);
        } else {
            light.radius = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_sonoluminescence_conversion() {
        let mut world = World::new();

        // Setup NoiseMap with high noise at (5, 5)
        let mut noise = NoiseMap::new(10, 10);
        noise.set(5, 5, 0.8);
        world.insert_resource(noise);

        // Spawn a node
        let entity = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                SonoluminescentNode { efficiency: 2.0 },
                LightSource {
                    radius: 0.0,
                    intensity: 0.0,
                    color: (0, 255, 255),
                    is_outdoor: false,
                },
            ))
            .id();

        world.run_system_once(sonoluminescence_system).unwrap();

        let light = world.get::<LightSource>(entity).unwrap();

        // 0.8 noise * 2.0 efficiency = 1.6 intensity
        assert!((light.intensity - 1.6).abs() < f32::EPSILON);
        assert!(light.radius > 1.0);
    }

    #[test]
    fn test_sonoluminescence_silence() {
        let mut world = World::new();

        // Setup NoiseMap with 0 noise at (2, 2)
        let mut noise = NoiseMap::new(10, 10);
        noise.set(2, 2, 0.0);
        world.insert_resource(noise);

        // Spawn a node
        let entity = world
            .spawn((
                GridPosition { x: 2, y: 2 },
                SonoluminescentNode { efficiency: 1.0 },
                LightSource {
                    radius: 5.0, // Start with radius to ensure it gets zeroed
                    intensity: 1.0,
                    color: (0, 255, 255),
                    is_outdoor: false,
                },
            ))
            .id();

        world.run_system_once(sonoluminescence_system).unwrap();

        let light = world.get::<LightSource>(entity).unwrap();

        // 0.0 noise = 0.0 intensity and 0.0 radius
        assert!((light.intensity - 0.0).abs() < f32::EPSILON);
        assert!((light.radius - 0.0).abs() < f32::EPSILON);
    }
}
