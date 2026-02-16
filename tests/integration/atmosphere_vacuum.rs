use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use scale::layer1::atmosphere::AtmosphereGrid;
use scale::layer1::integration::vacuum_clears_pollution_system;
use scale::layer1::pressure::PressureGrid;

#[test]
fn test_vacuum_clears_pollution() {
    let mut world = World::new();

    // Setup grids
    let width = 10;
    let height = 10;

    // 1. Atmosphere with pollution at (5,5)
    let mut atmosphere = AtmosphereGrid::new(width, height);
    atmosphere.set(5, 5, 1.0);
    world.insert_resource(atmosphere);

    // 2. Pressure grid with vacuum (0.0) at (5,5)
    let pressure = PressureGrid::new(width, height);
    world.insert_resource(pressure);

    // 3. Run the system
    world
        .run_system_once(vacuum_clears_pollution_system)
        .unwrap();

    // 4. Assert pollution is cleared
    let atmosphere = world.resource::<AtmosphereGrid>();
    let pollution = atmosphere.get(5, 5);

    assert!(
        pollution < 0.01,
        "Pollution should be cleared in vacuum! Found: {}",
        pollution
    );
}
