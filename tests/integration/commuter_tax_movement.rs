use bevy_ecs::prelude::*;
use scale::layer1::economy::Wallet;
use scale::layer1::erosion::ErosionGrid;
use scale::layer1::execution::components::{JustMoved, MovementTarget};
use scale::layer1::execution::movement::{cleanup_just_moved_system, movement_system};
use scale::layer1::infrastructure::transit::{transit_toll_system, Toll, TransitInfrastructure};
use scale::layer1::map::GridPosition;
use scale::layer1::pop::{Pop, Speed};
use scale::layer1::stress::StressTracker;
use scale::layer1::utility_types::ActionType;

#[test]
fn test_commuter_tax_applied_on_movement() {
    let mut world = World::new();

    // Setup required resources for movement_system
    world.insert_resource(ErosionGrid::new(20, 20));
    let mut terrain = scale::layer1::terrain::generate_terrain(20, 20);
    // Make sure path is walkable
    terrain.set(4, 5, scale::layer1::terrain::TerrainType::Grass);
    terrain.set(5, 5, scale::layer1::terrain::TerrainType::Grass);
    terrain.set(6, 5, scale::layer1::terrain::TerrainType::Grass);
    world.insert_resource(terrain);

    let road_pos = GridPosition { x: 5, y: 5 };

    // Create toll road
    world.spawn((
        TransitInfrastructure { speed_multiplier: 1.0 },
        Toll { cost: 2.0 },
        road_pos,
    ));

    // Create a pop near the road, moving to it
    let pop = world.spawn((
        Pop,
        GridPosition { x: 4, y: 5 }, // Adjacent to road
        MovementTarget {
            target_entity: Entity::from_raw(999),
            target_position: GridPosition { x: 5, y: 5 }, // Move onto road
            for_action: ActionType::Idle,
        },
        Speed { base: 10.0, current: 10.0, accumulator: 10.0 }, // Enough speed to move
        Wallet { credits: 10.0 },
        StressTracker::default(),
    )).id();

    // Required for movement logic pathfinding fallback
    world.insert_resource(scale::layer1::building::OccupiedTiles::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(movement_system);
    schedule.add_systems(transit_toll_system.after(movement_system));
    schedule.add_systems(cleanup_just_moved_system.after(transit_toll_system));

    // Tick 1: Pop moves to (5, 5) -> toll applied
    schedule.run(&mut world);

    let pop_pos = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(*pop_pos, road_pos);

    let pop_wallet = world.get::<Wallet>(pop).unwrap();
    assert_eq!(pop_wallet.credits, 8.0, "Pop should have paid 2.0 toll when moving onto road");

    assert!(world.get::<JustMoved>(pop).is_none(), "JustMoved should be cleaned up");

    // Give pop speed again but don't move (not enough to leave tile yet)
    let mut speed = world.get_mut::<Speed>(pop).unwrap();
    speed.accumulator = 0.1; // Not enough to move

    // Tick 2: Pop stays on (5, 5) -> toll NOT applied again
    schedule.run(&mut world);

    let pop_pos2 = world.get::<GridPosition>(pop).unwrap();
    assert_eq!(*pop_pos2, road_pos);

    let pop_wallet2 = world.get::<Wallet>(pop).unwrap();
    assert_eq!(pop_wallet2.credits, 8.0, "Pop should NOT pay toll again if it didn't move");
}
