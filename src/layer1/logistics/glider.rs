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
        use crate::layer1::OccupiedTiles;
        use crate::layer1::pathfinding::find_path_for_glider;
        use crate::layer1::BuildingMap;
        use crate::layer1::wind::WindGrid;

        let mut world = World::new();
        let mut terrain = crate::layer1::nature::terrain::generate_terrain(10, 10);
        // Ensure no water or rock blocks the path
        for tile in terrain.tiles.iter_mut() {
            *tile = crate::layer1::nature::terrain::TerrainType::Grass;
        }
        world.insert_resource(terrain);
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(BuildingMap::default());
        world.insert_resource(WindGrid::new(10, 10));

        let mut temp_grid = TemperatureGrid::new(10, 10, 0.0);

        // Path 1: Direct route (x=0 to x=5 at y=0), extremely cold (0.0) -> High cost
        for x in 0..6 {
            temp_grid.set(x, 0, 0.0);
        }

        // Path 2: Longer route (y=1), very hot (100.0) -> Low cost
        for x in 0..6 {
            temp_grid.set(x, 1, 100.0);
        }
        world.insert_resource(temp_grid);

        let start = (0, 0);
        let end = (5, 0);

        let path = find_path_for_glider(&world, start, end);
        assert!(path.is_some(), "Path should be found");
        let path = path.unwrap();

        // The path should dip down into the hot tiles at y=1 to save "cost",
        // instead of taking the shorter but colder y=0 path.
        let uses_hot_path = path.iter().any(|pos| pos.1 == 1);
        assert!(
            uses_hot_path,
            "Glider pathfinding should prefer longer hot paths over shorter cold ones. Path: {:?}",
            path
        );
    }
}
