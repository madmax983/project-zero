#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::nanite_fabrication::ContainmentBreachEvent;
    use scale::layer1::integration::containment_breach_chronicle_bridge;
    use scale::layer1::GridPosition;

    #[test]
    fn containment_breach_event_emits_major_chronicle() {
        let mut app = bevy_app::App::new();
        app.add_event::<ContainmentBreachEvent>();
        app.add_event::<AddChronicleEvent>();

        app.add_systems(bevy_app::Update, containment_breach_chronicle_bridge);

        let entity = app.world_mut().spawn_empty().id();
        app.world_mut().send_event(ContainmentBreachEvent {
            source_entity: entity,
            position: GridPosition { x: 0, y: 0 },
        });

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let events: Vec<_> = reader.read(chronicle_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].importance, EventImportance::Major);
        assert!(events[0].text.contains("Breach"));
    }
}
