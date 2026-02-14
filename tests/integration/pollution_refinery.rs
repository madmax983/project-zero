use scale::layer1::atmosphere::{AtmosphereGrid, update_atmosphere_system};
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::map::GridPosition;
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;

#[test]
fn test_refinery_emits_pollution() {
    let mut world = World::new();

    // Setup Atmosphere
    let grid = AtmosphereGrid::new(10, 10);
    world.insert_resource(grid);

    // Spawn Refinery
    world.spawn((
        Building {
            building_type: BuildingType::Refinery,
        },
        GridPosition { x: 5, y: 5 },
    ));

    // Run system
    world.run_system_once(update_atmosphere_system).unwrap();

    // Check pollution
    let grid = world.resource::<AtmosphereGrid>();
    let pollution = grid.get(5, 5);

    // Should be > 0.0
    // Currently fails because Refinery is not in the emission list
    assert!(pollution > 0.0, "Refinery should emit pollution, got {}", pollution);
}
