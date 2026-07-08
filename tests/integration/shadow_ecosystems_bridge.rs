use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::shadow_ecosystems::{
    shadow_ecosystems_short_circuit_bridge, EmMachine, ShortCircuitEvent,
};

#[test]
fn test_shadow_ecosystems_short_circuit_bridge() {
    let mut app = App::new();

    // Register events
    app.add_event::<ShortCircuitEvent>();
    app.add_event::<AddChronicleEvent>();

    // Register the bridge system
    app.add_systems(bevy_app::Update, shadow_ecosystems_short_circuit_bridge);

    // Spawn an active EmMachine
    let machine_entity = app.world_mut().spawn(EmMachine { active: true }).id();

    // Send a ShortCircuitEvent targeting the machine
    app.world_mut()
        .resource_mut::<Events<ShortCircuitEvent>>()
        .send(ShortCircuitEvent {
            target: machine_entity,
        });

    app.update();

    // Verify the machine is now inactive
    let machine = app.world().get::<EmMachine>(machine_entity).unwrap();
    assert!(
        !machine.active,
        "EmMachine should be deactivated by the short circuit bridge"
    );

    // Verify a chronicle event was emitted
    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    assert_eq!(
        reader.len(chronicle_events),
        1,
        "A chronicle event should be created"
    );

    let event = reader.read(chronicle_events).next().unwrap();
    assert!(
        event.text.contains("short-circuited"),
        "Chronicle event text should mention the short circuit"
    );
}
