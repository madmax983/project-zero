#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer3::diplomacy::succession::{
        CurrentLeader, Faction, Leader, SuccessionCrisis,
    };
    use scale::layer3::integration::dynastic_succession_chronicle_bridge;

    #[test]
    fn test_dynastic_succession_bridge() {
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<Events<AddChronicleEvent>>();

        app.add_systems(bevy::app::Update, dynastic_succession_chronicle_bridge);

        let leader1 = app
            .world_mut()
            .spawn(Leader {
                name: "King Old".to_string(),
            })
            .id();

        let leader2 = app
            .world_mut()
            .spawn(Leader {
                name: "Queen New".to_string(),
            })
            .id();

        let faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "House of Scale".to_string(),
                },
                CurrentLeader(leader1),
            ))
            .id();

        app.update(); // Flush initial Added/Changed states
        app.world_mut().clear_trackers();

        // Act: leader changes (simulating what process_succession_system does)
        app.world_mut()
            .entity_mut(faction)
            .insert(CurrentLeader(leader2));

        app.update();

        // Check if an event was emitted
        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let ev = reader.read(events).last().expect("Missing AddChronicleEvent!");

        assert_eq!(ev.importance, EventImportance::Major);
        assert!(ev.text.contains("Queen New"));
        assert!(ev.text.contains("House of Scale"));
        assert!(ev.text.contains("took the throne"));
    }

    #[test]
    fn test_dynastic_succession_crisis_bridge() {
        let mut app = bevy::app::App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.init_resource::<Events<AddChronicleEvent>>();

        app.add_systems(bevy::app::Update, dynastic_succession_chronicle_bridge);

        let leader1 = app
            .world_mut()
            .spawn(Leader {
                name: "King Old".to_string(),
            })
            .id();

        let faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "House of Scale".to_string(),
                },
                CurrentLeader(leader1),
            ))
            .id();

        app.update(); // Flush
        app.world_mut().clear_trackers();

        // Act: succession crisis
        app.world_mut().entity_mut(faction).insert(SuccessionCrisis);

        app.update();

        let events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let ev = reader.read(events).last().expect("Missing AddChronicleEvent!");

        assert_eq!(ev.importance, EventImportance::Major);
        assert!(ev.text.contains("House of Scale"));
        assert!(ev.text.contains("Succession crisis"));
    }
}
