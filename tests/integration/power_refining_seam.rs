#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::energy::PowerConsumer;
    use scale::layer1::pop::Pop;
    use scale::layer1::refining::process_refining_system;
    use scale::layer1::resources::{ColonyResources, RefiningProgress};
    use scale::layer1::skills::Skills;
    use scale::layer1::utility_ai::{ActionType, PopAction};
    use scale::layer1::GridPosition;

    #[test]
    fn test_smelter_requires_power_to_operate() {
        let mut world = World::new();

        // Setup Resources: Ore and Wood for Smelting
        let resources = ColonyResources {
            ore: 10.0,
            wood: 10.0,
            metal: 0.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        // Spawn Smelter with PowerConsumer (active: false)
        let smelter = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                PowerConsumer {
                    demand: 5.0,
                    active: false, // Not powered!
                },
            ))
            .id();

        // Spawn Worker at building
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Skills::default(),
            PopAction {
                current: ActionType::Refine,
                ..Default::default()
            },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_refining_system);
        schedule.run(&mut world);

        // Check progress
        let progress = world.get::<RefiningProgress>(smelter).unwrap();

        // Should NOT work (no power)
        assert_eq!(
            progress.current, 0.0,
            "Smelter should not refine without power!"
        );
    }

    #[test]
    fn test_smelter_operates_with_power() {
        let mut world = World::new();

        // Setup Resources
        let resources = ColonyResources {
            ore: 10.0,
            wood: 10.0,
            metal: 0.0,
            ..Default::default()
        };
        world.insert_resource(resources);

        // Spawn Smelter with PowerConsumer (active: true)
        let smelter = world
            .spawn((
                Building {
                    building_type: BuildingType::Smelter,
                },
                GridPosition { x: 5, y: 5 },
                RefiningProgress {
                    current: 0.0,
                    max: 10.0,
                },
                PowerConsumer {
                    demand: 5.0,
                    active: true, // Powered!
                },
            ))
            .id();

        // Spawn Worker at building
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Skills::default(),
            PopAction {
                current: ActionType::Refine,
                ..Default::default()
            },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_refining_system);
        schedule.run(&mut world);

        // Check progress
        let progress = world.get::<RefiningProgress>(smelter).unwrap();

        // Should work
        assert!(progress.current > 0.0, "Smelter should refine when powered");
    }
}
