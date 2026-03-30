use bevy_ecs::prelude::{Entity, Query, ResMut, With};
use crate::layer1::map::GridPosition;
use crate::layer1::building::building_components::{Building, BuildingMap};

/// System to update the spatial building map.
///
/// This rebuilds the map every frame to ensure pathfinding has fresh data.
/// It avoids iterating all entities during pathfinding calls.
pub fn update_building_map_system(
    mut map: ResMut<BuildingMap>,
    query: Query<(Entity, &GridPosition), With<Building>>,
) {
    map.0.clear();
    for (entity, pos) in query.iter() {
        map.0.insert((pos.x, pos.y), entity);
    }
}