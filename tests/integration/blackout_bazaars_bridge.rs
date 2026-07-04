use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::blackout_bazaars::BlackoutBazaar;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::blackout_bazaar_chronicle_bridge;

fn verify_event(mut reader: EventReader<AddChronicleEvent>) {
    assert_eq!(reader.len(), 1, "Should emit exactly one chronicle event");
    for event in reader.read() {
        assert!(event.text.contains("Blackout Bazaar"));
    }
}

#[test]
fn test_blackout_bazaar_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(
        Update,
        (blackout_bazaar_chronicle_bridge, verify_event).chain(),
    );

    app.world_mut().spawn(BlackoutBazaar { active: true });

    app.update();
}
