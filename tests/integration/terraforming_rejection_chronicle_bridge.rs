use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::terraforming_rejection_chronicle_bridge;
use scale::layer1::disasters::{DisasterEvent, DisasterType};
use scale::layer1::core::map::GridPosition;

#[test]
fn test_terraforming_rejection_chronicle_bridge() {
    let mut app = App::new();
    app.init_resource::<Events<DisasterEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();

    app.add_systems(Update, terraforming_rejection_chronicle_bridge);

    app.world_mut()
        .resource_mut::<Events<DisasterEvent>>()
        .send(DisasterEvent {
            disaster_type: DisasterType::Fissure,
            position: GridPosition { x: 5, y: 5 },
        });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    let emitted: Vec<_> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1);
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("autoimmune response"));
}
