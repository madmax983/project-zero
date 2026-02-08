use scale::layer1::{Fire, GridPosition, Health, Needs, Pop};
use scale::shared::state::GameState;
use scale::simulation::run_simulation_tick;

#[test]
fn test_fire_damages_pop_on_same_tile() {
    // 1. Setup World
    let mut world = scale::setup::setup_world();
    *world.resource_mut::<GameState>() = GameState::Running;

    // 2. Spawn Pop and Fire at (5, 5)
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Health {
                current: 100.0,
                max: 100.0,
            },
            Needs::default(),
        ))
        .id();

    world.spawn((
        Fire {
            lifetime: 10,
            intensity: 1.0,
        },
        GridPosition { x: 5, y: 5 },
    ));

    // 3. Run Simulation Tick
    run_simulation_tick(&mut world);

    // 4. Assert Health Decreased
    let health = world.get::<Health>(pop).expect("Pop should still exist");

    assert!(
        health.current < 100.0,
        "Pop standing in fire should take damage (Current: {})",
        health.current
    );
}
