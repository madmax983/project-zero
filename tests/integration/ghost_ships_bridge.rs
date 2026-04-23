use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer3::ghost_ships::{
    evaluate_lost_ship_return_system, evaluate_transit_system, EvaluateLostShipReturnEvent,
    EvaluateTransitEvent, GhostShip, LostInTransit,
};

#[derive(Component)]
struct Ship;

#[test]
#[allow(deprecated)]
fn test_ghost_ships_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<EvaluateTransitEvent>();
    app.add_event::<EvaluateLostShipReturnEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (evaluate_transit_system, evaluate_lost_ship_return_system).chain(),
    );

    let ship = app.world_mut().spawn(Ship).id();

    // Simulate transit event
    app.world_mut().send_event(EvaluateTransitEvent { ship });
    app.update();

    // Check if LostInTransit was added
    assert!(app.world().get::<LostInTransit>(ship).is_some());

    // Simulate return event
    app.world_mut()
        .send_event(EvaluateLostShipReturnEvent { ship });
    app.update();

    // Check if GhostShip is added and LostInTransit is removed
    assert!(app.world().get::<LostInTransit>(ship).is_none());
    assert!(app.world().get::<GhostShip>(ship).is_some());

    // Check if AddChronicleEvent was sent
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_reader();
    assert!(reader.read(chronicle_events).next().is_some());
}
