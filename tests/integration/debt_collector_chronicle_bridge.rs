use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::auditor::debt_collector::AuditorArrivalEvent;
use scale::layer3::integration::auditor_arrival_chronicle_bridge;

#[test]
fn test_auditor_arrival_chronicle_bridge() {
    let mut app = bevy_app::App::new();
    app.add_event::<AuditorArrivalEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, auditor_arrival_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<AuditorArrivalEvent>>()
        .send(AuditorArrivalEvent);

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("An auditor has arrived"));
}
