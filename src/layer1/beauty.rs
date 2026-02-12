#![allow(clippy::collapsible_if)]
use bevy_ecs::prelude::*;

/// Component indicating an entity emits beauty (positive or negative).
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct BeautySource {
    /// The amount of beauty emitted.
    pub value: f32,
    /// The radius of effect (currently unused, defaults to 0.0/single tile).
    pub radius: f32,
}

/// Grid storing beauty values for the map.
#[derive(Resource, Default)]
pub struct BeautyGrid {
    /// Width of the grid.
    pub width: usize,
    /// Height of the grid.
    pub height: usize,
    /// Flattened vector of beauty values.
    pub values: Vec<f32>,
}

impl BeautyGrid {
    /// Create a new beauty grid.
    #[must_use]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }
    /// Get beauty value at position.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        self.values[y * self.width + x]
    }
    /// Set beauty value at position.
    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.values[y * self.width + x] = val;
    }

    /// Clear the grid.
    pub fn clear(&mut self) {
        self.values.fill(0.0);
    }
}

/// System to update the beauty grid based on sources.
pub fn update_beauty_grid_system(
    mut grid: ResMut<BeautyGrid>,
    terrain: Res<crate::layer1::TerrainGrid>,
    sources: Query<(&crate::layer1::GridPosition, &BeautySource)>,
    items: Query<(
        &crate::layer1::GridPosition,
        &crate::layer1::resources::ResourceItem,
    )>,
) {
    grid.clear();

    // Terrain Beauty
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            if let Some(tile) = terrain.get(x, y) {
                let mod_val = match tile {
                    crate::layer1::TerrainType::Grass => 1.0f32,
                    crate::layer1::TerrainType::Dirt => -1.0f32,
                    crate::layer1::TerrainType::Path => -2.0f32,
                    _ => 0.0f32,
                };
                if mod_val.abs() > f32::EPSILON {
                    let current = grid.get(x, y);
                    grid.set(x, y, current + mod_val);
                }
            }
        }
    }

    for (pos, source) in &sources {
        let value = source.value;
        #[allow(clippy::collapsible_if)]
        if value.abs() > f32::EPSILON {
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                let current = grid.get(x, y);
                grid.set(x, y, current + value);
            }
        }
    }

    for (pos, item) in &items {
        if item.resource_type == crate::layer1::resources::ResourceType::Waste {
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                let current = grid.get(x, y);
                grid.set(x, y, current - 5.0);
            }
        }
    }
}

/// System to apply beauty effects to pops.
pub fn apply_beauty_effects_system(
    grid: Res<BeautyGrid>,
    mut pops: Query<
        (
            &crate::layer1::GridPosition,
            &mut crate::layer1::needs::Needs,
        ),
        With<crate::layer1::pop::Pop>,
    >,
) {
    for (pos, mut needs) in &mut pops {
        if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
            let beauty = grid.get(x, y);
            if beauty > 0.0 {
                // Boost leisure
                needs.leisure = beauty.mul_add(0.001, needs.leisure).min(1.0);
            } else if beauty < 0.0 {
                // Decay leisure
                needs.leisure = beauty.mul_add(0.001, needs.leisure).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    // 1. Beauty Grid
    #[test]
    fn test_beauty_grid_resource_defaults() {
        // Need a new resource for BeautyGrid
        let grid = crate::layer1::beauty::BeautyGrid::new(10, 10);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_beauty_emission_from_source() {
        let mut world = World::new();
        // Setup grid
        world.insert_resource(crate::layer1::beauty::BeautyGrid::new(10, 10));
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });

        // Spawn entity with BeautySource
        world.spawn((
            BeautySource {
                value: 5.0,
                radius: 0.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system to update grid
        let _ = world.run_system_once(crate::layer1::beauty::update_beauty_grid_system);

        let grid = world.resource::<crate::layer1::beauty::BeautyGrid>();
        // 5.0 from source + 1.0 from Grass
        assert_eq!(grid.get(5, 5), 6.0);
    }

    #[test]
    fn test_negative_beauty_from_trash() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::beauty::BeautyGrid::new(10, 10));
        world.insert_resource(crate::layer1::terrain::TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![crate::layer1::terrain::TerrainType::Grass; 100],
        });

        let mut grid = world.resource_mut::<crate::layer1::beauty::BeautyGrid>();
        grid.set(0, 0, -5.0);

        assert_eq!(grid.get(0, 0), -5.0);
    }

    // 2. Morale Impact
    #[test]
    fn test_beauty_satisfies_leisure() {
        let mut world = World::new();
        let mut grid = crate::layer1::beauty::BeautyGrid::new(10, 10);
        grid.set(0, 0, 10.0); // High beauty
        world.insert_resource(grid);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.1,
                    ..Default::default()
                }, // Low leisure
            ))
            .id();

        // Run system
        let _ = world.run_system_once(crate::layer1::beauty::apply_beauty_effects_system);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.1, "High beauty should increase leisure");
    }

    #[test]
    fn test_negative_beauty_decreases_leisure() {
        let mut world = World::new();
        let mut grid = crate::layer1::beauty::BeautyGrid::new(10, 10);
        grid.set(0, 0, -10.0); // Disgusting
        world.insert_resource(grid);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        let _ = world.run_system_once(crate::layer1::beauty::apply_beauty_effects_system);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure < 0.5,
            "Negative beauty should decrease leisure"
        );
    }
}
