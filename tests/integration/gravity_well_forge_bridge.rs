#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rand::SeedableRng;
    use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::economy::resources::ColonyResources;
    use scale::layer2::integration::observe_forge_crush_event;
    use scale::layer2::station::ForgeCrushEvent;
    use scale::layer2::station::{process_deep_forges, Crew, DeepForge, MaintenanceLevel};
    use scale::shared::random::GlobalRng;

    #[test]
    fn test_forge_crush_emits_chronicle_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.add_event::<ForgeCrushEvent>();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(GlobalRng(rand::rngs::StdRng::seed_from_u64(42)));
        app.init_resource::<ColonyResources>();

        app.add_systems(
            Update,
            (process_deep_forges, observe_forge_crush_event).chain(),
        );

        app.world_mut().spawn((
            DeepForge {
                production_rate: 10.0,
                base_crush_chance: 1.0,
                is_active: true,
            },
            MaintenanceLevel { current: 0.0 },
            Crew { count: 50 },
        ));

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut cursor = chronicle_events.get_cursor();
        let events: Vec<_> = cursor.read(chronicle_events).collect();

        assert_eq!(
            events.len(),
            1,
            "Expected one AddChronicleEvent to be emitted"
        );
        assert_eq!(events[0].importance, EventImportance::Major);
        assert!(events[0].text.contains("50 casualties"));
    }
}
