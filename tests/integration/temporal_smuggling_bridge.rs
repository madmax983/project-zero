#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::tech::temporal_smuggling::ParadoxEvent;
    use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::core::integration::temporal_smuggling_chronicle_bridge;

    #[test]
    fn test_temporal_smuggling_chronicle_bridge() {
        let mut world = World::new();
        world.init_resource::<Events<ParadoxEvent>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        let mut schedule = Schedule::default();
        schedule.add_systems(temporal_smuggling_chronicle_bridge);

        world.send_event(ParadoxEvent);

        schedule.run(&mut world);

        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let chronicle_events: Vec<_> = reader.read(events).collect();

        assert_eq!(chronicle_events.len(), 1);
        assert_eq!(chronicle_events[0].text, "A Temporal Paradox has occurred due to defaulted debt!");
        assert_eq!(chronicle_events[0].importance, EventImportance::Major);
    }
}
