use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::diplomacy::TributeDemandEvent;
use scale::layer3::integration::sovereign_armada_chronicle_bridge;

#[test]
fn test_sovereign_armada_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<TributeDemandEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, sovereign_armada_chronicle_bridge);

    let system_node = app.world_mut().spawn_empty().id();
    let armada = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(TributeDemandEvent {
        aggressor: armada,
        system: system_node,
        amount: 5000,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(
        events.len(),
        1,
        "TributeDemandEvent should trigger AddChronicleEvent for Sovereign Armada"
    );
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Sovereign Armada"));
}
