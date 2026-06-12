use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::architecture::building::{Building, BuildingType};
use scale::layer1::architecture::edible::{consume_building_system, Consumed, EdibleMaterial};
use scale::layer1::core::chronicle::AddChronicleEvent;
use scale::layer1::core::integration::edible_architecture_chronicle_bridge;
use scale::layer1::economy::resources::ColonyResources;

#[test]
fn test_edible_architecture_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<AddChronicleEvent>();
    app.add_event::<scale::layer1::events::BuildingRemovedEvent>();
    app.insert_resource(ColonyResources::default());

    app.add_systems(
        Update,
        (
            edible_architecture_chronicle_bridge,
            consume_building_system,
        )
            .chain(),
    );

    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Housing,
        },
        EdibleMaterial { food_yield: 50.0 },
        Consumed,
    ));

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert!(
        emitted[0].text.contains("starvation") || emitted[0].text.contains("Housing"),
        "Text should contain narrative about eating the building: {}",
        emitted[0].text
    );
}
