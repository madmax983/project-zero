#[cfg(test)]
mod tests {
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::energy::PowerConsumer;
    use scale::layer1::pop::Pop;
    use scale::layer1::refining::process_refining_system;
    use scale::layer1::resources::{ColonyResources, RefiningProgress};
    use scale::layer1::GridPosition;
    use scale::layer1::skills::Skills;
    use bevy_ecs::prelude::*;

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
        let smelter = world.spawn((
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
        )).id();

        // Spawn Worker nearby
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 6 },
            Skills::default(),
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(process_refining_system);
        schedule.run(&mut world);

        // Check progress
        let progress = world.get::<RefiningProgress>(smelter).unwrap();

        // ISSUE: Currently this will be > 0.0 because power is ignored.
        // We want it to be 0.0.
        assert_eq!(progress.current, 0.0, "Smelter should not refine without power!");
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
        let smelter = world.spawn((
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
        )).id();

        // Spawn Worker nearby
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 6 },
            Skills::default(),
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
