use bevy_app::{App, Update};
use bevy_ecs::event::Events;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::diplomacy::retro_contracts::{
    AcceptRetroContractEvent, RetroContractFailedEvent,
};
use scale::layer3::integration::{
    retro_contract_accepted_bridge_system, retro_contract_failed_bridge_system,
};

#[test]
fn test_retro_contract_accepted_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AcceptRetroContractEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, retro_contract_accepted_bridge_system);

    app.world_mut().send_event(AcceptRetroContractEvent {
        credit_advance: 1000.0,
        deadline_days: 10.0,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(matches!(emitted[0].importance, EventImportance::Major));
    assert!(emitted[0].text.contains("1000"));
}

#[test]
fn test_retro_contract_failed_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<RetroContractFailedEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, retro_contract_failed_bridge_system);

    app.world_mut()
        .send_event(RetroContractFailedEvent { penalty: 5000.0 });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(matches!(emitted[0].importance, EventImportance::Major));
    assert!(emitted[0].text.contains("5000"));
}
