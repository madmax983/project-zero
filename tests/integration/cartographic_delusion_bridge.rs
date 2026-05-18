use bevy::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer3::integration::anomaly_discovered_chronicle_bridge;
use scale::layer3::map::{AnomalyDiscoveredEvent, SectorId};

#[test]
fn test_anomaly_discovered_triggers_chronicle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<AnomalyDiscoveredEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, anomaly_discovered_chronicle_bridge);

    app.world_mut().send_event(AnomalyDiscoveredEvent {
        sector: SectorId(42),
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(events[0].text.contains("Anomaly Discovered"));
    assert!(events[0].text.contains("Sector 42"));
}
