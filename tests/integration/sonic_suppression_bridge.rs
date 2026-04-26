use bevy::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::physics::acoustic::{update_noise_system, NoiseMap};
use scale::layer1::sonic_suppression::{sonic_suppression_system, SonicTurret};
use scale::layer1::integration::sonic_turret_noise_bridge_system;

#[test]
fn test_sonic_turret_emits_noise() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let width = 10;
    let height = 10;
    app.insert_resource(NoiseMap::new(width, height));
    app.insert_resource(scale::layer1::terrain::TerrainGrid {
        width,
        height,
        tiles: vec![scale::layer1::terrain::TerrainType::Grass; width * height],
    });

    // Spawn a sonic turret
    app.world_mut().spawn((
        SonicTurret {
            range: 5.0,
            active: true,
        },
        GridPosition { x: 5, y: 5 },
    ));

    app.add_systems(Update, (sonic_suppression_system, sonic_turret_noise_bridge_system, update_noise_system).chain());
    app.update();

    let noise = app.world().resource::<NoiseMap>();
    assert!(noise.get(5, 5) > 0.0, "Sonic Turret should act as a noise source");
}
