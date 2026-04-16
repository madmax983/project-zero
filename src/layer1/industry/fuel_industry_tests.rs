#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::refining::process_refining_system;
    use crate::layer1::resources::{ColonyResources, RefiningProgress};
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::GridPosition;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_resources_fuel_fields() {
        let res = ColonyResources::default();
        // New field
        assert_eq!(res.fuel, 0.0);
        // Default max
        assert_eq!(res.max_fuel, 20.0);
    }

    #[test]
    fn test_building_type_refinery() {
        let b = BuildingType::Refinery;
        assert_eq!(b.label(), "Refinery");
        assert_eq!(b.char(), 'R');
    }

    #[test]
    fn test_refining_ore_to_fuel() {
        let mut world = World::new();
        let res = ColonyResources {
            ore: 10.0,
            fuel: 0.0,
            ..Default::default()
        };
        world.insert_resource(res);

        // Spawn Refinery
        world.spawn((
            Building {
                building_type: BuildingType::Refinery,
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
                current_utility: 0.5,
                ticks_committed: 1,
            },
        ));

        // Run system
        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Recipe: 2 Ore -> 1 Fuel
        assert!((res.ore - 8.0).abs() < f32::EPSILON, "Should consume 2 Ore");
        assert!(
            (res.fuel - 1.0).abs() < f32::EPSILON,
            "Should produce 1 Fuel"
        );
    }

    #[test]
    fn test_refining_stops_if_insufficient_ore() {
        let mut world = World::new();
        let res = ColonyResources {
            ore: 1.0,
            fuel: 0.0,
            ..Default::default()
        };
        world.insert_resource(res);

        world.spawn((
            Building {
                building_type: BuildingType::Refinery,
            },
            GridPosition { x: 5, y: 5 },
            RefiningProgress {
                current: 0.0,
                max: 10.0,
            },
        ));
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            PopAction {
                current: ActionType::Refine,
                current_utility: 0.5,
                ticks_committed: 1,
            },
        ));

        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert_eq!(progress.current, 0.0);
    }

    #[test]
    fn test_add_fuel_clamps_to_max() {
        let mut res = ColonyResources {
            max_fuel: 10.0,
            fuel: 5.0,
            ..Default::default()
        };
        res.add_fuel(10.0);
        assert!((res.fuel - 10.0).abs() < f32::EPSILON);
    }
}
