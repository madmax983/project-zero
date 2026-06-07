use bevy::prelude::*;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};

#[derive(Component)]
pub struct LostTech;

#[derive(Resource, Default)]
pub struct VerticalExtension {
    stump_positions: Vec<(usize, usize)>,
}

impl VerticalExtension {
    pub fn new() -> Self {
        Self {
            stump_positions: Vec::new(),
        }
    }

    pub fn mark_stump(&mut self, x: usize, y: usize) {
        self.stump_positions.push((x, y));
    }

    pub fn is_stump(&self, x: usize, y: usize) -> bool {
        self.stump_positions.contains(&(x, y))
    }

    pub fn get_pressure(&self, z: usize) -> f32 {
        (1.0 - (z as f32 / 100.0)).clamp(0.0, 1.0).powf(2.0)
    }
}

pub fn generate_tether_stump(mut commands: Commands, mut terrain: ResMut<TerrainGrid>) {
    let center_x = terrain.width / 2;
    let center_y = terrain.height / 2;

    let mut extensions = VerticalExtension::new();

    // Create a 2x2 stump
    for x in center_x..center_x+2 {
        for y in center_y..center_y+2 {
            terrain.set(x, y, TerrainType::IndestructibleStump);
            extensions.mark_stump(x, y);
        }
    }

    commands.insert_resource(extensions);
}

pub fn initialize_atmosphere_gradient(_extensions: &mut VerticalExtension) {
    // This is essentially a no-op as the gradient logic is evaluated lazily in get_pressure
}

pub fn spawn_lost_tech_caches(mut commands: Commands, terrain: Res<TerrainGrid>) {
    let (stump_x, stump_y) = find_stump_center(&terrain).unwrap_or((50, 50));

    // Spawn one piece of tech high up
    commands.spawn((
        LostTech,
        Transform::from_xyz(stump_x as f32, stump_y as f32, 50.0),
    ));
}

// Helper for minimal implementation
pub fn find_stump_center(terrain: &TerrainGrid) -> Option<(usize, usize)> {
    for x in 0..terrain.width {
        for y in 0..terrain.height {
            if terrain.get(x, y) == Some(TerrainType::IndestructibleStump) {
                return Some((x, y));
            }
        }
    }
    None
}
