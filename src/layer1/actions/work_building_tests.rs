#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::farm::Farm;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, RefiningProgress};
    use crate::layer1::tech::Library;
    use crate::layer1::utility_types::UtilityWeights;
    use bevy_ecs::prelude::*;

    // Helper to evaluate refine
    use crate::layer1::actions::refine::evaluate_refine;
    // Helper to evaluate farm
    use crate::layer1::actions::farm::evaluate_farm;
    // Helper to evaluate research
    use crate::layer1::actions::research::evaluate_research;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::utility_types::UtilityConfig::default());
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

    #[test]
    fn test_evaluate_refine_respects_shifts() {
        let mut world = setup_world();
        world.resource_mut::<ColonyResources>().wood = 10.0; // Input available

        // Set Time to Night
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Building with Day Shift Only (Default)
        let building_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::LumberMill,
                },
                GridPosition { x: 2, y: 0 },
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // Evaluate Refine -> Should be None
        {
            let mut buildings = world.query::<(
                Entity,
                &GridPosition,
                &Building,
                &RefiningProgress,
                Option<&ShiftSchedule>,
            )>();
            let resources = world.resource::<ColonyResources>();
            let cycle = world.resource::<DayNightCycle>();

            let result =
                evaluate_refine(&pop_pos, &weights, resources, cycle, buildings.iter(&world));
            assert!(
                result.is_none(),
                "Should not work at night if night shift is disabled"
            );
        }

        // Enable Night Shift
        {
            let mut schedule = world.get_mut::<ShiftSchedule>(building_entity).unwrap();
            schedule.night_shift = true;
        }

        // Evaluate Refine -> Should be Some
        {
            let mut buildings = world.query::<(
                Entity,
                &GridPosition,
                &Building,
                &RefiningProgress,
                Option<&ShiftSchedule>,
            )>();
            let resources = world.resource::<ColonyResources>();
            let cycle = world.resource::<DayNightCycle>();

            let result =
                evaluate_refine(&pop_pos, &weights, resources, cycle, buildings.iter(&world));
            assert!(
                result.is_some(),
                "Should work at night if night shift is enabled"
            );
        }
    }

    #[test]
    fn test_evaluate_farm_respects_shifts() {
        let mut world = setup_world();

        // Set Time to Night
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Farm
        let farm_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 2, y: 0 },
                Farm::default(),
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // Evaluate Farm -> Should be None
        {
            let mut farms = world.query::<(Entity, &GridPosition, &Farm, Option<&ShiftSchedule>)>();
            let cycle = world.resource::<DayNightCycle>();

            let result = evaluate_farm(&pop_pos, &weights, cycle, farms.iter(&world));
            assert!(
                result.is_none(),
                "Should not farm at night if night shift is disabled"
            );
        }

        // Enable Night Shift
        {
            let mut schedule = world.get_mut::<ShiftSchedule>(farm_entity).unwrap();
            schedule.night_shift = true;
        }

        // Evaluate Farm -> Should be Some
        {
            let mut farms = world.query::<(Entity, &GridPosition, &Farm, Option<&ShiftSchedule>)>();
            let cycle = world.resource::<DayNightCycle>();

            let result = evaluate_farm(&pop_pos, &weights, cycle, farms.iter(&world));
            assert!(
                result.is_some(),
                "Should farm at night if night shift is enabled"
            );
        }
    }

    #[test]
    fn test_evaluate_research_respects_shifts() {
        let mut world = setup_world();

        // Set Time to Night
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Library
        let lib_entity = world
            .spawn((
                Building {
                    building_type: BuildingType::Library,
                },
                GridPosition { x: 2, y: 0 },
                Library,
                ShiftSchedule {
                    day_shift: true,
                    night_shift: false,
                },
            ))
            .id();

        // Evaluate Research -> Should be None
        {
            let mut libraries = world.query::<(
                Entity,
                &GridPosition,
                &Library,
                Option<&ShiftSchedule>,
            )>();
            let resources = world.resource::<ColonyResources>();
            let cycle = world.resource::<DayNightCycle>();

            let result = evaluate_research(
                &pop_pos,
                &weights,
                resources,
                cycle,
                libraries.iter(&world),
            );
            assert!(
                result.is_none(),
                "Should not research at night if night shift is disabled"
            );
        }

        // Enable Night Shift
        {
            let mut schedule = world.get_mut::<ShiftSchedule>(lib_entity).unwrap();
            schedule.night_shift = true;
        }

        // Evaluate Research -> Should be Some
        {
            let mut libraries = world.query::<(
                Entity,
                &GridPosition,
                &Library,
                Option<&ShiftSchedule>,
            )>();
            let resources = world.resource::<ColonyResources>();
            let cycle = world.resource::<DayNightCycle>();

            let result = evaluate_research(
                &pop_pos,
                &weights,
                resources,
                cycle,
                libraries.iter(&world),
            );
            assert!(
                result.is_some(),
                "Should research at night if night shift is enabled"
            );
        }
    }
}
