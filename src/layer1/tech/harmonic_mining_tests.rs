#[cfg(test)]
mod tests {
    use crate::layer1::building::BuildingType;
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::layer1::tech::harmonic::{
        process_harmonic_mining, Frequency, HarmonicDrill, ResonantMaterial,
    };
    use crate::layer1::Structure;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_drill_mines_matching_ore() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Drill tuned to Iron (Frequency 100)
        let _drill = world
            .spawn((
                HarmonicDrill {
                    frequency: Frequency(100),
                    radius: 3.0,
                    active: true,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Spawn Ore Node (Iron) nearby
        let ore = world
            .spawn((
                ResonantMaterial {
                    frequency: Frequency(100),
                    material_type: ResourceType::Ore,
                }, // Assuming Ore
                GridPosition { x: 6, y: 5 },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Ore should be destroyed/mined
        assert!(world.get_entity(ore).is_err());

        // Resources should increase
        let resources = world.resource::<ColonyResources>();
        assert!(resources.ore > 0.0);
    }

    #[test]
    fn test_drill_shatters_matching_structure() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Spawn Drill tuned to Planks (Frequency 200)
        world.spawn((
            HarmonicDrill {
                frequency: Frequency(200),
                radius: 5.0,
                active: true,
            },
            GridPosition { x: 10, y: 10 },
        ));

        // Spawn Greenhouse (Planks as substitute for Glass) nearby
        let greenhouse = world
            .spawn((
                crate::layer1::building::Building {
                    building_type: BuildingType::Greenhouse,
                },
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                ResonantMaterial {
                    frequency: Frequency(200),
                    material_type: ResourceType::Planks,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                GridPosition { x: 12, y: 10 },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Greenhouse should take massive damage or be destroyed
        let health = world.get::<Health>(greenhouse);
        // Either entity is gone OR health is 0
        if let Some(h) = health {
            assert_eq!(h.current, 0.0);
        } else {
            // Entity despawned implies destruction
        }
    }

    #[test]
    fn test_drill_ignores_mismatched_objects() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Drill tuned to Ore (100)
        world.spawn((
            HarmonicDrill {
                frequency: Frequency(100),
                radius: 3.0,
                active: true,
            },
            GridPosition { x: 0, y: 0 },
        ));

        // Planks Structure (200)
        let greenhouse = world
            .spawn((
                ResonantMaterial {
                    frequency: Frequency(200),
                    material_type: ResourceType::Planks,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    conditions: Vec::new(),
                },
                GridPosition { x: 1, y: 0 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(process_harmonic_mining);
        schedule.run(&mut world);

        // Should be unharmed
        let health = world.get::<Health>(greenhouse).unwrap();
        assert_eq!(health.current, 100.0);
    }
}
