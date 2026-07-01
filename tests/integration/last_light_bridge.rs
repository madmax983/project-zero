use bevy::prelude::*;
use scale::layer1::chronicle::{Chronicle, EventImportance};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer3::events::collapse::{process_civilization_collapse_system, process_refugee_arrival_system, CivilizationCollapseEvent, RefugeeFleetArrivalEvent};
use scale::layer1::pop::Pop;
use scale::layer1::morale::Morale;

#[test]
fn test_last_light_bridge() {
    let mut app = App::new();

    app.add_event::<CivilizationCollapseEvent>();
    app.add_event::<RefugeeFleetArrivalEvent>();
    app.add_event::<AddChronicleEvent>();
    app.init_resource::<Chronicle>();

    app.world_mut().spawn((Pop, Morale { value: 0.8, ..default() }));

    app.add_systems(Update, (process_civilization_collapse_system, process_refugee_arrival_system).chain());

    app.world_mut().send_event(CivilizationCollapseEvent {
        civ_id: Entity::PLACEHOLDER,
        population_lost: 1_000_000,
        tech_level: 5,
    });

    app.update();

    let chronicle_events = app
        .world()
        .resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected one chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0]
        .text
        .contains("refugee fleet has arrived"));
}
