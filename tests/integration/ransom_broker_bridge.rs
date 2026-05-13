use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::ransom_broker_chronicle_bridge;
use scale::layer1::ransom_broker::{PopLostToPiratesEvent, PopRansomedEvent};

#[test]
fn test_ransom_broker_chronicle_bridge() {
    let mut app = bevy_app::App::new();
    app.add_event::<PopRansomedEvent>();
    app.add_event::<PopLostToPiratesEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, ransom_broker_chronicle_bridge);

    app.world_mut().send_event(PopRansomedEvent {
        target_pop: Entity::PLACEHOLDER,
    });
    app.world_mut().send_event(PopLostToPiratesEvent {
        target_pop: Entity::PLACEHOLDER,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = events.get_reader();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 2);
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("ransomed"));
    assert_eq!(emitted[1].importance, EventImportance::Major);
    assert!(emitted[1].text.contains("lost to pirates"));
}
