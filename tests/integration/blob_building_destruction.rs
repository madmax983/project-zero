use bevy_app::{App, Update};
use bevy_ecs::prelude::*;
use scale::layer1::map::GridPosition;
use scale::layer1::nature::terrain::generate_terrain;
use scale::layer1::nature::terrain::TerrainType;
use scale::layer1::blob::{BlobNode, BlobNetwork, blob_expansion_system};
use scale::layer1::building::{Building, BuildingType, BuildingMap};
use scale::layer1::events::BuildingRemovedEvent;
use scale::layer1::integration::blob_building_destruction_system;
use bevy_app::Schedule;

#[test]
fn test_blob_destroys_building() {
    let mut app = App::new();

    app.init_resource::<Events<BuildingRemovedEvent>>();

    let mut grid = generate_terrain(10, 10);
    grid.set(5, 5, TerrainType::Dirt);
    grid.set(6, 5, TerrainType::Dirt);
    app.insert_resource(grid);

    // Provide BuildingMap
    let mut building_map = std::collections::HashMap::new();

    app.add_systems(Update, (
        blob_expansion_system,
        blob_building_destruction_system,
    ).chain());

    let network = app.world_mut().spawn(BlobNetwork { expansion_timer: 1.0, current_time: 1.0 }).id();

    app.world_mut().spawn((
        BlobNode { network_id: network },
        GridPosition { x: 5, y: 5 },
    ));

    // Spawn a building at (6, 5)
    let building = app.world_mut().spawn((
        Building { building_type: BuildingType::Farm },
        GridPosition { x: 6, y: 5 },
    )).id();
    building_map.insert((6, 5), building);
    app.insert_resource(BuildingMap(building_map));

    // Act
    app.update();

    // The blob expanded to (6, 5), where the building was. The building should be despawned and BuildingRemovedEvent should be sent.

    let events = app.world().resource::<Events<BuildingRemovedEvent>>();
    let event_reader: Vec<_> = events.get_reader().read(events).collect();

    assert_eq!(event_reader.len(), 1, "Blob should destroy the building and emit an event");
    assert_eq!(event_reader[0].entity, building);
    assert_eq!(event_reader[0].building_type, BuildingType::Farm);
    assert_eq!(event_reader[0].position, GridPosition { x: 6, y: 5 });

    let building_query = app.world_mut().query::<&Building>().iter(&app.world()).count();
    assert_eq!(building_query, 0, "Building should be despawned");
}
