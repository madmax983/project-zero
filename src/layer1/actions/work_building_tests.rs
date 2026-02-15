#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::farm::Farm;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ColonyResources, RefiningProgress};
    use crate::layer1::tech::Library;
    use crate::layer1::utility_types::{
        FarmProxy, LibraryProxy, RefiningProxy, UtilityWeights,
    };
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

        // Create Proxy
        let proxies = vec![RefiningProxy {
            entity: mill,
            pos: GridPosition { x: 2, y: 0 },
            progress_current: 0.0,
        }];

        let result = evaluate_refine(&pop_pos, &weights, &proxies);

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, mill);
        assert!(utility > 0.0);
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
                },
                ShiftSchedule::default(),
            ))
            .id();

        // Create Proxy
        let proxies = vec![FarmProxy {
            entity: farm,
            pos: GridPosition { x: 2, y: 0 },
            capacity: 1,
            workers: 0,
        }];

        let result = evaluate_farm(&pop_pos, &weights, &proxies);

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, farm);
        assert!(utility > 0.0);
    }

    // Removed tests for shift schedules and capacity checks as those responsibilities
    // have moved to `evaluate_actions_system` (pre-filtering).
    // The `evaluate_*` functions now assume they receive valid candidates.
}
