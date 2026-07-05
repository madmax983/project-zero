use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::hive_mind_integration::{IntegratedCollective, SurgeryEvent};
use scale::layer1::pop::{Pop, PopName};
use scale::layer1::core::integration::{hive_mind_chronicle_bridge, integrated_collective_needs_bridge};

fn verify_event(mut reader: EventReader<AddChronicleEvent>) {
    assert_eq!(reader.len(), 1, "Should emit exactly one chronicle event");
    for event in reader.read() {
        assert!(event.text.contains("Collective"));
    }
}

#[test]
fn test_hive_mind_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<AddChronicleEvent>>();
    app.add_event::<SurgeryEvent>();

    app.add_systems(
        Update,
        (hive_mind_chronicle_bridge, verify_event).chain(),
    );

    let pop = app.world_mut().spawn((Pop, PopName("John Doe".to_string()))).id();

    app.world_mut().resource_mut::<Events<SurgeryEvent>>().send(SurgeryEvent {
        patient: pop,
        procedure: "XenoIntegration".to_string(),
    });

    app.update();
}

#[test]
fn test_integrated_collective_needs_bridge() {
    let mut app = App::new();

    app.add_systems(
        Update,
        integrated_collective_needs_bridge,
    );

    let pop = app.world_mut().spawn((
        IntegratedCollective,
        scale::layer1::psychology::needs::Needs { hunger: 0.5, rest: 0.1, leisure: 0.1, hygiene: 0.5 },
    )).id();

    app.update();

    let needs = app.world().get::<scale::layer1::psychology::needs::Needs>(pop).unwrap();
    assert_eq!(needs.rest, 1.0, "Rest should be set to 1.0");
    assert_eq!(needs.leisure, 1.0, "Leisure should be set to 1.0");
}
