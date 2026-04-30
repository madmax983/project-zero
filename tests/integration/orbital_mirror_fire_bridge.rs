use bevy::prelude::*;
use scale::layer1::temperature::TemperatureGrid;
use scale::layer1::nature::fire::{Fire, Flammable};
use scale::layer1::GridPosition;
use scale::layer2::orbital_mirrors::{OrbitalMirror, orbital_mirror_focus_system};
use scale::layer1::nature::fire::fire_ignition_system;

#[test]
fn test_orbital_mirror_starts_fire() {
    let mut app = App::new();

    app.add_systems(Update, (
        orbital_mirror_focus_system,
        fire_ignition_system.after(orbital_mirror_focus_system),
    ));

    let temp_grid = TemperatureGrid::new(10, 10, 20.0);
    let light_grid = scale::layer1::lighting::LightMap::new(10, 10);
    app.world_mut().insert_resource(temp_grid);
    app.world_mut().insert_resource(light_grid);

    // We also need a TerrainGrid
    let mut terrain_tiles = vec![scale::layer1::terrain::TerrainType::Grass; 100];
    terrain_tiles[55] = scale::layer1::terrain::TerrainType::Tree; // at 5, 5
    app.world_mut().insert_resource(scale::layer1::terrain::TerrainGrid {
        width: 10,
        height: 10,
        tiles: terrain_tiles,
    });

    let mut time: Time<()> = Time::default();
    time.advance_by(std::time::Duration::from_secs(1));

    app.world_mut().insert_resource(time);

    // Spawn highly intense mirror
    app.world_mut().spawn(OrbitalMirror {
        target: Vec2::new(5.0, 5.0),
        intensity: 500.0, // Flash ignition temperature
        radius: 1.0,
        alignment_error: 0.0,
    });

    // Optionally spawn a Flammable building at 6, 6
    app.world_mut().spawn((
        Flammable::default(),
        GridPosition { x: 6, y: 6 },
    ));

    app.update();

    let updated_temp = app.world().resource::<TemperatureGrid>();
    assert!(updated_temp.get(5, 5) >= 500.0);

    // Check if Fire component exists at 5,5 (tree) or 6,6 (flammable)
    let mut fire_exists = false;
    for (pos, _fire) in app.world_mut().query::<(&GridPosition, &Fire)>().iter(app.world()) {
        if (pos.x == 5 && pos.y == 5) || (pos.x == 6 && pos.y == 6) {
            fire_exists = true;
            break;
        }
    }
    assert!(fire_exists, "Orbital mirror should have started a fire");
}
