use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use scale::layer1::atmosphere::AtmosphereGrid;
use scale::layer1::integration::vacuum_clears_pollution_system;
use scale::layer1::pressure::PressureGrid;

#[test]
fn test_vacuum_clears_pollution() {
    use scale::layer1::nature::atmospheric_empathy::TraceGasGrid;
    let mut world = World::new();

    let width = 10;
    let height = 10;
    world.insert_resource(TraceGasGrid::new(width, height));

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

#[test]
fn test_vacuum_clears_trace_gases() {
    use scale::layer1::nature::atmospheric_empathy::{TraceGasGrid, GasType};

    let mut world = World::new();

    // Setup grids
    let width = 10;
    let height = 10;

    // 1. Atmosphere
    let mut atmosphere = AtmosphereGrid::new(width, height);
    atmosphere.set(5, 5, 1.0);
    world.insert_resource(atmosphere);

    // 2. TraceGasGrid with trace gas at (5,5)
    let mut trace_gas = TraceGasGrid::new(width, height);
    trace_gas.add_gas(5, 5, GasType::Euphoric, 1.0);
    trace_gas.add_gas(5, 5, GasType::Fear, 0.5);
    trace_gas.add_gas(5, 5, GasType::Rage, 0.3);
    world.insert_resource(trace_gas);

    // 3. Pressure grid with vacuum (0.0) at (5,5)
    let pressure = PressureGrid::new(width, height);
    world.insert_resource(pressure);

    // 4. Run the system
    world
        .run_system_once(vacuum_clears_pollution_system)
        .unwrap();

    // 5. Assert trace gas is cleared
    let trace_gas = world.resource::<TraceGasGrid>();
    let euphoric = trace_gas.get_gas(5, 5, GasType::Euphoric);
    let fear = trace_gas.get_gas(5, 5, GasType::Fear);
    let rage = trace_gas.get_gas(5, 5, GasType::Rage);

    assert!(
        euphoric < 0.01,
        "Euphoric gas should be cleared in vacuum! Found: {}",
        euphoric
    );
    assert!(
        fear < 0.01,
        "Fear gas should be cleared in vacuum! Found: {}",
        fear
    );
    assert!(
        rage < 0.01,
        "Rage gas should be cleared in vacuum! Found: {}",
        rage
    );
}
