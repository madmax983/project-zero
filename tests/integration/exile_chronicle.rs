use bevy::prelude::*;
use scale::layer1::chronicle::{chronicle_event_handler_system, AddChronicleEvent, Chronicle};
use scale::layer1::integration::exile_returned_chronicle_bridge;
use scale::layer1::social::exile::{evaluate_exile_returns, ExileReturnedEvent, ExiledPop};
use scale::shared::time::SimulationTime;

#[test]
fn test_exile_chronicle_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<Chronicle>();
    app.init_resource::<SimulationTime>();
    app.add_event::<ExileReturnedEvent>();
    app.add_event::<AddChronicleEvent>();

    // Register systems in sequence
    app.add_systems(
        Update,
        (
            evaluate_exile_returns,
            exile_returned_chronicle_bridge,
            chronicle_event_handler_system,
        )
            .chain(),
    );

    // Setup an exiled pop that should return immediately (time has passed)
    let pop_entity = app
        .world_mut()
        .spawn((ExiledPop {
            exiled_at_tick: 0,
            base_crime_severity: 5,
            has_returned: false,
            return_role: None,
        },))
        .id();

    // Fast forward time
    app.world_mut().resource_mut::<SimulationTime>().tick = 1_000_000;

    app.update();

    let exiled_pop = app.world().get::<ExiledPop>(pop_entity).unwrap();
    assert!(exiled_pop.has_returned, "ExiledPop should have returned.");

    let chronicle = app.world().resource::<Chronicle>();

    assert!(
        !chronicle.events.is_empty(),
        "Chronicle should have received the return event."
    );

    let last_event = chronicle.events.last().unwrap();
    assert!(
        last_event.text.contains("An exile has returned from the void as a Pirate."),
        "Chronicle text should match expected."
    );
}
