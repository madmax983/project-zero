use bevy_app::App;
use bevy_ecs::prelude::*;
use scale::layer1::architecture::biomimetic::BiomimeticShiftEvent;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};

#[test]
fn test_sympathetic_infrastructure_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<BiomimeticShiftEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(bevy_app::Update, scale::layer1::core::integration::biomimetic_chronicle_bridge);

    app.world_mut().resource_mut::<Events<BiomimeticShiftEvent>>().send(BiomimeticShiftEvent {
        building: Entity::PLACEHOLDER,
        new_output: 10.0,
        delta: -5.0,
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let iter: Vec<_> = reader.read(events).collect();

    assert_eq!(iter.len(), 1, "Should emit AddChronicleEvent");
    assert_eq!(iter[0].importance, EventImportance::Major);
}
