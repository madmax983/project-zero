#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use scale::layer1::{
        GridPosition,
        acoustic::{NoiseMap, NoiseSource, update_noise_system},
        building::{Building, BuildingType},
        housing::{Housing, restore_rest_in_housing_system},
        needs::Needs,
        pop::Pop,
        terrain::generate_terrain,
    };

    #[test]
    fn test_noise_reduces_rest_recovery() {
        let mut world = World::new();

        // Initialize resources
        world.insert_resource(NoiseMap::new(10, 10));
        world.insert_resource(generate_terrain(10, 10));

        // Create two pops with low rest
        let pop_quiet = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.1,
                    ..Default::default()
                },
            ))
            .id();

        let pop_noisy = world
            .spawn((
                Pop,
                Needs {
                    rest: 0.1,
                    ..Default::default()
                },
            ))
            .id();

        // Create quiet housing at (0, 0)
        let housing_quiet = Housing {
            capacity: 1,
            residents: vec![pop_quiet],
        };
        world.spawn((
            housing_quiet,
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Create noisy housing at (5, 5) with a noise source
        let housing_noisy = Housing {
            capacity: 1,
            residents: vec![pop_noisy],
        };
        world.spawn((
            housing_noisy,
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Add noise source at (5, 5)
        world.spawn((
            NoiseSource {
                radius: 5.0,
                intensity: 1.0,
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run systems
        // 1. Update noise map
        world.run_system_once(update_noise_system).unwrap();

        // 2. Restore rest
        // We run this multiple times to see a difference
        for _ in 0..10 {
            world
                .run_system_once(restore_rest_in_housing_system)
                .unwrap();
        }

        // Check results
        let rest_quiet = world.get::<Needs>(pop_quiet).unwrap().rest;
        let rest_noisy = world.get::<Needs>(pop_noisy).unwrap().rest;

        println!("Rest Quiet: {}, Rest Noisy: {}", rest_quiet, rest_noisy);

        // Expectation: Quiet rest recovery > Noisy rest recovery
        assert!(
            rest_quiet > rest_noisy,
            "Noisy environment should reduce rest recovery"
        );
    }
}
