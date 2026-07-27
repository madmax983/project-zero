#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::core::integration::subterranean_biosphere_chronicle_bridge;
    use scale::layer1::nature::biosphere_inversion::HazardFlora;

    #[test]
    fn test_subterranean_biosphere_chronicle_bridge() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<AddChronicleEvent>();
        app.add_systems(Update, subterranean_biosphere_chronicle_bridge);

        app.world_mut().spawn(HazardFlora {
            lethality: 10.0,
            spread_rate: 2.0,
        });

        app.update();

        let events = app
            .world()
            .get_resource::<Events<AddChronicleEvent>>()
            .unwrap();
        let mut reader = events.get_cursor();
        let mut found = false;
        for ev in reader.read(events) {
            if ev.text.contains("Deep crust mining") {
                assert_eq!(ev.importance, EventImportance::Major);
                found = true;
            }
        }
        assert!(
            found,
            "Bridge should emit AddChronicleEvent when HazardFlora is spawned."
        );
    }
}
