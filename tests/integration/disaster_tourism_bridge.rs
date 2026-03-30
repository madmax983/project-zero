#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::disasters::{DisasterEvent, DisasterType};
    use scale::layer1::geology::tectonic::MegaQuakeEvent;
    use scale::layer1::volatile::ExplosionEvent;
    use scale::layer2::governance::RebellionEvent;
    use scale::layer1::map::GridPosition;
    use scale::layer2::tourism::disaster_tourism::GriefTouristArrivalEvent;
    use scale::layer2::integration::{layer1_disaster_bridge_system, grief_tourist_arrival_handler_system};
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<MegaQuakeEvent>>();
        world.init_resource::<Events<ExplosionEvent>>();
        world.init_resource::<Events<RebellionEvent>>();
        world.init_resource::<Events<DisasterEvent>>();
        world.init_resource::<Events<GriefTouristArrivalEvent>>();
        world.init_resource::<Events<AddChronicleEvent>>();
        world.init_resource::<ColonyResources>();
        world
    }

    #[test]
    fn test_mega_quake_triggers_disaster_event() {
        let mut world = setup_world();

        let mut schedule = Schedule::default();
        schedule.add_systems(layer1_disaster_bridge_system);

        world.send_event(MegaQuakeEvent);

        schedule.run(&mut world);

        let disaster_events = world.resource::<Events<DisasterEvent>>();
        #[allow(deprecated)]
        let mut reader = disaster_events.get_reader();
        let emitted: Vec<_> = reader.read(disaster_events).collect();
        assert_eq!(emitted.len(), 1);

        let disaster = emitted[0];
        assert_eq!(disaster.disaster_type, DisasterType::MassiveEarthquake);
        assert_eq!(disaster.severity, 90.0);
    }

    #[test]
    fn test_explosion_triggers_disaster_event() {
        let mut world = setup_world();

        let mut schedule = Schedule::default();
        schedule.add_systems(layer1_disaster_bridge_system);

        world.send_event(ExplosionEvent {
            center: GridPosition { x: 5, y: 5 },
            radius: 3,
            damage: 85.0,
        });

        schedule.run(&mut world);

        let disaster_events = world.resource::<Events<DisasterEvent>>();
        #[allow(deprecated)]
        let mut reader = disaster_events.get_reader();
        let emitted: Vec<_> = reader.read(disaster_events).collect();
        assert_eq!(emitted.len(), 1);

        let disaster = emitted[0];
        assert_eq!(disaster.disaster_type, DisasterType::ReactorMeltdown); // Map massive explosion to meltdown
        assert_eq!(disaster.location, GridPosition { x: 5, y: 5 });
        assert_eq!(disaster.severity, 85.0);
    }

    #[test]
    fn test_rebellion_triggers_disaster_event() {
        let mut world = setup_world();

        let mut schedule = Schedule::default();
        schedule.add_systems(layer1_disaster_bridge_system);

        world.send_event(RebellionEvent {
            planet_entity: Entity::PLACEHOLDER,
        });

        schedule.run(&mut world);

        let disaster_events = world.resource::<Events<DisasterEvent>>();
        #[allow(deprecated)]
        let mut reader = disaster_events.get_reader();
        let emitted: Vec<_> = reader.read(disaster_events).collect();
        assert_eq!(emitted.len(), 1);

        let disaster = emitted[0];
        assert_eq!(disaster.disaster_type, DisasterType::ViolentUprising);
        assert_eq!(disaster.severity, 90.0); // Arbitrary high severity
    }

    #[test]
    fn test_grief_tourists_provide_credits_and_chronicle() {
        let mut world = setup_world();
        world.resource_mut::<ColonyResources>().credits = 1000.0;

        let mut schedule = Schedule::default();
        schedule.add_systems(grief_tourist_arrival_handler_system);

        world.send_event(GriefTouristArrivalEvent {
            target_location: GridPosition { x: 10, y: 10 },
            offered_credits: 50000.0,
        });

        schedule.run(&mut world);

        // Check credits
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.credits, 51000.0);

        // Check chronicle
        let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
        #[allow(deprecated)]
        let mut reader = chronicle_events.get_reader();
        let emitted: Vec<_> = reader.read(chronicle_events).collect();
        assert_eq!(emitted.len(), 1);

        let chronicle = emitted[0];
        assert_eq!(chronicle.importance, EventImportance::Major);
        assert!(chronicle.text.contains("Grief tourists arrived offering 50000"));
    }
}
