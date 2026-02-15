#[cfg(test)]
mod tests {
    use crate::layer1::building::OccupiedTiles;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::execution::execute_demolish;
    use crate::layer1::heirloom::AncientStructure;
    use crate::layer1::map::GridPosition;
    use crate::layer1::map::ScreenShake;
    use crate::layer1::resources::ColonyResources;
    use crate::shared::log::MessageLog;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_demolish_normal_building_gives_no_knowledge() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ScreenShake::default());
        world.insert_resource(MessageLog::default());

        let _building = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Demolish,
                },
                GridPosition { x: 0, y: 0 },
            ))
            .id();

        // Execute
        let success = execute_demolish(&mut world, designation);
        assert!(success);

        // Verify no knowledge gain
        let res = world.resource::<ColonyResources>();
        assert_eq!(res.knowledge, 0.0);
    }

    #[test]
    fn test_demolish_ancient_reactor_gives_500_knowledge() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        // Raise max knowledge so we can hold 500
        let mut resources = ColonyResources::default();
        resources.max_knowledge = 1000.0;
        world.insert_resource(resources);

        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ScreenShake::default());
        world.insert_resource(MessageLog::default());

        let _building = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                AncientStructure,
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Demolish,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Execute
        execute_demolish(&mut world, designation);

        // Verify massive knowledge gain
        let res = world.resource::<ColonyResources>();
        assert_eq!(res.knowledge, 500.0);
    }

    #[test]
    fn test_demolish_ancient_fabricator_gives_300_knowledge() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.max_knowledge = 1000.0;
        world.insert_resource(resources);

        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ScreenShake::default());
        world.insert_resource(MessageLog::default());

        let _building = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientFabricator,
                },
                AncientStructure,
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Demolish,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Execute
        execute_demolish(&mut world, designation);

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.knowledge, 300.0);
    }

    #[test]
    fn test_demolish_unknown_ancient_structure_gives_100_knowledge() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.max_knowledge = 1000.0;
        world.insert_resource(resources);

        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ScreenShake::default());
        world.insert_resource(MessageLog::default());

        // Tower is not an ancient structure by default, but we force the component
        let _building = world
            .spawn((
                Building {
                    building_type: BuildingType::Tower,
                },
                AncientStructure,
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Demolish,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Execute
        execute_demolish(&mut world, designation);

        let res = world.resource::<ColonyResources>();
        assert_eq!(res.knowledge, 100.0);
    }

    #[test]
    fn test_demolish_ancient_structure_notification() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.max_knowledge = 1000.0;
        world.insert_resource(resources);

        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ScreenShake::default());
        world.insert_resource(MessageLog::default());

        let _building = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                AncientStructure,
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Demolish,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        execute_demolish(&mut world, designation);

        let log = world.resource::<MessageLog>();
        assert!(
            log.messages
                .back()
                .unwrap()
                .text
                .contains("Retrograde Engineering")
        );
    }
}
