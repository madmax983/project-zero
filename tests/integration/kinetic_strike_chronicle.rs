#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::geology::subsurface::KineticStrikeEvent;
    use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::core::integration::kinetic_strike_chronicle_bridge;

    #[test]
    fn test_kinetic_strike_chronicle_bridge() {
        let mut app = App::new();
        app.world_mut().init_resource::<Events<KineticStrikeEvent>>();
        app.world_mut().init_resource::<Events<AddChronicleEvent>>();
        app.add_systems(Update, kinetic_strike_chronicle_bridge);

        app.world_mut().send_event(KineticStrikeEvent {
            target_x: 5,
            target_y: 5,
            accuracy_offset: 0.0,
        });

        app.update();

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut cursor = events.get_cursor();
        let chronicle_events: Vec<_> = cursor.read(events).collect();

        assert_eq!(chronicle_events.len(), 1);
        assert_eq!(chronicle_events[0].importance, EventImportance::Major);
        assert!(chronicle_events[0].text.contains("Kinetic Strike"));
    }
}
