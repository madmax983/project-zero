use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::navigation::stellar_weather::FleetDamagedEvent;
use scale::layer3::integration::jump_risk_bridge_system;
use scale::layer3::stellar_cartography::JumpRisk;

#[test]
fn test_jump_risk_bridge() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_event::<FleetDamagedEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(Update, jump_risk_bridge_system);

    let fleet_entity = app.world_mut().spawn(JumpRisk).id();

    app.update();

    // Verify JumpRisk is removed
    assert!(
        app.world().get::<JumpRisk>(fleet_entity).is_none(),
        "JumpRisk should be removed after applying damage"
    );

    // Verify FleetDamagedEvent was sent
    let damage_events = app.world().resource::<Events<FleetDamagedEvent>>();
    let mut reader = damage_events.get_cursor();
    let events: Vec<&FleetDamagedEvent> = reader.read(damage_events).collect();

    assert_eq!(
        events.len(),
        1,
        "Should emit exactly one FleetDamagedEvent for the risky jump"
    );
    assert_eq!(events[0].fleet, fleet_entity);
    assert_eq!(events[0].amount, 20.0);

    // Verify AddChronicleEvent was sent
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut chron_reader = chronicle_events.get_cursor();
    let chron_events: Vec<&AddChronicleEvent> = chron_reader.read(chronicle_events).collect();

    assert_eq!(
        chron_events.len(),
        1,
        "Should emit exactly one AddChronicleEvent for the risky jump"
    );
    assert_eq!(chron_events[0].importance, EventImportance::Major);
}
