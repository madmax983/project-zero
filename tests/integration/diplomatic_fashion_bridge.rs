use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::entities::pop::Pop;
use scale::layer3::diplomacy::diplomatic_fashion::{
    Apparel, AttireTag, Diplomat, DiplomaticMeetingEvent, PreferredAttire,
};
use scale::layer3::integration::diplomatic_fashion_chronicle_bridge;

#[test]
fn test_diplomatic_fashion_chronicle_match() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<DiplomaticMeetingEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, diplomatic_fashion_chronicle_bridge);

    let ambassador = app
        .world_mut()
        .spawn((
            Diplomat {
                civ_id: "alien_empire".to_string(),
            },
            PreferredAttire {
                tags: vec![AttireTag::Organic],
            },
        ))
        .id();

    let envoy = app
        .world_mut()
        .spawn((
            Pop,
            Apparel {
                tags: vec![AttireTag::Organic],
            },
        ))
        .id();

    app.world_mut().send_event(DiplomaticMeetingEvent {
        ambassador,
        envoy,
        player_civ_id: "player".to_string(),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one chronicle event");
    assert_eq!(emitted[0].importance, EventImportance::Standard);
    assert!(emitted[0].text.contains("went well"));
}

#[test]
fn test_diplomatic_fashion_chronicle_mismatch() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<DiplomaticMeetingEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, diplomatic_fashion_chronicle_bridge);

    let ambassador = app
        .world_mut()
        .spawn((
            Diplomat {
                civ_id: "alien_empire".to_string(),
            },
            PreferredAttire {
                tags: vec![AttireTag::HeavyArmor],
            },
        ))
        .id();

    let envoy = app
        .world_mut()
        .spawn((
            Pop,
            Apparel {
                tags: vec![AttireTag::Organic],
            },
        ))
        .id();

    app.world_mut().send_event(DiplomaticMeetingEvent {
        ambassador,
        envoy,
        player_civ_id: "player".to_string(),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<&AddChronicleEvent> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one chronicle event");
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("offended"));
}
