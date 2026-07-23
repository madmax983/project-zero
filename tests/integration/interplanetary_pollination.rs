#[cfg(test)]
mod integration_tests {
    use bevy::prelude::*;
    use scale::cross_layer::interplanetary_pollination::{
        spore_escape_system, spore_infection_system, Biome, FloraCultivation, Planet,
        SporeReleaseEvent,
    };
    use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer2::integration::interplanetary_pollination_chronicle_bridge;

    #[test]
    fn test_interplanetary_pollination_bridge() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<SporeReleaseEvent>();
        app.add_event::<AddChronicleEvent>();
        app.add_systems(
            Update,
            (
                spore_escape_system,
                spore_infection_system,
                interplanetary_pollination_chronicle_bridge,
            )
                .chain(),
        );

        let source_planet = app
            .world_mut()
            .spawn((
                Planet { gravity: 0.5 },
                FloraCultivation {
                    amount: 1000.0,
                    is_gmo: true,
                },
            ))
            .id();
        let target_planet = app
            .world_mut()
            .spawn((
                Planet { gravity: 1.0 },
                Biome {
                    invasive_flora: 0.0,
                },
            ))
            .id();

        app.update();

        // 1) `spore_escape_system` should release SporeReleaseEvent (target=None)
        let spore_events_res = app.world().get_resource::<Events<SporeReleaseEvent>>().unwrap();
        let mut binding = spore_events_res.get_cursor();
        let release_events: Vec<_> = binding.read(spore_events_res).collect();
        assert_eq!(release_events.len(), 1);
        assert_eq!(release_events[0].source_planet, source_planet);

        let strength = release_events[0].spore_strength;

        // Manually send an event to simulate travel to the target planet
        app.world_mut().send_event(SporeReleaseEvent {
            source_planet,
            target_planet: Some(target_planet),
            spore_strength: strength,
        });

        app.update();

        // 2) `spore_infection_system` should update Biome.invasive_flora
        let biome = app.world().get::<Biome>(target_planet).unwrap();
        assert_eq!(biome.invasive_flora, strength);

        // 3) `interplanetary_pollination_chronicle_bridge` should log it
        let chron_events_res = app.world().get_resource::<Events<AddChronicleEvent>>().unwrap();
        let mut binding_chron = chron_events_res.get_cursor();
        let chron_events: Vec<_> = binding_chron.read(chron_events_res).collect();
        assert!(chron_events.len() > 0);
        let last_event = chron_events.last().unwrap();
        assert_eq!(last_event.importance, EventImportance::Major);
        assert!(last_event.text.contains("Interplanetary Pollination"));
    }
}
