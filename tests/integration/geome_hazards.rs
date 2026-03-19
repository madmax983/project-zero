use bevy_ecs::prelude::*;
use scale::layer1::{TerrainGrid, TerrainType, GridPosition};
use scale::layer1::geomes::{diffuse_geome_hazards_system, environmental_damage_system};
use scale::layer1::health::Health;
use scale::layer1::pop::Pop;

#[test]
fn test_geome_hazard_damages_health() {
    let mut world = World::new();

    let mut grid = TerrainGrid {
        width: 10,
        height: 10,
        tiles: vec![TerrainType::Rock; 100],
    };
    // Setup a SporeBloom tile
    grid.set(6, 5, TerrainType::SporeBloom);
    world.insert_resource(grid);

    let miner = world
        .spawn((
            Pop,
            Health {
                current: 100.0,
                max: 100.0,
            },
            GridPosition { x: 6, y: 5 }, // Standing on the SporeBloom tile
        ))
        .id();

    // Run hazard diffusion system to spawn the hazard entity
    let mut hazard_schedule = Schedule::default();
    hazard_schedule.add_systems(diffuse_geome_hazards_system);
    hazard_schedule.run(&mut world);

    // Run environmental damage system to apply damage
    let mut damage_schedule = Schedule::default();
    damage_schedule.add_systems(environmental_damage_system);
    damage_schedule.run(&mut world);

    // The miner should have taken damage from the Spore hazard
    let health = world.get::<Health>(miner).unwrap();
    assert!(
        health.current < 100.0,
        "Miner should take damage from breached geome hazard"
    );
}
