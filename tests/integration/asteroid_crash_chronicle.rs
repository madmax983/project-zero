use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer2::integration::asteroid_crash_chronicle_bridge;
use scale::layer2::orbit::tether::AsteroidCrashEvent;

#[test]
fn test_asteroid_crash_chronicle_bridge() {
    let mut app = bevy_app::App::new();
    app.add_event::<AsteroidCrashEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, asteroid_crash_chronicle_bridge);

    app.world_mut().send_event(AsteroidCrashEvent {
        tether_entity: Entity::from_raw(1),
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("crashing into the planet"));
}
