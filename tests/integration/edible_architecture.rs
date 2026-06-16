use bevy::prelude::*;
use scale::layer1::architecture::building::{Building, BuildingType};
use scale::layer1::architecture::edible::{consume_building_system, Consumed, EdibleMaterial};
use scale::layer1::core::events::BuildingRemovedEvent;

use scale::layer1::economy::resources::ColonyResources;
use scale::layer1::map::GridPosition;

#[test]
fn test_consume_edible_architecture_integration() {
    let mut app = App::new();
    app.add_event::<BuildingRemovedEvent>();
    app.insert_resource(ColonyResources {
        food: 10.0,
        ..Default::default()
    });
    app.add_systems(Update, consume_building_system);

    let building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 5, y: 5 },
            EdibleMaterial { food_yield: 50.0 },
            Consumed, // Mark it for consumption
        ))
        .id();

    app.update();

    // Building should be despawned
    assert!(app.world().get_entity(building).is_err());

    // Food should have increased
    assert_eq!(app.world().resource::<ColonyResources>().food, 60.0);
}

#[test]
fn test_edible_architecture_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<scale::layer1::core::events::BuildingRemovedEvent>();
    app.add_event::<scale::layer1::core::chronicle::AddChronicleEvent>();
    app.insert_resource(ColonyResources::default());
    app.add_systems(
        Update,
        (
            scale::layer1::architecture::edible::consume_building_system,
            scale::layer1::core::integration::edible_architecture_chronicle_bridge,
        ),
    );

    let _building = app
        .world_mut()
        .spawn((
            scale::layer1::architecture::building::Building {
                building_type: scale::layer1::architecture::building::BuildingType::Housing,
            },
            scale::layer1::map::GridPosition { x: 5, y: 5 },
            scale::layer1::architecture::edible::EdibleMaterial { food_yield: 50.0 },
            scale::layer1::architecture::edible::Consumed,
        ))
        .id();

    app.update();

    let chronicle_events = app
        .world()
        .resource::<Events<scale::layer1::core::chronicle::AddChronicleEvent>>();

    let mut iter = chronicle_events.get_cursor();
    let mut found = false;
    for event in iter.read(chronicle_events) {
        if event.text.contains("Edible Architecture") {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "Chronicle event for Edible Architecture should have been emitted"
    );
}
