#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::refining::process_refining_system;
    use crate::layer1::resources::{ColonyResources, MiningProgress, RefiningProgress, mine_rock};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::{Designation, DesignationType, GridPosition};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_resources_metal_fields() {
        let res = ColonyResources::default();
        assert_eq!(res.ore, 0.0);
        assert_eq!(res.metal, 0.0);
        assert_eq!(res.max_ore, 20.0);
        assert_eq!(res.max_metal, 20.0);
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

        let mut ore_count = 0.0;

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

            let res = world.resource::<ColonyResources>();
            if res.ore > ore_count {
                ore_count = res.ore;
            }
        }

        // Should have found SOME ore (20% chance * 100 trials = ~20)
        assert!(ore_count > 0.0, "Mining 100 rocks should yield some ore");
        assert!(ore_count < 100.0, "Every rock shouldn't yield ore");
    }

    #[test]
    fn test_smelter_refines_ore_to_metal() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.ore = 10.0;
        res.wood = 10.0;
        res.metal = 0.0;
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
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system
        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Cost: 1 Ore + 1 Wood
        assert!((res.ore - 9.0).abs() < f32::EPSILON);
        assert!((res.wood - 9.0).abs() < f32::EPSILON);
        // Gain: 1 Metal
        assert!((res.metal - 1.0).abs() < f32::EPSILON);
    }
}
