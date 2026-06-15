use bevy::prelude::*;
use scale::layer1::architecture::Building;
use scale::layer1::architecture::BuildingType;
use scale::layer1::architecture_superstition::NegativeEventHistory;
use scale::layer1::core::events::BuildingRemovedEvent;
use scale::layer1::haunted_assembly_lines::PopDiedInAccidentEvent;
use scale::layer1::map::GridPosition;
use scale::layer1::pop::PopDied;

#[test]
fn test_architectural_superstition_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add the components needed
    app.add_event::<PopDied>();
    app.add_event::<BuildingRemovedEvent>();
    app.add_event::<PopDiedInAccidentEvent>();

    // Register the new system
    app.init_resource::<scale::shared::time::SimulationTime>();
    app.add_systems(
        Update,
        scale::layer1::core::integration::track_negative_events_bridge_system,
    );

    // Spawn a building that should get cursed
    let target_building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 10, y: 10 },
            NegativeEventHistory::default(),
        ))
        .id();

    // Spawn another building far away that shouldn't get cursed
    let far_building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 50, y: 50 },
            NegativeEventHistory::default(),
        ))
        .id();

    // Emulate PopDied (we spawn a pop to get its GridPosition)
    let dead_pop = app.world_mut().spawn(GridPosition { x: 12, y: 11 }).id();
    app.world_mut().send_event(PopDied {
        entity: dead_pop,
        name: "TestPop".to_string(),
        tick: 1,
        reason: "Unknown".to_string(),
    });

    // Emulate BuildingRemovedEvent
    app.world_mut().send_event(BuildingRemovedEvent {
        entity: Entity::from_raw(999),
        position: GridPosition { x: 8, y: 9 },
        building_type: BuildingType::Housing,
    });

    // Emulate PopDiedInAccidentEvent
    let factory_building = app
        .world_mut()
        .spawn((
            Building {
                building_type: BuildingType::Farm,
            },
            GridPosition { x: 10, y: 11 },
        ))
        .id();
    app.world_mut().send_event(PopDiedInAccidentEvent {
        pop: Entity::from_raw(998),
        location: factory_building,
    });

    app.update();

    // Verify target building got the negative events
    let history = app
        .world()
        .get::<NegativeEventHistory>(target_building)
        .unwrap();
    assert_eq!(
        history.events.len(),
        3,
        "Target building should have received 3 negative events"
    );

    // Verify far building didn't get them
    let far_history = app
        .world()
        .get::<NegativeEventHistory>(far_building)
        .unwrap();
    assert_eq!(
        far_history.events.len(),
        0,
        "Far building should have 0 negative events"
    );
}
