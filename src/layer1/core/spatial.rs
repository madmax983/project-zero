use bevy_ecs::prelude::*;
use bevy_utils::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct OccupiedTiles(pub HashSet<(i32, i32)>);

#[derive(Resource, Default)]
pub struct BuildingMap(pub HashMap<(i32, i32), Entity>);
