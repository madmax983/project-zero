#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer3::silence::HostileSpawnEvent;
    use scale::layer1::integration::hostile_spawn_chronicle_bridge;

    #[test]
    fn hostile_spawn_event_emits_major_chronicle() {
        let mut app = bevy_app::App::new();
        app.add_event::<HostileSpawnEvent>();
        app.add_event::<AddChronicleEvent>();

        app.add_systems(bevy_app::Update, hostile_spawn_chronicle_bridge);

        app.world_mut().send_event(HostileSpawnEvent {
            severity: 5,
        });

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let events: Vec<_> = reader.read(chronicle_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].importance, EventImportance::Major);
        assert!(events[0].text.contains("Hostile forces have spawned"));
    }
}
