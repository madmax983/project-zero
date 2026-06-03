use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::phantom_signal_chronicle_bridge;
use scale::layer2::phantom_signal::SignalRevealEvent;

#[test]
fn test_phantom_signal_ambush_chronicle_integration() {
    let mut app = App::new();

    app.add_event::<SignalRevealEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(bevy_app::Update, phantom_signal_chronicle_bridge);

    app.world_mut()
        .send_event(SignalRevealEvent { is_ambush: true });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected exactly 1 AddChronicleEvent");
    assert!(
        events[0].text.contains("ambush"),
        "Expected chronicle text to mention an ambush"
    );
}

#[test]
fn test_phantom_signal_cache_chronicle_integration() {
    let mut app = App::new();

    app.add_event::<SignalRevealEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(bevy_app::Update, phantom_signal_chronicle_bridge);

    app.world_mut()
        .send_event(SignalRevealEvent { is_ambush: false });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected exactly 1 AddChronicleEvent");
    assert!(
        events[0].text.contains("cache"),
        "Expected chronicle text to mention a cache"
    );
}
