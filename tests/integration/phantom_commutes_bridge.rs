use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::architecture::BuildingType;
use scale::layer1::core::events::BuildingRemovedEvent;
use scale::layer1::core::integration::phantom_commutes_bridge_system;
use scale::layer1::execution::components::{HabituatedRoute, MovementTarget};
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::utility_types::ActionType;

#[test]
fn test_building_removed_triggers_phantom_commute() {
    let mut app = App::new();
    app.init_resource::<Events<BuildingRemovedEvent>>();
    app.init_resource::<Events<scale::layer1::core::chronicle::AddChronicleEvent>>();
    app.add_systems(Update, phantom_commutes_bridge_system);

    let building_pos = GridPosition { x: 5, y: 5 };

    let pop = app
        .world_mut()
        .spawn((
            Pop,
            GridPosition { x: 4, y: 5 },
            MovementTarget {
                target_entity: Entity::PLACEHOLDER,
                target_position: building_pos,
                for_action: ActionType::Work,
            },
        ))
        .id();

    // Fire BuildingRemovedEvent for a conveyor belt
    app.world_mut()
        .resource_mut::<Events<BuildingRemovedEvent>>()
        .send(BuildingRemovedEvent {
            entity: Entity::PLACEHOLDER,
            position: building_pos,
            building_type: BuildingType::ConveyorBelt,
        });

    app.update();

    let route = app.world().get::<HabituatedRoute>(pop);
    assert!(
        route.is_some(),
        "Pop should receive a HabituatedRoute when targeting a removed transit building."
    );

    let route = route.unwrap();
    assert_eq!(route.path.len(), 2);
    assert_eq!(route.path[0], GridPosition { x: 4, y: 5 });
    assert_eq!(route.path[1], building_pos);
}

#[test]
fn test_phantom_commute_emits_chronicle_event() {
    use scale::layer1::core::chronicle::AddChronicleEvent;

    let mut app = App::new();
    app.init_resource::<Events<BuildingRemovedEvent>>();
    app.init_resource::<Events<scale::layer1::core::chronicle::AddChronicleEvent>>();
    app.init_resource::<Events<AddChronicleEvent>>();
    app.add_systems(Update, phantom_commutes_bridge_system);

    let building_pos = GridPosition { x: 5, y: 5 };

    app.world_mut()
        .spawn((
            Pop,
            GridPosition { x: 4, y: 5 },
            MovementTarget {
                target_entity: Entity::PLACEHOLDER,
                target_position: building_pos,
                for_action: ActionType::Work,
            },
        ));

    app.world_mut()
        .resource_mut::<Events<BuildingRemovedEvent>>()
        .send(BuildingRemovedEvent {
            entity: Entity::PLACEHOLDER,
            position: building_pos,
            building_type: BuildingType::ConveyorBelt,
        });

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut cursor = chronicle_events.get_cursor();
    let events: Vec<_> = cursor.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Should emit exactly one chronicle event");
    assert!(events[0].text.contains("phantom commute"));
    assert!(events[0].text.contains("5,5"));
}
