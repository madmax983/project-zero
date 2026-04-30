use bevy::prelude::*;
use scale::layer1::integration::sonic_turret_noise_bridge_system;
use scale::layer1::map::GridPosition;
use scale::layer1::physics::acoustic::{update_noise_system, NoiseMap};
use scale::layer1::sonic_suppression::{sonic_suppression_system, SonicTurret};

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

    app.add_systems(
        Update,
        (
            sonic_suppression_system,
            sonic_turret_noise_bridge_system,
            update_noise_system,
        )
            .chain(),
    );
    app.update();

    let noise = app.world().resource::<NoiseMap>();
    assert!(
        noise.get(5, 5) > 0.0,
        "Sonic Turret should act as a noise source"
    );
}

#[test]
fn test_sonic_turret_shatters_glass_buildings() {
    use scale::layer1::architecture::building::{BuildingType, try_place_building};
    use scale::layer1::architecture::structure::Structure;
    use scale::layer1::resources::ColonyResources;
    use scale::layer1::{OccupiedTiles, BuildingMap};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let width = 10;
    let height = 10;
    app.insert_resource(scale::layer1::terrain::TerrainGrid {
        width,
        height,
        tiles: vec![scale::layer1::terrain::TerrainType::Grass; width * height],
    });
    app.insert_resource(OccupiedTiles::default());
    app.insert_resource(BuildingMap::default());
    app.insert_resource(ColonyResources {
        wood: 1000.0,
        stone: 1000.0,
        metal: 1000.0,
        ..Default::default()
    });

    let greenhouse_success = try_place_building(app.world_mut(), 8, 5, BuildingType::Greenhouse);
    assert!(greenhouse_success, "Greenhouse placement should succeed");

    // Spawn a sonic turret
    app.world_mut().spawn((
        SonicTurret {
            range: 5.0,
            active: true,
        },
        GridPosition { x: 5, y: 5 },
    ));

    app.add_systems(
        Update,
        sonic_suppression_system,
    );
    app.update();

    let mut glass_destroyed = false;
    for (structure, _glass) in app.world_mut().query::<(&Structure, &scale::layer1::sonic_suppression::Glass)>().iter(app.world_mut()) {
        if structure.current_hp == 0.0 {
            glass_destroyed = true;
        }
    }

    assert!(glass_destroyed, "Glass building should be destroyed by sonic turret");
}
