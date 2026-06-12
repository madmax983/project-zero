use scale::layer1::architecture::edible::EdibleBuildingConsumedEvent;
use scale::layer1::core::chronicle::AddChronicleEvent;
use bevy_ecs::prelude::*;

#[test]
fn test_edible_architecture_chronicle_bridge() {
    let mut app = bevy_app::App::new();
    app.init_resource::<Events<EdibleBuildingConsumedEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();
    app.add_systems(bevy_app::Update, scale::layer1::core::integration::edible_architecture_chronicle_bridge);
    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().resource_mut::<Events<EdibleBuildingConsumedEvent>>().send(EdibleBuildingConsumedEvent { entity });
    app.update();
    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = events.get_cursor();
    assert_eq!(cursor.read(events).count(), 1);
}
