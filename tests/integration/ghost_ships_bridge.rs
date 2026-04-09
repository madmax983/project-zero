use bevy::prelude::*;
use scale::layer2::fleet::{Fleet, InTransit};
use scale::layer3::ghost_ships::{EvaluateLostShipReturnEvent, EvaluateTransitEvent, LostInTransit};
use scale::layer3::integration::ghost_ships_bridge_system;

#[test]
fn test_ghost_ships_bridge_emits_evaluate_transit_event() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_event::<EvaluateTransitEvent>();
    app.add_event::<EvaluateLostShipReturnEvent>();
    app.add_systems(Update, ghost_ships_bridge_system);

    let ship_entity = app.world_mut().spawn((
        Fleet,
        InTransit {
            origin: Entity::PLACEHOLDER,
            destination: Entity::PLACEHOLDER,
            progress: 0.1,
            duration: 10.0,
        },
    )).id();

    app.update();

    let events = app.world().resource::<Events<EvaluateTransitEvent>>();
    let mut reader = events.get_cursor();
    let transit_events: Vec<&EvaluateTransitEvent> = reader.read(events).collect();

    assert_eq!(
        transit_events.len(),
        1,
        "Should emit exactly one EvaluateTransitEvent for the ship in transit"
    );
    assert_eq!(transit_events[0].ship, ship_entity);
}

#[test]
fn test_ghost_ships_bridge_emits_evaluate_return_event() {
    let mut app = App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.add_event::<EvaluateTransitEvent>();
    app.add_event::<EvaluateLostShipReturnEvent>();
    app.add_systems(Update, ghost_ships_bridge_system);

    let ship_entity = app.world_mut().spawn((
        Fleet,
        LostInTransit {
            cycles_lost: 10,
        },
    )).id();

    app.update();

    let events = app.world().resource::<Events<EvaluateLostShipReturnEvent>>();
    let mut reader = events.get_cursor();
    let return_events: Vec<&EvaluateLostShipReturnEvent> = reader.read(events).collect();

    assert_eq!(
        return_events.len(),
        1,
        "Should emit exactly one EvaluateLostShipReturnEvent for the lost ship"
    );
    assert_eq!(return_events[0].ship, ship_entity);
}
