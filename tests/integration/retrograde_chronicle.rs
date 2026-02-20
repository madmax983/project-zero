#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::building::{Building, BuildingType};
    use scale::layer1::chronicle::{AddChronicleEvent, Chronicle};
    use scale::layer1::designation::{Designation, DesignationType};
    use scale::layer1::execution::execute_demolish;
    use scale::layer1::heirloom::{AncientStructure, RetrogradeEngineeringEvent};
    use scale::layer1::map::GridPosition;
    use scale::layer1::resources::ColonyResources;
    use scale::shared::log::MessageLog;

    #[test]
    fn test_retrograde_engineering_integration_flow() {
        let mut world = World::new();
        // Setup resources
        world.insert_resource(ColonyResources::default());
        world.insert_resource(MessageLog::default());
        world.insert_resource(Chronicle::default());
        world.insert_resource(scale::layer1::building::OccupiedTiles::default());
        world.insert_resource(scale::layer1::map::ScreenShake::default());

        // Initialize Events
        world.init_resource::<Events<RetrogradeEngineeringEvent>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        // Spawn Ancient Reactor
        let _reactor = world
            .spawn((
                Building {
                    building_type: BuildingType::AncientReactor,
                },
                AncientStructure,
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Designation
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Demolish,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // 1. Run execute_demolish
        execute_demolish(&mut world, designation);

        // 2. Check for RetrogradeEngineeringEvent (This part tests emission)
        let events = world.resource::<Events<RetrogradeEngineeringEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader.read(events).collect();

        assert_eq!(emitted.len(), 1, "Should emit RetrogradeEngineeringEvent");
        assert_eq!(emitted[0].building_label, "Ancient Reactor");
        assert_eq!(emitted[0].knowledge_gained, 500.0);

        // 3. Run Bridge System (This part tests Glue)
        let mut schedule = Schedule::default();
        schedule.add_systems(scale::layer1::integration::retrograde_chronicle_bridge);
        schedule.run(&mut world);

        let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
        let mut chronicle_reader = chronicle_events.get_cursor();
        let chronicle_emitted: Vec<_> = chronicle_reader.read(chronicle_events).collect();

        assert_eq!(
            chronicle_emitted.len(),
            1,
            "Glue should produce Chronicle event"
        );
        assert!(
            chronicle_emitted[0]
                .text
                .contains("Sacrificed Ancient Reactor")
        );
    }
}
