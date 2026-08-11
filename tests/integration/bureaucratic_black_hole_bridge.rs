use bevy::prelude::*;
use scale::layer1::administration::bureaucratic_black_hole::BureaucraticBlackHole;
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::bureaucratic_black_hole_chronicle_bridge;

#[test]
fn bureaucratic_black_hole_chronicle_bridge_emits_event() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, bureaucratic_black_hole_chronicle_bridge);

    app.world_mut().spawn(BureaucraticBlackHole { radius: 2.0 });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<&AddChronicleEvent> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("Bureaucratic Black Hole"));
}
