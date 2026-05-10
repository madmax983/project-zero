use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::psychology::simulacrum::Simulacrum;

#[test]
fn test_simulacrum_chronicle_integration() {
    let mut app = App::new();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(
        Update,
        scale::layer1::core::integration::simulacrum_chronicle_bridge,
    );

    app.world_mut().spawn(Simulacrum {
        radius: 10.0,
        active: true,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    assert_eq!(events.len(), 1);

    let mut cursor = events.get_cursor();
    let ev = cursor.read(events).next().unwrap();
    assert_eq!(ev.importance, EventImportance::Major);
    assert!(ev.text.contains("Propaganda Simulacrum"));
}
