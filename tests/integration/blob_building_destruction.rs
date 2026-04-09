use bevy::prelude::*;
use scale::layer1::blob::{BlobNetwork, BlobNode, blob_expansion_system};
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::events::BuildingRemovedEvent;
use scale::layer1::integration::blob_building_destruction_system;
use scale::layer1::map::GridPosition;
use scale::layer1::nature::terrain::TerrainGrid;

#[test]
fn test_blob_destroys_building_on_expansion() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<BuildingRemovedEvent>();
    app.insert_resource(TerrainGrid::new(10, 10));

    // Add integration seam and original expansion system
    app.add_systems(Update, (
        blob_expansion_system,
        blob_building_destruction_system,
    ).chain());

    // Spawn a target building at (6, 5)
    let building_entity = app.world_mut().spawn((
        Building {
            building_type: BuildingType::Housing,
        },
        GridPosition { x: 6, y: 5 },
    )).id();

    // Spawn the blob network and a node at (5, 5)
    let network = app.world_mut().spawn(BlobNetwork { expansion_timer: 1.0, current_time: 1.0 }).id();
    app.world_mut().spawn((
        BlobNode { network_id: network },
        GridPosition { x: 5, y: 5 },
    ));

    app.update(); // Trigger expansion and our destruction seam

    // Assert: The building was removed
    let events = app.world().resource::<Events<BuildingRemovedEvent>>();
    let mut reader = events.get_reader();
    let events_emitted: Vec<_> = reader.read(events).collect();

    assert_eq!(events_emitted.len(), 1, "Expected one BuildingRemovedEvent to be emitted.");
    assert_eq!(events_emitted[0].entity, building_entity);
    assert_eq!(events_emitted[0].building_type, BuildingType::Housing);
    assert_eq!(events_emitted[0].position, GridPosition { x: 6, y: 5 });

    // Assert: the building entity has been despawned
    assert!(app.world().get_entity(building_entity).is_err(), "Building entity should be despawned.");
}
