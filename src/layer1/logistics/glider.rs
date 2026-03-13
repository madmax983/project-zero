// Implementation block to make tests compilable (but fail).
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Speed;
use crate::layer1::temperature::TemperatureGrid;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ThermalGlider;

pub fn update_glider_movement_system(
    mut query: Query<(&ThermalGlider, &GridPosition, &mut Speed)>,
    temp_grid: Res<TemperatureGrid>,
) {
    for (_, pos, mut speed) in query.iter_mut() {
        if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
            let temp = temp_grid.get(x, y);

            if temp > 30.0 {
                // Updraft!
                speed.current = 2.0;
            } else if temp > 10.0 {
                // Weak lift
                speed.current = 1.0;
            } else {
                // Grounded
                speed.current = 0.1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::logistics::glider::{update_glider_movement_system, ThermalGlider};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Speed;
    use crate::layer1::temperature::TemperatureGrid;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_glider_speed_boost_in_heat() {
        let mut world = World::new();
        let mut temp_grid = TemperatureGrid::new(10, 10, 0.0);

        // Heat Highway at (0,0)
        temp_grid.set(0, 0, 50.0);
        // Cold Zone at (0,1)
        temp_grid.set(0, 1, 0.0);

        world.insert_resource(temp_grid);

        // Spawn Glider
        let glider = world
            .spawn((
                ThermalGlider,
                Speed {
                    base: 1.0,
                    current: 0.0, // Base speed 0? Or just modified.
                    accumulator: 0.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_glider_movement_system);
        schedule.run(&mut world);

        let speed = world.get::<Speed>(glider).unwrap();
        assert!(speed.current > 1.0, "Glider should fly in heat");
    }

    #[test]
    fn test_glider_grounded_in_cold() {
        let mut world = World::new();
        let temp_grid = TemperatureGrid::new(10, 10, 0.0); // Default 0
        world.insert_resource(temp_grid);

        let glider = world
            .spawn((
                ThermalGlider,
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_glider_movement_system);
        schedule.run(&mut world);

        let speed = world.get::<Speed>(glider).unwrap();
        assert!(speed.current < 0.2, "Glider should crawl/stop in cold");
    }

    #[test]
    fn test_glider_pathfinding_prefers_heat() {
        use crate::layer1::building::{BuildingMap, OccupiedTiles};
        use crate::layer1::pathfinding::find_path_with_cost;
        use crate::layer1::terrain::{TerrainGrid, TerrainType};

        let mut world = World::new();

        // 10x10 grass grid
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(BuildingMap::default());

        let mut temp_grid = TemperatureGrid::new(10, 10, 0.0);

        // Path directly across is short but cold (y=0)
        // Path below is longer but hot (y=1 to y=3)
        // Set up a hot "C" shaped highway:
        // (0,0) -> (0,1) -> (1,1) -> (2,1) -> (2,0)
        temp_grid.set(0, 1, 50.0);
        temp_grid.set(1, 1, 50.0);
        temp_grid.set(2, 1, 50.0);

        world.insert_resource(temp_grid);

        let temp_grid_ref = world.resource::<TemperatureGrid>();

        // Custom cost function for glider
        let glider_cost = |pos: (i32, i32), _base_cost: i32| -> i32 {
            let temp = temp_grid_ref.get(pos.0 as usize, pos.1 as usize);
            if temp > 30.0 {
                // High temperature => low cost
                1
            } else {
                // Low temperature => high cost, avoid
                100
            }
        };

        // Start (0,0) -> End (2,0)
        // The path should prefer going down to the heat:
        // (0,0) -> (0,1) -> (1,1) -> (2,1) -> (2,0)
        let path_opt = find_path_with_cost(&world, (0, 0), (2, 0), &glider_cost);
        assert!(path_opt.is_some(), "Path should exist");

        let path = path_opt.unwrap();

        let uses_heat = path.iter().any(|pos| pos.1 == 1);
        assert!(
            uses_heat,
            "Glider path should prefer the longer, hot route. Path was: {:?}",
            path
        );
    }
}
