#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer2::orbit::tether::AsteroidCrashEvent;
    use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer2::integration::asteroid_crash_chronicle_bridge;

    #[test]
    fn test_asteroid_crash_emits_chronicle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Events<AsteroidCrashEvent>>();
        app.init_resource::<Events<AddChronicleEvent>>();

        app.add_systems(Update, asteroid_crash_chronicle_bridge);

        let entity = app.world_mut().spawn_empty().id();
        app.world_mut().send_event(AsteroidCrashEvent { tether_entity: entity });

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut cursor = chronicle_events.get_cursor();
        let events: Vec<_> = cursor.read(chronicle_events).collect();
        assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
        assert_eq!(events[0].importance, EventImportance::Major);
        assert!(events[0].text.contains("Asteroid crash"));
    }
}
