#[cfg(test)]
mod tests {
    use crate::layer1::building::Building;
    use crate::layer1::building::BuildingType;
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::resources::ResourceType;
    use crate::layer1::tech::harmonic::{
        process_harmonic_mining, Frequency, HarmonicDrill, ResonantMaterial,
    };
    use crate::layer1::Structure;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_drill_mines_matching_ore() {
        let mut world = setup_world();

        let ore_entity = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                ResonantMaterial {
                    material_type: ResourceType::Ore,
                    frequency: Frequency(100),
                },
            ))
            .id();

        world.spawn((
            GridPosition { x: 5, y: 5 },
            HarmonicDrill {
                radius: 3.0,
                frequency: Frequency(100),
                active: true,
            },
        ));

        // Run the system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // The iron ore should be destroyed (health reduced to 0 or entity despawned)
        assert!(
            world.get_entity(ore_entity).is_err()
                || world.get::<Health>(ore_entity).unwrap().current == 0.0
        );
    }

    #[test]
    fn test_drill_shatters_collateral_buildings() {
        let mut world = setup_world();

        // Spawn a greenhouse nearby
        let greenhouse = world
            .spawn((
                GridPosition { x: 6, y: 5 },
                Structure {
                    current_hp: 50.0,
                    max_hp: 50.0,
                },
                ResonantMaterial {
                    material_type: ResourceType::Blocks,
                    frequency: Frequency(200),
                },
                Building {
                    building_type: BuildingType::Greenhouse,
                },
            ))
            .id();

        // The sonic drill is tuned to the frequency of Glass
        world.spawn((
            GridPosition { x: 5, y: 5 },
            HarmonicDrill {
                radius: 3.0,
                frequency: Frequency(200),
                active: true,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // The greenhouse should be destroyed or severely damaged
        assert!(
            world.get_entity(greenhouse).is_err()
                || world.get::<Structure>(greenhouse).unwrap().current_hp == 0.0
        );
    }

    #[test]
    fn test_drill_ignores_non_matching_materials() {
        let mut world = setup_world();

        // Spawn a steel wall nearby
        let steel_wall = world
            .spawn((
                GridPosition { x: 6, y: 5 },
                Structure {
                    current_hp: 200.0,
                    max_hp: 200.0,
                },
                ResonantMaterial {
                    material_type: ResourceType::Metal,
                    frequency: Frequency(300),
                },
                Building {
                    building_type: BuildingType::Wall,
                },
            ))
            .id();

        // The sonic drill is tuned to Iron
        world.spawn((
            GridPosition { x: 5, y: 5 },
            HarmonicDrill {
                radius: 3.0,
                frequency: Frequency(100),
                active: true,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // The steel wall should be completely unaffected
        assert_eq!(
            world.get::<Structure>(steel_wall).unwrap().current_hp,
            200.0
        );
    }

    #[test]
    fn test_drill_multiple_distance_and_inactive_checks() {
        let mut world = setup_world();

        let ore_entity = world
            .spawn((
                GridPosition { x: 10, y: 10 },
                ResonantMaterial {
                    material_type: ResourceType::Ore,
                    frequency: Frequency(100),
                },
            ))
            .id();

        // Active drill but out of range
        world.spawn((
            GridPosition { x: 0, y: 0 },
            HarmonicDrill {
                radius: 3.0,
                frequency: Frequency(100),
                active: true,
            },
        ));

        // Inactive drill in range
        world.spawn((
            GridPosition { x: 10, y: 10 },
            HarmonicDrill {
                radius: 3.0,
                frequency: Frequency(100),
                active: false,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Ore should be unaffected
        assert!(world.get_entity(ore_entity).is_ok());
    }

    #[test]
    fn test_drill_resource_types() {
        let types = vec![
            ResourceType::Wood,
            ResourceType::Stone,
            ResourceType::Planks,
            ResourceType::Food,
            ResourceType::Metal,
            ResourceType::Blocks,
            ResourceType::Waste,
            ResourceType::Rations,
            ResourceType::Fuel,
            ResourceType::Alcohol,
            ResourceType::Scrap,
            ResourceType::Tools,
            ResourceType::BuildingPermit,
            ResourceType::MemoryCore,
            ResourceType::VoidAle,
            ResourceType::HyperValuable, // Test default branch
        ];

        for (i, res_type) in types.into_iter().enumerate() {
            let mut world = setup_world();
            world.spawn((
                GridPosition { x: 0, y: 0 },
                ResonantMaterial {
                    material_type: res_type,
                    frequency: Frequency(100 + i as u32),
                },
            ));

            world.spawn((
                GridPosition { x: 0, y: 0 },
                HarmonicDrill {
                    radius: 3.0,
                    frequency: Frequency(100 + i as u32),
                    active: true,
                },
            ));

            let mut schedule = Schedule::default();
            schedule.add_systems(process_harmonic_mining);
            schedule.run(&mut world);
        }
    }

    #[test]
    fn test_drill_damages_health_only() {
        let mut world = setup_world();

        let health_entity = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                Health {
                    current: 50.0,
                    max: 50.0,
                    has_rust_lung: false,
                },
                ResonantMaterial {
                    material_type: ResourceType::Wood,
                    frequency: Frequency(100),
                },
            ))
            .id();

        world.spawn((
            GridPosition { x: 0, y: 0 },
            HarmonicDrill {
                radius: 3.0,
                frequency: Frequency(100),
                active: true,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        assert_eq!(world.get::<Health>(health_entity).unwrap().current, 0.0);
    }
}
