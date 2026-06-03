use bevy_ecs::prelude::*;
use bevy_utils::{HashMap, HashSet};

use crate::layer1::architecture::building::Building;
use crate::layer1::core::map::GridPosition;

#[derive(Resource, Default)]
pub struct OccupiedTiles(pub HashSet<(i32, i32)>);

#[derive(Resource, Default)]
pub struct BuildingMap(pub HashMap<(i32, i32), Entity>);

pub fn update_building_map_system(
    mut map: ResMut<BuildingMap>,
    query: Query<(Entity, &GridPosition), With<Building>>,
) {
    map.0.clear();
    for (entity, pos) in query.iter() {
        map.0.insert((pos.x, pos.y), entity);
    }
}
