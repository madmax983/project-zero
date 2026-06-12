use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::terrain::TerrainGrid;

#[derive(Resource)]
pub struct HullIntegrity {
    pub integrity: f32,
}

#[derive(Event)]
pub struct MineEvent {
    pub pos: GridPosition,
}

pub fn process_hull_integrity(
    mut events: EventReader<MineEvent>,
    mut integrity: ResMut<HullIntegrity>,
    grid: Res<TerrainGrid>,
) {
    for ev in events.read() {
        // Calculate distance to nearest edge
        let dist_x = ev.pos.x.min(grid.width as i32 - 1 - ev.pos.x);
        let dist_y = ev.pos.y.min(grid.height as i32 - 1 - ev.pos.y);
        let min_dist = dist_x.min(dist_y);

        // If mining near the edge, it damages the hull
        if min_dist <= 2 {
            integrity.integrity -= 10.0 / (min_dist as f32 + 1.0);
        }
    }
}
