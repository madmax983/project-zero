#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
    use scale::layer1::void_weed::PirateRaidEvent;
    use scale::layer1::integration::pirate_raid_chronicle_bridge;

    #[test]
    fn pirate_raid_event_emits_major_chronicle() {
        let mut app = bevy_app::App::new();
        app.add_event::<PirateRaidEvent>();
        app.add_event::<AddChronicleEvent>();

        app.add_systems(bevy_app::Update, pirate_raid_chronicle_bridge);

        app.world_mut().send_event(PirateRaidEvent);

        app.update();

        let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
        let mut reader = chronicle_events.get_cursor();
        let events: Vec<_> = reader.read(chronicle_events).collect();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].importance, EventImportance::Major);
        assert!(events[0].text.contains("Pirate Raid"));
    }
}
