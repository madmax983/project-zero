use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer3::ghost_ships::{
    evaluate_lost_ship_return_system, evaluate_transit_system, EvaluateLostShipReturnEvent,
    EvaluateTransitEvent, GhostShip, LostInTransit,
};

// Mock Ship component just to spawn an entity
#[derive(Component)]
struct Ship;

fn setup_app() -> App {
    let mut app = App::new();

    app.add_event::<EvaluateTransitEvent>();
    app.add_event::<EvaluateLostShipReturnEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (evaluate_transit_system, evaluate_lost_ship_return_system),
    );

    app
}

#[test]
fn test_ghost_ships_bridge() {
    let mut app = setup_app();

    let ship_entity = app
        .world_mut()
        .spawn((Ship, LostInTransit { cycles_lost: 10 }))
        .id();

    // Trigger the return evaluation system
    app.world_mut()
        .send_event(EvaluateLostShipReturnEvent { ship: ship_entity });

    app.update();

    // Verify GhostShip was added
    assert!(
        app.world().get::<GhostShip>(ship_entity).is_some(),
        "Ship should become a GhostShip"
    );

    // Verify LostInTransit was removed
    assert!(
        app.world().get::<LostInTransit>(ship_entity).is_none(),
        "Ship should no longer be LostInTransit"
    );

    // Verify AddChronicleEvent was sent
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();

    let mut events_count = 0;
    for event in reader.read(chronicle_events) {
        assert_eq!(event.importance, EventImportance::Major);
        assert_eq!(event.text, "A ghost ship has returned from the void.");
        events_count += 1;
    }

    assert!(
        events_count > 0,
        "An AddChronicleEvent should have been emitted by the ghost ship return bridge"
    );
}
