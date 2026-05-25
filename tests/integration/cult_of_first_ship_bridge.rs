use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::first_ship_destruction_chronicle_bridge;
use scale::layer1::culture::cult_of_first_ship::{first_ship_destruction_system, FirstShip};

#[test]
fn first_ship_destruction_causes_unrest_event() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(
        Update,
        (
            first_ship_destruction_system,
            first_ship_destruction_chronicle_bridge,
        )
            .chain(),
    );

    let ship = app.world_mut().spawn(FirstShip { is_intact: true }).id();

    // Initial update to clear removed components
    app.update();

    // Destroy the ship
    app.world_mut().despawn(ship);

    // Run systems
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let events_list: Vec<_> = cursor.read(events).collect();

    assert_eq!(
        events_list.len(),
        1,
        "Should emit exactly one chronicle event"
    );
    assert_eq!(events_list[0].importance, EventImportance::Major);
    assert!(
        events_list[0].text.contains("First Ship"),
        "Event text should mention First Ship"
    );
}
