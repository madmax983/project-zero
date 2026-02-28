#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::map::GridPosition;
    use crate::layer1::seasons::{Season, SeasonState};
    use crate::layer1::temperature::{update_temperature_system, TemperatureGrid};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup Season (Summer for heat)
        let season = SeasonState {
            current_season: Season::Summer,
        };
        let ambient = season.current_season.base_temperature(); // 30.0

        world.insert_resource(season);

        // Setup Grid starting AT AMBIENT to test divergence
        let grid = TemperatureGrid::new(10, 10, ambient);
        world.insert_resource(grid);

        // Setup Day/Night (Day initially)
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        });

        // Setup Terrain (Grass default)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        world
    }

    #[test]
    fn test_solar_heat_gain_during_day() {
        let mut world = setup_world();

        // Run update during Day
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        let ambient = world
            .resource::<SeasonState>()
            .current_season
            .base_temperature();

        // Solar gain should push it ABOVE ambient.
        assert!(
            grid.get(5, 5) > ambient,
            "Tiles should gain heat from sun above ambient. Current: {}, Ambient: {}",
            grid.get(5, 5),
            ambient
        );
    }

    #[test]
    fn test_no_solar_heat_at_night() {
        let mut world = setup_world();
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        // Run update
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        let _ambient = world
            .resource::<SeasonState>()
            .current_season
            .base_temperature();

        // Should stay at ambient (or cool down if hotter), but not gain heat.
        // Since we start at 20.0 and ambient might be 30.0 (Summer), it will drift TOWARDS ambient.
        // But it shouldn't jump *above* the ambient + margin.
        // Actually, if it's night, solar input is 0.
        // If we start at 20.0 and ambient is 30.0, it will drift up to 30.0.
        // So checking if it's > ambient isn't quite right if it's drifting up.
        // The spec says: "Compare Day gain vs Night gain."

        // Let's do that.
        let night_temp = grid.get(5, 5);

        // Reset and run Day
        let mut world_day = setup_world();
        world_day
            .run_system_once(update_temperature_system)
            .unwrap();
        let day_temp = world_day.resource::<TemperatureGrid>().get(5, 5);

        assert!(
            day_temp > night_temp,
            "Day should be hotter than Night due to solar gain"
        );
    }

    #[test]
    fn test_retention_slows_cooling() {
        // Setup two tiles: One Grass (Low Retention), One Concrete/Wall (High Retention)
        // Both start HOT (50C). Ambient is COOL (10C).
        // Wall should cool down SLOWER.

        let mut world = World::new();
        let mut grid = TemperatureGrid::new(10, 10, 10.0); // Ambient 10
        grid.set(0, 0, 50.0); // Tile 1 (Grass)
        grid.set(1, 0, 50.0); // Tile 2 (Wall)
        world.insert_resource(grid);
        world.insert_resource(SeasonState {
            current_season: Season::Autumn,
        }); // Ambient likely 10-15
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        }); // No sun

        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Add Wall at (1,0)
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 1, y: 0 },
        ));

        // Run update
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        let temp_grass = grid.get(0, 0);
        let temp_wall = grid.get(1, 0);

        // Both should cool down (< 50)
        assert!(temp_grass < 50.0);
        assert!(temp_wall < 50.0);

        // Grass should be COOLER than Wall (Wall retains heat)
        assert!(
            temp_wall > temp_grass,
            "High retention wall should stay hotter longer. Wall: {}, Grass: {}",
            temp_wall,
            temp_grass
        );
    }

    #[test]
    fn test_urban_heat_island_effect() {
        // A cluster of buildings should be hotter than surrounding area during the day
        let mut world = setup_world();

        // Spawn 3x3 block of StoneMason buildings (Industrial/Stone)
        for y in 3..6 {
            for x in 3..6 {
                world.spawn((
                    Building {
                        building_type: BuildingType::StoneMason,
                    },
                    GridPosition { x, y },
                ));
            }
        }

        // Run simulation for a few ticks during Day
        for _ in 0..5 {
            world.run_system_once(update_temperature_system).unwrap();
        }

        let grid = world.resource::<TemperatureGrid>();
        let city_temp = grid.get(4, 4); // Center of city
        let nature_temp = grid.get(0, 0); // Corner (Grass)

        assert!(
            city_temp > nature_temp + 1.0,
            "Urban center should be significantly hotter. City: {}, Nature: {}",
            city_temp,
            nature_temp
        );
    }
}
