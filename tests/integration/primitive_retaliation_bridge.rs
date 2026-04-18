#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer2::integration::primitive_retaliation_chronicle_bridge;
    use scale::layer2::primitives::PrimitiveRetaliationEvent;

    #[test]
    fn test_primitive_retaliation_chronicle_bridge() {
        let mut app = App::new();
        app.add_event::<PrimitiveRetaliationEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, primitive_retaliation_chronicle_bridge);

        let entity = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<Events<PrimitiveRetaliationEvent>>()
            .send(PrimitiveRetaliationEvent { target: entity });

        app.update();

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let chronicle_events: Vec<_> = reader.read(events).collect();

        assert_eq!(chronicle_events.len(), 1);
        assert_eq!(chronicle_events[0].importance, EventImportance::Major);
        assert!(chronicle_events[0].text.contains("retaliat"));
    }
}
