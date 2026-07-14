use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::integration::hyperlane_collapse_chronicle_bridge;
use scale::layer3::map::TradeRouteSeveredEvent;

#[test]
fn test_hyperlane_collapse_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_event::<TradeRouteSeveredEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, hyperlane_collapse_chronicle_bridge);

    let sys_a = app.world_mut().spawn_empty().id();
    let sys_b = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(TradeRouteSeveredEvent {
        system_a: sys_a,
        system_b: sys_b,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Should emit exactly one AddChronicleEvent for the hyperlane collapse"
    );
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("stellar drift"), "Chronicle event should mention stellar drift");
}
