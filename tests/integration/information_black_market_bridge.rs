use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::economy::information_black_market::EarlyWarningEvent;
use scale::layer1::core::integration::early_warning_chronicle_bridge;

#[test]
fn test_early_warning_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<EarlyWarningEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, early_warning_chronicle_bridge);

    app.world_mut().send_event(EarlyWarningEvent {
        sector_id: 42,
        event_type: "Impending Invasion".to_string(),
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("Impending Invasion"));
}
