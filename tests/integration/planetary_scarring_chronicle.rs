use bevy_app::App;
use bevy_ecs::event::Events;
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::planetary_scarring_chronicle_bridge;
use scale::layer1::disasters::mega_event::MegaEvent;

fn verify_event(mut reader: EventReader<AddChronicleEvent>) {
    let mut count = 0;
    for event in reader.read() {
        count += 1;
        assert!(event.text.contains("Planetary scarring detected"));
        match event.importance {
            EventImportance::Major => {}
            _ => panic!("Expected importance Major"),
        }
    }
    assert_eq!(count, 1, "Expected exactly 1 AddChronicleEvent");
}

fn verify_no_event(mut reader: EventReader<AddChronicleEvent>) {
    let count = reader.read().count();
    assert_eq!(count, 0, "Expected exactly 0 AddChronicleEvent");
}

#[test]
fn test_planetary_scarring_chronicle_bridge() {
    let mut app = App::new();

    app.init_resource::<Events<AddChronicleEvent>>();
    app.init_resource::<Events<MegaEvent>>();
    app.add_systems(
        bevy_app::Update,
        (planetary_scarring_chronicle_bridge, verify_event).chain(),
    );

    let planet = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Events<MegaEvent>>()
        .send(MegaEvent {
            planet_entity: planet,
            event_type: "NuclearBlast".to_string(),
            intensity: 80.0,
        });

    app.update();
}

#[test]
fn test_planetary_scarring_chronicle_bridge_low_intensity() {
    let mut app = App::new();

    app.init_resource::<Events<AddChronicleEvent>>();
    app.init_resource::<Events<MegaEvent>>();
    app.add_systems(
        bevy_app::Update,
        (planetary_scarring_chronicle_bridge, verify_no_event).chain(),
    );

    let planet = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Events<MegaEvent>>()
        .send(MegaEvent {
            planet_entity: planet,
            event_type: "MegaFire".to_string(),
            intensity: 40.0,
        });

    app.update();
}
