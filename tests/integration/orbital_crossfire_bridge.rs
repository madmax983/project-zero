use bevy::prelude::*;

use scale::layer1::orbital_crossfire::OrbitalEvent;
use scale::layer1::terrain::{TerrainGrid, TerrainType};
use scale::layer2::events::ShipDestroyedEvent;
use scale::layer1::integration::orbital_crossfire_bridge;

#[test]
fn test_ship_destroyed_spawns_orbital_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.init_resource::<Events<ShipDestroyedEvent>>();
    let terrain = TerrainGrid { width: 20, height: 20, tiles: vec![TerrainType::Grass; 400] };
    app.insert_resource(terrain);

    app.add_systems(Update, orbital_crossfire_bridge);

    // Send the event
    let planet_entity = app.world_mut().spawn_empty().id();
    app.world_mut().send_event(ShipDestroyedEvent {
        planet: planet_entity,
        ship_class: "Frigate".to_string(),
    });

    app.update();

    let mut query = app.world_mut().query::<&OrbitalEvent>();
    assert_eq!(query.iter(app.world()).count(), 1, "Should spawn exactly one OrbitalEvent");
    let event = query.iter(app.world()).next().unwrap();
    assert!(event.target.x >= 0 && event.target.x < 20);
    assert!(event.target.y >= 0 && event.target.y < 20);
    assert!(event.damage > 0.0);
    assert!(event.heat > 0.0);
}
