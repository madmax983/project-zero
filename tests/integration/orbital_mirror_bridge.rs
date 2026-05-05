use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer2::integration::orbital_mirror_chronicle_bridge;
use scale::layer2::orbital_mirrors::OrbitalMirror;

#[test]
fn test_orbital_mirror_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, orbital_mirror_chronicle_bridge);

    app.world_mut().spawn(OrbitalMirror {
        target: Vec2::new(5.0, 5.0),
        intensity: 10.0,
        radius: 2.0,
        alignment_error: 0.1,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let read_events: Vec<_> = reader.read(events).collect();
    assert_eq!(read_events.len(), 1);
    assert!(read_events[0].text.contains("Orbital Mirror was deployed"));
}
