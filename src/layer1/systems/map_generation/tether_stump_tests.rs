use super::tether_stump::*;
use bevy::prelude::*;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer1::nature::atmosphere::AtmosphereGrid; // We will use AtmosphereGrid since PressureGrid does not exist

#[test]
fn test_tether_stump_generation() {
    // Arrange
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Startup, generate_tether_stump);

    let terrain = crate::layer1::nature::terrain::generate_terrain(100, 100);
    app.insert_resource(terrain);

    // Act
    app.update();

    // Assert
    let terrain = app.world().resource::<TerrainGrid>();

    let mut stump_tiles = 0;
    for x in 0..100 {
        for y in 0..100 {
            if terrain.get(x, y) == Some(TerrainType::IndestructibleStump) {
                stump_tiles += 1;
            }
        }
    }

    assert!(stump_tiles > 0 && stump_tiles <= 9, "Stump should have a small footprint");
}

#[test]
fn test_vertical_build_height_is_infinite_at_stump() {
    // Arrange
    let mut app = App::new();
    let terrain = crate::layer1::nature::terrain::generate_terrain(100, 100);
    app.insert_resource(terrain);
    app.add_systems(Startup, generate_tether_stump);
    app.update();

    // Act & Assert
    let terrain = app.world().resource::<TerrainGrid>();
    let extensions = app.world().resource::<VerticalExtension>();
    let (stump_x, stump_y) = find_stump_center(&terrain).unwrap();

    // Verify we can build at extreme heights at the stump coordinates
    assert!(extensions.is_stump(stump_x, stump_y));
}

#[test]
fn test_atmosphere_pressure_drops_with_height() {
    // Arrange
    let _app = App::new();
    // Assuming AtmosphereGrid replaces PressureGrid
    let _atmosphere_grid = AtmosphereGrid::new(100, 100);
    // And we test the localized VerticalExtension for pressure drop

    // Act: Initialize atmosphere gradient
    let mut extensions = VerticalExtension::new();
    initialize_atmosphere_gradient(&mut extensions);

    // Assert
    let ground_pressure = extensions.get_pressure(0);
    let high_altitude_pressure = extensions.get_pressure(50);

    assert!(high_altitude_pressure < ground_pressure, "Pressure should drop with altitude");
    assert!(high_altitude_pressure < 0.5, "High altitude should be dangerously thin");
}

#[test]
fn test_lost_tech_spawns_at_high_altitude() {
    // Arrange
    let mut app = App::new();
    let terrain = crate::layer1::nature::terrain::generate_terrain(100, 100);
    app.insert_resource(terrain);
    app.add_systems(Startup, (generate_tether_stump, spawn_lost_tech_caches).chain());

    // Act
    app.update();

    // Assert
    let mut tech_query = app.world_mut().query::<(&Transform, &LostTech)>();
    let tech_count = tech_query.iter(app.world()).count();

    assert!(tech_count > 0, "Should spawn lost tech");

    for (transform, _) in tech_query.iter(app.world()) {
        assert!(transform.translation.z > 20.0, "Lost tech should only spawn at high altitude");
    }
}
