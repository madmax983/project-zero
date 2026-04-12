use bevy_app::App;
use bevy_app::Update;
use bevy_ecs::prelude::*;
use scale::layer1::administration::edicts::{AccessDeniedEvent, HackCentralHubEvent, Policy};
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::{access_denied_chronicle_bridge, hack_hub_chronicle_bridge};

#[test]
fn test_access_denied_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<AccessDeniedEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, access_denied_chronicle_bridge);

    app.world_mut().send_event(AccessDeniedEvent {
        reason: "Edict ShootInfected is orphaned and cannot be toggled".to_string(),
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(
        events[0].text.contains("Edict ShootInfected is orphaned"),
        "Event text should contain the reason"
    );
}

#[test]
fn test_hack_hub_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<HackCentralHubEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, hack_hub_chronicle_bridge);

    app.world_mut().send_event(HackCentralHubEvent {
        target_policy: Policy::ShootInfected,
    });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit one AddChronicleEvent");
    assert!(
        events[0].text.contains("ShootInfected"),
        "Event text should contain the policy name"
    );
}