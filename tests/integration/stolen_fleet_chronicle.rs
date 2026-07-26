use bevy::prelude::*;
use scale::layer2::stolen_fleet::WarDeclarationEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::integration::stolen_fleet_chronicle_bridge_system;

#[test]
fn test_stolen_fleet_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<WarDeclarationEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, stolen_fleet_chronicle_bridge_system);

    app.world_mut().send_event(WarDeclarationEvent { enemy_id: 1 });
    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert_eq!(emitted[0].text, "A massive fleet has defected to us! A Punitive War has been declared by their former empire.");
}
