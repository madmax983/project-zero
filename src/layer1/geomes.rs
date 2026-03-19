use crate::layer1::map::GridPosition;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeomeType {
    SunlessSea,
    MagmaRiver,
    CrystalForest,
    SporeCavern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZLevel(pub i32);

pub struct Rect {
    pub min: GridPosition,
    pub max: GridPosition,
}

impl Rect {
    pub fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Rect {
            min: GridPosition { x: min_x, y: min_y },
            max: GridPosition { x: max_x, y: max_y },
        }
    }
}

#[derive(Resource)]
pub struct GeomeManager {
    regions: Vec<(GeomeType, ZLevel, Rect)>,
}

impl Default for GeomeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GeomeManager {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    pub fn spawn_geome(&mut self, geome_type: GeomeType, z: ZLevel, area: Rect) {
        self.regions.push((geome_type, z, area));
    }

    pub fn apply_to_grid(&self, grid: &mut TerrainGrid) {
        // Since TerrainGrid is currently 2D, we apply geomes to the 2D grid
        // based on the area's X and Y bounds, ignoring ZLevel for now.
        for (g_type, _z, area) in &self.regions {
            // Note: Since x and y can be negative in GridPosition, but TerrainGrid uses usize,
            // we safely convert them, ignoring negative values.
            for y in area.min.y..=area.max.y {
                for x in area.min.x..=area.max.x {
                    if x >= 0 && y >= 0 {
                        let ux = x as usize;
                        let uy = y as usize;
                        let terrain = match g_type {
                            GeomeType::MagmaRiver => TerrainType::MagmaRock,
                            GeomeType::SporeCavern => TerrainType::SporeBloom,
                            _ => TerrainType::DeepRock,
                        };
                        grid.set(ux, uy, terrain);
                    }
                }
            }
        }
    }
}

/// A hazard present on a specific tile (e.g., Toxic Spores, Extreme Heat).
#[derive(Component, Debug, Clone, Copy)]
pub struct GeomeHazard {
    pub damage_per_tick: f32,
}

/// Diffuses hazards from breached geome tiles into adjacent empty/walkable tiles.
pub fn diffuse_geome_hazards_system(
    mut commands: Commands,
    grid: Res<TerrainGrid>,
    hazard_query: Query<&GridPosition, With<GeomeHazard>>,
) {
    // Collect existing hazard positions to avoid spawning multiple hazards on the same tile
    let mut existing_hazards = std::collections::HashSet::new();
    for pos in hazard_query.iter() {
        existing_hazards.insert(*pos);
    }

    let width = grid.width as i32;
    let height = grid.height as i32;

    for y in 0..height {
        for x in 0..width {
            let pos = GridPosition { x, y };
            if let Some(terrain) = grid.get(x as usize, y as usize) {
                // If this is a SporeBloom tile, it should spawn a hazard
                if terrain == TerrainType::SporeBloom && !existing_hazards.contains(&pos) {
                    commands.spawn((
                        GeomeHazard {
                            damage_per_tick: 5.0,
                        },
                        pos,
                    ));
                    existing_hazards.insert(pos);
                }

                // If this is a MagmaRock tile, it should spawn a hazard
                if terrain == TerrainType::MagmaRock && !existing_hazards.contains(&pos) {
                    commands.spawn((
                        GeomeHazard {
                            damage_per_tick: 10.0,
                        },
                        pos,
                    ));
                    existing_hazards.insert(pos);
                }
            }
        }
    }
}

/// Applies damage to entities with Health that are standing on GeomeHazards.
pub fn environmental_damage_system(
    mut health_query: Query<(&GridPosition, &mut crate::layer1::health::Health)>,
    hazard_query: Query<(&GridPosition, &GeomeHazard)>,
) {
    let mut hazard_map = std::collections::HashMap::new();
    for (pos, hazard) in hazard_query.iter() {
        *hazard_map.entry(*pos).or_insert(0.0) += hazard.damage_per_tick;
    }

    for (health_pos, mut health) in health_query.iter_mut() {
        if let Some(damage) = hazard_map.get(health_pos) {
            health.take_damage(*damage);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;

    #[test]
    fn test_deep_crust_geome_generation() {
        let mut world = World::new();

        let mut grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Rock; 100],
        };

        let mut geome_manager = GeomeManager::new();
        // Spawn Magma River geome at an area in the 2D grid
        geome_manager.spawn_geome(GeomeType::MagmaRiver, ZLevel(-4), Rect::new(2, 2, 5, 5));
        geome_manager.apply_to_grid(&mut grid);
        world.insert_resource(grid);

        let grid = world.get_resource::<TerrainGrid>().unwrap();
        // Should be converted to MagmaRock inside the rect
        assert_eq!(grid.get(3, 3), Some(TerrainType::MagmaRock));
        // Outside should remain Rock
        assert_eq!(grid.get(1, 1), Some(TerrainType::Rock));
    }

    #[test]
    fn test_breaching_geome_triggers_hazard() {
        let mut world = World::new();

        let mut grid = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Rock; 100],
        };
        // Setup a SporeBloom tile
        grid.set(6, 5, TerrainType::SporeBloom);
        world.insert_resource(grid);

        let miner = world
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                GridPosition { x: 6, y: 5 }, // Standing on the SporeBloom tile
            ))
            .id();

        // Run hazard diffusion system to spawn the hazard entity
        let mut hazard_schedule = Schedule::default();
        hazard_schedule.add_systems(diffuse_geome_hazards_system);
        hazard_schedule.run(&mut world);

        // Run environmental damage system to apply damage
        let mut damage_schedule = Schedule::default();
        damage_schedule.add_systems(environmental_damage_system);
        damage_schedule.run(&mut world);

        // The miner should have taken damage from the Spore hazard
        let health = world.get::<Health>(miner).unwrap();
        assert!(
            health.current < 100.0,
            "Miner should take damage from breached geome hazard"
        );
    }
}
