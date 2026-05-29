use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::kinetic_battery_chronicle_bridge;
use scale::layer1::health::Dead;
use scale::layer1::map::GridPosition;
use scale::layer1::kinetic_storage::{
    handle_battery_destruction_system, KineticBattery,
};

#[test]
fn test_kinetic_storage_catastrophe_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<scale::layer1::environment::volatile::ExplosionEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (
            handle_battery_destruction_system,
            kinetic_battery_chronicle_bridge,
        )
            .chain(),
    );

    let battery = app
        .world_mut()
        .spawn((
            KineticBattery {
                charge: 100.0,
                capacity: 100.0,
                charge_rate: 5.0,
                efficiency: 0.9,
            },
            GridPosition { x: 5, y: 5 },
        ))
        .id();

    app.world_mut().entity_mut(battery).insert(Dead);

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected exactly one Chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(
        events[0].text.contains("structural failure"),
        "Chronicle text should describe a kinetic battery structural failure"
    );
}
