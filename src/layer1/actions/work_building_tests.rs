#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
    use crate::layer1::day_night::DayNightCycle;
    use crate::layer1::farm::Farm;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, RefiningProgress};
    use crate::layer1::utility_ai::UtilityWeights;
    use bevy_ecs::prelude::*;

    // Helper to evaluate refine
    use crate::layer1::actions::refine::evaluate_refine;
    // Helper to evaluate farm
    use crate::layer1::actions::farm::evaluate_farm;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(DayNightCycle::default());
        world
    }

    #[test]
    fn test_evaluate_refine_finds_valid_work() {
        let mut world = setup_world();

        // Add resources for input (Wood for Lumber Mill)
        world.resource_mut::<ColonyResources>().wood = 10.0;

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Lumber Mill
        let mill = world
            .spawn((
                Building {
                    building_type: BuildingType::LumberMill,
                },
                GridPosition { x: 2, y: 0 },
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                ShiftSchedule::default(),
            ))
            .id();

        let mut buildings = world.query::<(
            Entity,
            &GridPosition,
            &Building,
            &RefiningProgress,
            Option<&ShiftSchedule>,
        )>();
        let resources = world.resource::<ColonyResources>();
        let cycle = world.resource::<DayNightCycle>();

        let result = evaluate_refine(&pop_pos, &weights, resources, cycle, buildings.iter(&world));

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, mill);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_refine_ignores_unaffordable_recipe() {
        let mut world = setup_world();
        // No wood!
        world.resource_mut::<ColonyResources>().wood = 0.0;

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 2, y: 0 },
            RefiningProgress::default(),
            ShiftSchedule::default(),
        ));

        let mut buildings = world.query::<(
            Entity,
            &GridPosition,
            &Building,
            &RefiningProgress,
            Option<&ShiftSchedule>,
        )>();
        let resources = world.resource::<ColonyResources>();
        let cycle = world.resource::<DayNightCycle>();

        let result = evaluate_refine(&pop_pos, &weights, resources, cycle, buildings.iter(&world));
        assert!(result.is_none(), "Should not refine if inputs are missing");
    }

    #[test]
    fn test_evaluate_farm_finds_work() {
        let mut world = setup_world();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Farm
        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 2, y: 0 },
                Farm {
                    capacity: 1,
                    workers: vec![],
                }, // Workers list might be deprecated/changed
                ShiftSchedule::default(),
            ))
            .id();

        let mut farms = world.query::<(Entity, &GridPosition, &Farm, Option<&ShiftSchedule>)>();
        let cycle = world.resource::<DayNightCycle>();

        let result = evaluate_farm(&pop_pos, &weights, cycle, farms.iter(&world));

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, farm);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_farm_respects_capacity() {
        let mut world = setup_world();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Full Farm (manually filling workers for test)
        let worker = world.spawn(Pop).id();
        world.spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 2, y: 0 },
            Farm {
                capacity: 1,
                workers: vec![worker],
            },
            ShiftSchedule::default(),
        ));

        let mut farms = world.query::<(Entity, &GridPosition, &Farm, Option<&ShiftSchedule>)>();
        let cycle = world.resource::<DayNightCycle>();

        let result = evaluate_farm(&pop_pos, &weights, cycle, farms.iter(&world));
        assert!(result.is_none(), "Should not target full farm");
    }
}
