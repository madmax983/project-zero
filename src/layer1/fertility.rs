use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::terrain::TerrainType;
use crate::layer1::utility_ai::ActionType;
use crate::layer1::utility_ai::PopAction;
use crate::layer1::TerrainGrid;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

const DEPLETION_RATE: f32 = 0.0001; // 1% every 100 ticks
const REGEN_RATE: f32 = 0.00005; // 0.5% every 100 ticks

/// Grid tracking soil fertility (0.0 to 1.0).
#[derive(Resource, Default)]
pub struct FertilityGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Fertility values (0.0 to 1.0), row-major.
    pub values: Vec<f32>,
}

impl FertilityGrid {
    /// Create a new fertility grid with default 1.0 values.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![1.0; width * height],
        }
    }

    /// Initialize fertility based on terrain types.
    #[must_use]
    pub fn from_terrain(terrain: &TerrainGrid) -> Self {
        let values = terrain
            .tiles
            .iter()
            .map(|t| match t {
                TerrainType::Grass | TerrainType::Tree => 1.0,
                TerrainType::Dirt => 0.8,
                _ => 0.0,
            })
            .collect();

        Self {
            width: terrain.width,
            height: terrain.height,
            values,
        }
    }

    /// Get fertility at (x, y). Returns 0.0 if out of bounds.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.values[y * self.width + x]
        } else {
            0.0
        }
    }

    /// Set fertility at (x, y). Clamps between 0.0 and 1.0.
    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.values[y * self.width + x] = value.clamp(0.0, 1.0);
        }
    }

    /// Modify fertility at (x, y) by delta. Clamps result.
    pub fn modify(&mut self, x: usize, y: usize, delta: f32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.values[idx] = (self.values[idx] + delta).clamp(0.0, 1.0);
        }
    }
}

/// System to deplete fertility on farmed tiles and regenerate fallow ones.
pub fn update_fertility_system(
    mut grid: ResMut<FertilityGrid>,
    pop_query: Query<(&PopAction, &GridPosition), With<Pop>>,
) {
    // 1. Identify farmed tiles
    let mut farmed_tiles = HashSet::new();
    for (action, pos) in &pop_query {
        if action.current == ActionType::Farm {
            farmed_tiles.insert((pos.x, pos.y));
        }
    }

    // 2. Update grid
    let width = grid.width;
    let height = grid.height;

    for y in 0..height {
        for x in 0..width {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let is_farmed = farmed_tiles.contains(&(x as i32, y as i32));

            if is_farmed {
                grid.modify(x, y, -DEPLETION_RATE);
            } else {
                // Only regenerate if it has some base fertility potential.
                // Since we don't have access to TerrainGrid here easily without another Res,
                // we can assume if current > 0.0 it is soil.
                // However, if it was depleted to 0.0 completely, it might never recover.
                // But from_terrain sets non-soil to 0.0.
                // So checking > 0.0 is a reasonable heuristic,
                // UNLESS it reached exactly 0.0 but is soil.
                // For now, let's assume we don't deplete to exactly 0.0 usually,
                // or if we do, it's dead soil.
                // Spec Refactor Phase suggests looking up TerrainType.
                // For Green Phase, > 0.0 check is fine.
                // Wait, if I start with 0.5 (test case), it should regen.

                if grid.get(x, y) > 0.0 {
                    grid.modify(x, y, REGEN_RATE);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::farm::{produce_food_system, Farm};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_fertility_grid_initialization() {
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        let fertility = FertilityGrid::from_terrain(&terrain);

        assert_eq!(fertility.width, 10);
        assert_eq!(fertility.height, 10);
        assert_eq!(fertility.get(0, 0), 1.0); // Grass = 100%
    }

    #[test]
    fn test_fertility_values_by_terrain() {
        let mut tiles = vec![TerrainType::Grass; 4];
        tiles[0] = TerrainType::Grass;
        tiles[1] = TerrainType::Dirt;
        tiles[2] = TerrainType::Rock;
        tiles[3] = TerrainType::Water;

        let terrain = TerrainGrid {
            width: 2,
            height: 2,
            tiles,
        };
        let fertility = FertilityGrid::from_terrain(&terrain);

        assert!((fertility.get(0, 0) - 1.0).abs() < f32::EPSILON); // Grass
        assert!((fertility.get(1, 0) - 0.8).abs() < f32::EPSILON); // Dirt
        assert_eq!(fertility.get(0, 1), 0.0); // Rock
        assert_eq!(fertility.get(1, 1), 0.0); // Water
    }

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<crate::layer1::eureka::EurekaEvent>>();
        world
    }

    #[test]
    fn test_production_scales_with_fertility() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::pop::Pop;
        use crate::layer1::utility_ai::{ActionType, PopAction};

        let mut world = setup_test_world();
        world.insert_resource(ColonyResources {
            food: 0.0,
            ..Default::default()
        });

        // Setup fertility grid with 50% fertility at (0,0)
        let mut fertility = FertilityGrid::new(10, 10);
        fertility.set(0, 0, 0.5);
        world.insert_resource(fertility);

        // Spawn farm at (0,0)
        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn worker
        world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        // Run production
        world.run_system_once(produce_food_system).unwrap();

        // Expected: Base (Wheat 0.006) * Fertility (0.5) = 0.003
        let food = world.resource::<ColonyResources>().food;
        assert!((food - 0.003).abs() < 0.0001, "Food produced: {}", food);
    }

    #[test]
    fn test_farming_depletes_fertility() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::pop::Pop;
        use crate::layer1::utility_ai::{ActionType, PopAction};

        let mut world = World::new();
        let mut fertility = FertilityGrid::new(10, 10);
        fertility.set(0, 0, 1.0);
        world.insert_resource(fertility);

        // Spawn active farm
        world.spawn((
            Farm::default(),
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn worker to make it active
        world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            PopAction {
                current: ActionType::Farm,
                ..Default::default()
            },
        ));

        world.run_system_once(update_fertility_system).unwrap();

        let new_fertility = world.resource::<FertilityGrid>().get(0, 0);
        assert!(new_fertility < 1.0);
        assert!(new_fertility > 0.9); // Shouldn't deplete instantly
    }

    #[test]
    fn test_fallow_land_regenerates() {
        let mut world = World::new();
        let mut fertility = FertilityGrid::new(10, 10);
        fertility.set(0, 0, 0.5); // Depleted
        world.insert_resource(fertility);

        // No farm or empty farm at (0,0)

        world.run_system_once(update_fertility_system).unwrap();

        let new_fertility = world.resource::<FertilityGrid>().get(0, 0);
        assert!(new_fertility > 0.5);
    }

    #[test]
    fn test_fertility_clamping() {
        let mut grid = FertilityGrid::new(1, 1);

        grid.set(0, 0, 1.5);
        assert_eq!(grid.get(0, 0), 1.0); // Max 1.0

        grid.set(0, 0, -0.5);
        assert_eq!(grid.get(0, 0), 0.0); // Min 0.0
    }
}
