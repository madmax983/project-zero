#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::refining::process_refining_system;
    use crate::layer1::resources::{mine_rock, ColonyResources, MiningProgress, RefiningProgress};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::{Designation, DesignationType, GridPosition};
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_resources_metal_fields() {
        let res = ColonyResources::default();
        assert!(res.ore.abs() < f32::EPSILON);
        assert!(res.metal.abs() < f32::EPSILON);
        assert!((res.max_ore - 20.0).abs() < f32::EPSILON);
        assert!((res.max_metal - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_building_type_smelter() {
        let b = BuildingType::Smelter;
        assert_eq!(b.label(), "Smelter");
        assert_eq!(b.char(), 'S');
    }

    #[test]
    fn test_mine_rock_yields_ore_probabilistically() {
        let mut world = World::new();
        // Setup massive grid of rocks
        let tiles = vec![TerrainType::Rock; 1000];
        world.insert_resource(TerrainGrid {
            width: 100,
            height: 10,
            tiles,
        });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(100, 10));

        // Mine 100 rocks
        for i in 0..100 {
            let entity = world
                .spawn((
                    Designation {
                        designation_type: DesignationType::Mine,
                    },
                    MiningProgress {
                        current: 9.9,
                        max: 10.0,
                    }, // Almost done
                    GridPosition { x: i % 100, y: 0 },
                ))
                .id();

            mine_rock(&mut world, entity, 0.2); // Complete it
        }

        // Check for ResourceItems of type Ore
        let ore_count = world
            .query::<&crate::layer1::resources::ResourceItem>()
            .iter(&world)
            .filter(|item| item.resource_type == crate::layer1::resources::ResourceType::Ore)
            .count();

        // Should have found SOME ore (20% chance * 100 trials = ~20)
        assert!(ore_count > 0, "Mining 100 rocks should yield some ore");
        assert!(ore_count < 100, "Every rock shouldn't yield ore");
    }

    #[test]
    fn test_smelter_refines_ore_to_metal() {
        let mut world = World::new();
        let res = ColonyResources {
            ore: 10.0,
            wood: 10.0,
            metal: 0.0,
            ..Default::default()
        };
        world.insert_resource(res);

        // Spawn Smelter
        world.spawn((
            Building {
                building_type: BuildingType::Smelter,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress {
                current: 9.9,
                max: 10.0,
            },
        ));

        // Spawn Worker
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Refine,
                ..Default::default()
            },
        ));

        // Run system
        world.run_system_once(process_refining_system).unwrap();

        let res = world.resource::<ColonyResources>();
        // Cost: 1 Ore + 1 Wood
        assert!((res.ore - 9.0).abs() < f32::EPSILON);
        assert!((res.wood - 9.0).abs() < f32::EPSILON);
        // Gain: 1 Metal
        assert!((res.metal - 1.0).abs() < f32::EPSILON);
    }
}
