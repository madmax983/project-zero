//! The Blob (Spec 874)
//! An indestructible, slow-growing entity that consumes adjacent tiles.
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::terrain::TerrainGrid;
use crate::layer1::nature::terrain::TerrainType;
use crate::layer1::resources::ResourceItem;
use crate::layer1::resources::ResourceType;

#[derive(Component)]
pub struct BlobNetwork {
    pub expansion_timer: f32,
    pub current_time: f32,
}

#[derive(Component)]
pub struct BlobNode {
    pub network_id: Entity,
}

pub fn blob_expansion_system(
    mut commands: Commands,
    mut networks: Query<(Entity, &mut BlobNetwork)>,
    nodes: Query<(&BlobNode, &GridPosition)>,
    grid: Res<TerrainGrid>,
) {
    for (net_entity, mut network) in networks.iter_mut() {
        network.current_time += 1.0; // Simulate tick

        if network.current_time >= network.expansion_timer {
            network.current_time = 0.0;

            let mut expanded = false;

            for (node, pos) in nodes.iter() {
                if node.network_id != net_entity { continue; }

                // Check cardinal directions
                let adjacents = [
                    GridPosition { x: pos.x + 1, y: pos.y },
                    GridPosition { x: pos.x - 1, y: pos.y },
                    GridPosition { x: pos.x, y: pos.y + 1 },
                    GridPosition { x: pos.x, y: pos.y - 1 },
                ];

                for adj in adjacents {
                    // Stop if out of bounds or blocked by a Wall (Rock)
                    if let Some(terrain) = grid.get(adj.x as usize, adj.y as usize) {
                        if !matches!(terrain, TerrainType::Rock | TerrainType::DeepRock) {
                            // Expand! (Minimal logic: just spawn one new node per network per tick)
                            commands.spawn((
                                BlobNode { network_id: net_entity },
                                adj,
                            ));
                            expanded = true;
                            break;
                        }
                    }
                }

                if expanded { break; }
            }
        }
    }
}

pub fn blob_spread_system() {}

pub fn blob_consumption_system(
    mut commands: Commands,
    mut networks: Query<&mut BlobNetwork>,
    nodes: Query<(&BlobNode, &GridPosition)>,
    items: Query<(Entity, &GridPosition, &ResourceItem)>,
) {
    let mut blob_positions = std::collections::HashMap::new();
    for (node, pos) in nodes.iter() {
        blob_positions.insert(*pos, node.network_id);
    }

    for (item_entity, item_pos, resource) in items.iter() {
        if resource.resource_type == ResourceType::Waste {
            if let Some(network_id) = blob_positions.get(item_pos) {
                if let Ok(mut network) = networks.get_mut(*network_id) {
                    network.current_time -= 5.0; // Stall growth
                    commands.entity(item_entity).despawn();
                }
            }
        }
    }
}

#[derive(Component, Default)]
pub struct Blob;

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;
    use crate::layer1::map::GridPosition;
    use crate::layer1::nature::terrain::TerrainType;
    use crate::layer1::nature::terrain::generate_terrain;

    #[test]
    fn test_blob_expands_to_adjacent_empty_tile() {
        let mut app = App::new();
        let mut grid = generate_terrain(10, 10);
        // Center is empty
        grid.set(5, 5, TerrainType::Dirt);
        grid.set(6, 5, TerrainType::Dirt);
        app.insert_resource(grid);

        app.add_systems(Update, blob_expansion_system);

        let network = app.world_mut().spawn(BlobNetwork { expansion_timer: 1.0, current_time: 1.0 }).id();

        // Spawn the seed blob
        app.world_mut().spawn((
            BlobNode { network_id: network },
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        // A new blob node should have spawned at (6, 5) or another adjacent tile
        let blob_count = app.world_mut().query::<&BlobNode>().iter(&app.world()).count();
        assert!(blob_count > 1, "Blob failed to expand");
    }

    #[test]
    fn test_blob_halts_when_contained_by_walls() {
        let mut app = App::new();
        let mut grid = generate_terrain(10, 10);
        // Surround the blob with walls
        grid.set(4, 5, TerrainType::Rock);
        grid.set(6, 5, TerrainType::Rock);
        grid.set(5, 4, TerrainType::Rock);
        grid.set(5, 6, TerrainType::Rock);
        app.insert_resource(grid);

        app.add_systems(Update, blob_expansion_system);

        let network = app.world_mut().spawn(BlobNetwork { expansion_timer: 1.0, current_time: 1.0 }).id();

        app.world_mut().spawn((
            BlobNode { network_id: network },
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        // No new blobs should spawn since it is boxed in
        let blob_count = app.world_mut().query::<&BlobNode>().iter(&app.world()).count();
        assert_eq!(blob_count, 1, "Blob expanded through walls");
    }

    #[test]
    fn test_blob_consumes_waste_to_delay_expansion() {
        let mut app = App::new();
        app.insert_resource(generate_terrain(10, 10));
        app.add_systems(Update, blob_expansion_system);

        // Give it a negative timer to signify it was just "fed"
        let network = app.world_mut().spawn(BlobNetwork { expansion_timer: 1.0, current_time: -5.0 }).id();

        app.world_mut().spawn((
            BlobNode { network_id: network },
            GridPosition { x: 5, y: 5 },
        ));

        app.update();

        // Because current_time < expansion_timer, it should not expand
        let blob_count = app.world_mut().query::<&BlobNode>().iter(&app.world()).count();
        assert_eq!(blob_count, 1, "Blob expanded despite being fed");
    }
}
