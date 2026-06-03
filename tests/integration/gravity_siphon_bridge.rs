#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::energy::gravity_siphon::OrbitalDecayEvent;

    #[test]
    fn gravity_siphon_bridge_triggers_chronicle() {
        let mut app = App::new();

        app.add_event::<OrbitalDecayEvent>();
        app.add_event::<AddChronicleEvent>();

        // Add the bridge system
        app.add_systems(Update, scale::layer1::core::integration::gravity_siphon_chronicle_bridge);

        // Send an orbital decay event
        app.world_mut().send_event(OrbitalDecayEvent { anomaly_strength: 10000.0 });

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

        assert_eq!(events.len(), 1, "Should emit exactly one chronicle event");
        assert!(matches!(events[0].importance, EventImportance::Legendary));
        assert!(events[0].text.contains("Micro-Singularity Generator"));
    }
}
