use bevy::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::nature::atmosphere::{simulate_diffusion_system, AtmosphereGrid, DiffusionConfig, update_atmosphere_system};
use scale::layer1::weather::WeatherState;
use scale::layer1::physics::vent::{VentConnection, VentilationPlugin};

#[test]
fn test_atmosphere_diffuses_through_open_vent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(VentilationPlugin);

    app.insert_resource(DiffusionConfig {
        rate: 1.0,
        vertical_escape: 0.05,
    });
    app.insert_resource(WeatherState::default());

    let mut grid = AtmosphereGrid::new(5, 1);
    grid.set(0, 0, 1.0); // Source
    app.insert_resource(grid);

    // Smelter at (0, 0) to maintain source
    app.world_mut().spawn((
        scale::layer1::building::Building {
            building_type: scale::layer1::building::BuildingType::Smelter,
        },
        GridPosition { x: 0, y: 0 },
    ));

    // Place an open vent at 1,0 to allow airflow
    app.world_mut().spawn((
        VentConnection {
            pos_a: UVec2::new(0, 0),
            pos_b: UVec2::new(2, 0),
            grated: false,
        },
        GridPosition { x: 1, y: 0 }
    ));

    // Wall at 1,0 to ensure it's normally blocked
    app.world_mut().spawn((
        scale::layer1::building::Building {
            building_type: scale::layer1::building::BuildingType::Wall,
        },
        GridPosition { x: 1, y: 0 }
    ));

    app.add_systems(Update, (update_atmosphere_system, simulate_diffusion_system).chain());

    for _ in 0..20 {
        // Manually refill source
        app.world_mut().resource_mut::<AtmosphereGrid>().set(0, 0, 1.0);
        app.update();
    }

    let grid = app.world().resource::<AtmosphereGrid>();
    let pollution = grid.get(2, 0);
    assert!(pollution > 0.05, "Pollution should diffuse through open vent even with wall, but was {}", pollution);
}

#[test]
fn test_atmosphere_diffusion_reduced_by_grate() {
    // Scenario 1: Open vent
    let mut app1 = App::new();
    app1.add_plugins(MinimalPlugins);
    app1.add_plugins(VentilationPlugin);
    app1.insert_resource(DiffusionConfig {
        rate: 1.0,
        vertical_escape: 0.05,
    });
    app1.insert_resource(WeatherState::default());

    let mut grid1 = AtmosphereGrid::new(5, 1);
    grid1.set(0, 0, 1.0);
    app1.insert_resource(grid1);

    app1.world_mut().spawn((
        scale::layer1::building::Building {
            building_type: scale::layer1::building::BuildingType::Smelter,
        },
        GridPosition { x: 0, y: 0 },
    ));

    app1.world_mut().spawn((
        scale::layer1::building::Building {
            building_type: scale::layer1::building::BuildingType::Wall,
        },
        VentConnection {
            pos_a: UVec2::new(0, 0),
            pos_b: UVec2::new(2, 0),
            grated: false,
        },
        GridPosition { x: 1, y: 0 }
    ));

    app1.add_systems(Update, (update_atmosphere_system, simulate_diffusion_system).chain());
    for _ in 0..20 {
        app1.world_mut().resource_mut::<AtmosphereGrid>().set(0, 0, 1.0);
        app1.update();
    }
    let pollution1 = app1.world().resource::<AtmosphereGrid>().get(2, 0);

    // Scenario 2: Grated vent
    let mut app2 = App::new();
    app2.add_plugins(MinimalPlugins);
    app2.add_plugins(VentilationPlugin);
    app2.insert_resource(DiffusionConfig {
        rate: 1.0,
        vertical_escape: 0.05,
    });
    app2.insert_resource(WeatherState::default());

    let mut grid2 = AtmosphereGrid::new(5, 1);
    grid2.set(0, 0, 1.0);
    app2.insert_resource(grid2);

    app2.world_mut().spawn((
        scale::layer1::building::Building {
            building_type: scale::layer1::building::BuildingType::Smelter,
        },
        GridPosition { x: 0, y: 0 },
    ));

    app2.world_mut().spawn((
        scale::layer1::building::Building {
            building_type: scale::layer1::building::BuildingType::Wall,
        },
        VentConnection {
            pos_a: UVec2::new(0, 0),
            pos_b: UVec2::new(2, 0),
            grated: true,
        },
        GridPosition { x: 1, y: 0 }
    ));

    app2.add_systems(Update, (update_atmosphere_system, simulate_diffusion_system).chain());
    for _ in 0..20 {
        app2.world_mut().resource_mut::<AtmosphereGrid>().set(0, 0, 1.0);
        app2.update();
    }
    let pollution2 = app2.world().resource::<AtmosphereGrid>().get(2, 0);

    assert!(pollution2 < pollution1, "Grated vent should reduce pollution diffusion compared to open vent ({} vs {})", pollution2, pollution1);
    assert!(pollution2 > 0.0, "Grated vent should still allow some diffusion");
}
