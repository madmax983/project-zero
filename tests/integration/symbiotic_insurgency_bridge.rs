use bevy::prelude::*;
use scale::layer1::biology::symbiotic_insurgency::{SabotageEvent, SabotageTarget};
use scale::layer1::access_control::{AccessControl, AccessMode};
use scale::layer1::architecture::{Building, BuildingType, Structure};
use scale::layer1::core::chronicle::{Chronicle, AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::symbiont_sabotage_bridge_system;

#[test]
fn test_symbiont_sabotage_bridge_airlocks() {
    let mut app = App::new();
    app.add_event::<SabotageEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, symbiont_sabotage_bridge_system);

    let airlock1 = app.world_mut().spawn((
        Building { building_type: BuildingType::Airlock },
        AccessControl { mode: AccessMode::Lockdown, ..Default::default() },
    )).id();

    let airlock2 = app.world_mut().spawn((
        Building { building_type: BuildingType::Airlock },
        AccessControl { mode: AccessMode::Lockdown, ..Default::default() },
    )).id();

    app.world_mut().send_event(SabotageEvent { target: SabotageTarget::Airlocks });
    app.update();

    // Airlocks should be forced open
    assert_eq!(app.world().get::<AccessControl>(airlock1).unwrap().mode, AccessMode::Public);
    assert_eq!(app.world().get::<AccessControl>(airlock2).unwrap().mode, AccessMode::Public);

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].importance, EventImportance::Major);
}

#[test]
fn test_symbiont_sabotage_bridge_air_filtration() {
    let mut app = App::new();
    app.add_event::<SabotageEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, symbiont_sabotage_bridge_system);

    let filter = app.world_mut().spawn((
        Building { building_type: BuildingType::LifeSupport },
        Structure { current_hp: 100.0, max_hp: 100.0, ..Default::default() },
    )).id();

    app.world_mut().send_event(SabotageEvent { target: SabotageTarget::AirFiltration });
    app.update();

    // Air filtration should take damage
    assert!(app.world().get::<Structure>(filter).unwrap().current_hp < 100.0);

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].importance, EventImportance::Minor);
}
