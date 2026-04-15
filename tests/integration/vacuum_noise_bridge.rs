use bevy::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::physics::acoustic::{update_noise_system, NoiseMap, NoiseSource};
use scale::layer1::physics::pressure::{update_pressure_system, PressureGrid};
use scale::layer1::terrain::{TerrainGrid, TerrainType};

#[test]
fn test_vacuum_blocks_noise() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let width = 10;
    let height = 10;
    let size = width * height;

    app.insert_resource(TerrainGrid {
        width,
        height,
        tiles: vec![TerrainType::Grass; size],
    });
    app.insert_resource(NoiseMap::new(width, height));

    let mut pressure_grid = PressureGrid::new(width, height);
    pressure_grid.fill(1.0); // Air everywhere initially
    // Manually create a vacuum gap at x=5
    for y in 0..height {
        pressure_grid.set(5, y as i32, 0.0);
    }
    app.insert_resource(pressure_grid);

    // Spawn a loud source at (2, 5)
    app.world_mut().spawn((
        GridPosition { x: 2, y: 5 },
        NoiseSource {
            radius: 10.0,
            intensity: 1.0,
        },
    ));

    // Chain systems to ensure update_pressure_system runs, then noise updates based on NEW pressure
    app.add_systems(Update, (update_pressure_system, update_noise_system).chain());

    app.update(); // Calculate noise based on pressure

    let pressure = app.world().resource::<PressureGrid>();
    assert!(pressure.get(5, 5) < 0.1, "Pressure should drop to create vacuum");

    let noise = app.world().resource::<NoiseMap>();

    assert!(noise.get(2, 5) > 0.5, "Source should be loud");
    assert!(noise.get(8, 5) < 0.2, "Vacuum gap at x=5 should block sound from reaching 8,5");
}
