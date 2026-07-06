use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::administration::feral_administration::{UnprocessedForms, FeralColony};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::feral_administration_chronicle_bridge;

#[test]
fn test_feral_administration_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, feral_administration_chronicle_bridge);

    app.world_mut().spawn((UnprocessedForms { stack_size: 10 }, FeralColony));

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert!(emitted[0].text.contains("Feral Administration"));
}
