use bevy_ecs::prelude::*;
use scale::layer1::logistics::glider::{update_glider_movement_system, ThermalGlider};
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Speed;
use scale::layer1::tech::event_horizon_tap::{apply_time_dilation_system, EventHorizonTap};
use scale::layer1::temperature::TemperatureGrid;

#[test]
fn test_thermal_glider_time_dilation_integration() {
    let mut world = World::new();
    let mut temp_grid = TemperatureGrid::new(10, 10, 0.0);

    // High temp so glider gets speed = 2.0 (Updraft)
    temp_grid.set(0, 0, 50.0);
    world.insert_resource(temp_grid);

    // Spawn Glider at hot tile
    let glider = world
        .spawn((
            ThermalGlider,
            Speed {
                base: 1.0,
                current: 0.0,
                accumulator: 0.0,
            },
            GridPosition { x: 0, y: 0 },
        ))
        .id();

    // Spawn Tap generating Time Dilation Zone
    world.spawn((
        EventHorizonTap {
            is_active: true,
            instability: 0.5,
        },
        GridPosition { x: 0, y: 0 },
    ));

    // Register systems exactly as they should be ordered
    let mut schedule = Schedule::default();
    schedule.add_systems((
        update_glider_movement_system,
        apply_time_dilation_system.after(update_glider_movement_system),
    ));

    schedule.run(&mut world);

    let speed = world.get::<Speed>(glider).unwrap();
    // Updraft -> base speed becomes 2.0. Time Dilation Zone (10% speed) -> 2.0 * 0.1 = 0.2.
    assert!(
        (speed.current - 0.2).abs() < f32::EPSILON,
        "Glider speed should be scaled down by time dilation zone. Expected 0.2, got {}",
        speed.current
    );
}
