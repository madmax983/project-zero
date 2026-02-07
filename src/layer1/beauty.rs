use bevy_ecs::prelude::*;

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

/// System to update the beauty grid based on buildings.
pub fn update_beauty_grid_system(
    mut grid: ResMut<BeautyGrid>,
    buildings: Query<(
        &crate::layer1::GridPosition,
        &crate::layer1::building::Building,
    )>,
) {
    grid.clear();

    for (pos, building) in &buildings {
        let value = building.building_type.beauty_value();
        #[allow(clippy::collapsible_if)]
        if value.abs() > f32::EPSILON {
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                let current = grid.get(x, y);
                grid.set(x, y, current + value);
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
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    // 1. Beauty Grid
    #[test]
    fn test_beauty_grid_resource_defaults() {
        // Need a new resource for BeautyGrid
        let grid = crate::layer1::beauty::BeautyGrid::new(10, 10);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_beauty_emission_from_buildings() {
        let mut world = World::new();
        // Setup grid
        world.insert_resource(crate::layer1::beauty::BeautyGrid::new(10, 10));

        // Spawn FlowerBed (Beauty +5)
        world.spawn((
            Building {
                building_type: BuildingType::FlowerBed,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system to update grid
        let _ = world.run_system_once(crate::layer1::beauty::update_beauty_grid_system);

        let grid = world.resource::<crate::layer1::beauty::BeautyGrid>();
        assert_eq!(grid.get(5, 5), 5.0);

        // Check falloff? (Optional for MVP, maybe just local tile)
        // Let's assume simple 1-tile radius for MVP
        assert_eq!(grid.get(4, 5), 0.0);
    }

    #[test]
    fn test_negative_beauty_from_trash() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::beauty::BeautyGrid::new(10, 10));

        // Spawn Trash (Beauty -5) - Future proofing
        // For now, let's say a specific "Debris" building or similar
        // Or just test that grid accepts negative values
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
