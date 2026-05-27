use bevy_app::App;
use bevy::MinimalPlugins;
use bevy_app::Update;
use scale::layer1::resources::ResourceType;
use scale::layer2::mining::{CargoStack, FleetCargo};
use scale::layer2::station::{Station, StationType, ShipConstructionCompletedEvent};
use scale::layer2::station::{ShipConstruction, process_drydock_construction_system};

#[test]
fn test_orbital_drydock_construction_progress() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, process_drydock_construction_system);

    let required_metal = 1000.0;

    let drydock_entity = app.world_mut().spawn((
        Station { station_type: StationType::OrbitalDrydock },
        ShipConstruction {
            target_ship_class: "Dreadnought".to_string(),
            metal_required: required_metal,
            metal_delivered: 0.0,
            is_complete: false,
        },
    )).id();

    // Deliver some cargo to the drydock
    app.world_mut().entity_mut(drydock_entity).insert(FleetCargo {
        contents: vec![CargoStack {
            resource_type: ResourceType::Metal,
            amount: 500.0,
        }],
        capacity: 2000.0,
    });

    // Act
    app.update();

    // Assert
    let construction = app.world().get::<ShipConstruction>(drydock_entity).unwrap();
    assert_eq!(construction.metal_delivered, 500.0);
    assert_eq!(construction.is_complete, false);

    let cargo = app.world().get::<FleetCargo>(drydock_entity).unwrap();
    assert_eq!(cargo.contents.iter().find(|s| s.resource_type == ResourceType::Metal).map(|s| s.amount).unwrap_or(0.0), 0.0);

    // Deliver the rest
    app.world_mut().get_mut::<FleetCargo>(drydock_entity).unwrap().contents.push(CargoStack {
        resource_type: ResourceType::Metal,
        amount: 500.0,
    });

    app.update();

    // Assert Completion
    let construction = app.world().get::<ShipConstruction>(drydock_entity).unwrap();
    assert_eq!(construction.metal_delivered, 1000.0);
    assert_eq!(construction.is_complete, true);
}
