use bevy::prelude::*;
use scale::layer1::chronicle::AddChronicleEvent;
use scale::layer2::bombardment::BombardmentEvent;
use scale::layer2::integration::orbital_bombardment_chronicle_bridge;

#[test]
fn test_orbital_bombardment_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<BombardmentEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, orbital_bombardment_chronicle_bridge);

    app.world_mut().send_event(BombardmentEvent {
        target: Vec2::new(10.0, 10.0),
        damage: 500.0,
        scatter_radius: 5.0,
        blast_radius: 2.0,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let read_events: Vec<_> = reader.read(events).collect();
    assert_eq!(read_events.len(), 1);
    assert!(read_events[0].text.contains("Orbital Bombardment"));
    assert!(read_events[0].text.contains("500"));
    assert!(read_events[0].text.contains("2"));
}
