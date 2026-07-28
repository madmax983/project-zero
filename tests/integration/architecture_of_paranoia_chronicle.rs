#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::core::integration::architecture_of_paranoia_chronicle_bridge;
    use scale::layer1::social::subversion::Surveillance;

    #[test]
    fn test_architecture_of_paranoia_chronicle_bridge() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, architecture_of_paranoia_chronicle_bridge);

        app.world_mut().spawn(Surveillance { radius: 5 });

        app.update();

        let events = app
            .world()
            .get_resource::<Events<AddChronicleEvent>>()
            .unwrap();
        let mut reader = events.get_cursor();
        let mut found = false;
        for ev in reader.read(events) {
            if ev
                .text
                .contains("A new Surveillance installation has been constructed")
            {
                assert_eq!(ev.importance, EventImportance::Major);
                found = true;
            }
        }
        assert!(
            found,
            "Bridge should emit AddChronicleEvent when Surveillance is spawned."
        );
    }
}
