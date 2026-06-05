#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use scale::layer1::chronicle::AddChronicleEvent;
    use scale::layer3::diplomacy::dead_internet::{
        automated_diplomat_system, AutomatedDiplomat, DiplomaticInteraction,
        GenerateDiplomaticInteractionEvent,
    };
    use scale::layer3::diplomacy::succession::Faction;
    use scale::layer3::integration::dead_internet_chronicle_bridge;

    #[test]
    fn test_dead_internet_chronicle_bridge() {
        let mut app = App::new();

        app.add_event::<DiplomaticInteraction>();
        app.add_event::<GenerateDiplomaticInteractionEvent>();
        app.add_event::<AddChronicleEvent>();

        app.add_systems(
            Update,
            (automated_diplomat_system, dead_internet_chronicle_bridge).chain(),
        );

        let faction = app
            .world_mut()
            .spawn((
                Faction {
                    name: "ScriptBot".to_string(),
                },
                AutomatedDiplomat,
            ))
            .id();

        app.world_mut()
            .send_event(GenerateDiplomaticInteractionEvent { faction });

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        assert!(
            !chronicle_events.is_empty(),
            "Automated diplomatic interactions should trigger a chronicle event"
        );
    }
}
