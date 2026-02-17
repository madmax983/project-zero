// tests/layer1/cannibalization_tests.rs

use bevy_ecs::prelude::*;
use scale::layer1::GridPosition;
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::designation::{Designation, DesignationType};
use scale::layer1::resources::{ResourceItem, ResourceType};

#[test]
fn test_lander_building_exists() {
    // Act
    let lander = BuildingType::Lander;

    // Assert
    assert_eq!(lander.label(), "Lander");
    // Should provide utility
    // We can't check components on Enum directly, but we can check spawn logic in Green phase test
}

#[test]
fn test_cannibalize_designation_exists() {
    let des = DesignationType::Cannibalize;
    assert_eq!(des.label(), "Cannibalize");
}

#[test]
fn test_execute_cannibalize_removes_lander_and_spawns_resources() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(scale::layer1::map::ScreenShake::default()); // Dependency
    world.insert_resource(scale::layer1::building::OccupiedTiles::default()); // Dependency
    world.insert_resource(scale::shared::log::MessageLog::default());

    let lander_pos = GridPosition { x: 10, y: 10 };
    let lander_entity = world
        .spawn((
            Building {
                building_type: BuildingType::Lander,
            },
            lander_pos,
        ))
        .id();

    // Mark tile as occupied so logic can find it if it uses OccupiedTiles (execution usually queries GridPosition)
    world
        .resource_mut::<scale::layer1::building::OccupiedTiles>()
        .0
        .insert((10, 10));

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Cannibalize,
            },
            lander_pos,
        ))
        .id();

    // Act
    // We assume a system or function `execute_cannibalize` handles this.
    // For TDD, we can expose it via `scale::layer1::execution::execute_cannibalize`
    let success = scale::layer1::execution::execute_cannibalize(&mut world, designation);

    // Assert
    assert!(success, "Cannibalization should succeed");
    assert!(
        world.get_entity(lander_entity).is_err(),
        "Lander should be despawned"
    );
    assert!(
        world.get_entity(designation).is_err(),
        "Designation should be despawned"
    );

    // Check Resources
    let items: Vec<&ResourceItem> = world.query::<&ResourceItem>().iter(&world).collect();

    // Expect Metal, Fuel, Rations
    let metal = items
        .iter()
        .find(|i| i.resource_type == ResourceType::Metal)
        .expect("Should yield Metal");
    let fuel = items
        .iter()
        .find(|i| i.resource_type == ResourceType::Fuel)
        .expect("Should yield Fuel");
    let rations = items
        .iter()
        .find(|i| i.resource_type == ResourceType::Rations)
        .expect("Should yield Rations");

    assert!(metal.amount >= 50.0);
    assert!(fuel.amount >= 20.0);
    assert!(rations.amount >= 20.0);
}

#[test]
fn test_cannibalize_only_works_on_lander() {
    let mut world = World::new();
    let pos = GridPosition { x: 5, y: 5 };
    world.insert_resource(scale::layer1::building::OccupiedTiles::default());

    // Spawn a House
    world.spawn((
        Building {
            building_type: BuildingType::Housing,
        },
        pos,
    ));
    world
        .resource_mut::<scale::layer1::building::OccupiedTiles>()
        .0
        .insert((5, 5));

    let designation = world
        .spawn((
            Designation {
                designation_type: DesignationType::Cannibalize,
            },
            pos,
        ))
        .id();

    // Act
    let success = scale::layer1::execution::execute_cannibalize(&mut world, designation);

    // Assert
    assert!(!success, "Should not cannibalize non-Lander");
}
