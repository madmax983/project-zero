use bevy::prelude::*;
use scale::layer1::energy::load_limits::{evaluate_grid_load_system, PowerCable};
use scale::layer1::energy::PowerConsumer;
use scale::layer1::fire::Fire;
use scale::layer1::map::GridPosition;
use scale::layer1::temperature::TemperatureGrid;

#[test]
fn test_grid_instability_heat_and_fire_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add essential resources
    app.insert_resource(TemperatureGrid::new(10, 10, 0.0));

    // Add the system
    app.add_systems(bevy_app::Update, evaluate_grid_load_system);

    // 1. Setup a cable with a normal capacity
    let pos = GridPosition { x: 5, y: 5 };
    app.world_mut().spawn((
        PowerCable {
            capacity: 50.0,
            current_load: 0.0,
        },
        pos,
    ));

    // 2. Setup a massive consumer on the same tile
    app.world_mut().spawn((
        PowerConsumer {
            demand: 200.0, // > 2x capacity
            active: true,
        },
        pos,
    ));

    // 3. Run the system
    app.update();

    // 4. Assert Heat was generated
    let temp_grid = app.world().resource::<TemperatureGrid>();
    let tile_heat = temp_grid.get(pos.x as usize, pos.y as usize);
    assert!(
        tile_heat > 0.0,
        "Grid instability should generate heat in the TemperatureGrid. Got: {}",
        tile_heat
    );

    // 5. Assert Fire was spawned
    let fire_count = app.world_mut().query::<&Fire>().iter(app.world()).count();
    assert_eq!(
        fire_count, 1,
        "A severe grid instability should spawn exactly one Fire entity"
    );
}
