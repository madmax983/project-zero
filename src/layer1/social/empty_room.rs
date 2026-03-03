use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::stress::StressTracker;
use crate::layer1::building::Building;
use crate::layer1::inventory::Inventory;
use crate::layer1::clutter::ClutterGrid;
use std::collections::HashSet;

/// Tracks valid empty room sanctuaries on the map.
#[derive(Resource, Default)]
pub struct SanctuaryTracker {
    /// Maps a tile coordinate (x, y) to the effectiveness (size) of the valid sanctuary it belongs to.
    /// If a tile is not in this map, it provides no sanctuary bonus.
    pub valid_tiles: std::collections::HashMap<(i32, i32), f32>,
}

impl SanctuaryTracker {
    pub fn get_effectiveness(&self, pos: GridPosition) -> f32 {
        *self.valid_tiles.get(&(pos.x, pos.y)).unwrap_or(&0.0)
    }
}

/// Updates the `SanctuaryTracker` by checking for empty `ZoneType::Sanctuary` designations.
pub fn update_sanctuary_system(
    zone_grid_opt: Option<Res<ZoneGrid>>,
    mut tracker: ResMut<SanctuaryTracker>,
    buildings: Query<&GridPosition, With<Building>>,
    items: Query<&GridPosition, With<Inventory>>,
    clutter_grid_opt: Option<Res<ClutterGrid>>,
) {
    tracker.valid_tiles.clear();

    let Some(zone_grid) = zone_grid_opt else { return; };

    let mut occupied_tiles = HashSet::new();

    // Add all building positions
    for pos in buildings.iter() {
        occupied_tiles.insert((pos.x, pos.y));
    }

    // Add all item positions (Inventories represent stockpiles or dropped items)
    for pos in items.iter() {
        occupied_tiles.insert((pos.x, pos.y));
    }

    // Add all clutter positions
    if let Some(clutter_grid) = clutter_grid_opt {
        for y in 0..clutter_grid.height {
            for x in 0..clutter_grid.width {
                let ux = x;
                let uy = y;
                if clutter_grid.get(ux, uy) > 0.0 {
                    occupied_tiles.insert((x as i32, y as i32));
                }
            }
        }
    }

    // Now find contiguous groups of Sanctuary tiles
    let mut visited = HashSet::new();

    for y in 0..zone_grid.height {
        for x in 0..zone_grid.width {
            let x = x as i32;
            let y = y as i32;
            if zone_grid.get(x, y) == ZoneType::Sanctuary && !visited.contains(&(x, y)) {
                // BFS to find the whole room
                let mut room_tiles = Vec::new();
                let mut queue = vec![(x, y)];
                visited.insert((x, y));
                let mut is_valid = true;

                while let Some((cx, cy)) = queue.pop() {
                    room_tiles.push((cx, cy));

                    if occupied_tiles.contains(&(cx, cy)) {
                        is_valid = false; // The whole room is invalidated if any tile is occupied
                    }

                    // Check neighbors
                    let neighbors = [
                        (cx + 1, cy),
                        (cx - 1, cy),
                        (cx, cy + 1),
                        (cx, cy - 1),
                    ];

                    for &(nx, ny) in &neighbors {
                        if nx >= 0 && ny >= 0 && nx < zone_grid.width as i32 && ny < zone_grid.height as i32 {
                            if zone_grid.get(nx, ny) == ZoneType::Sanctuary && !visited.contains(&(nx, ny)) {
                                visited.insert((nx, ny));
                                queue.push((nx, ny));
                            }
                        }
                    }
                }

                if is_valid {
                    let effectiveness = room_tiles.len() as f32;
                    for tile in room_tiles {
                        tracker.valid_tiles.insert((tile.0, tile.1), effectiveness);
                    }
                }
            }
        }
    }
}

/// Allows pops in an empty room sanctuary to reduce their stress.
pub fn visit_sanctuary_system(
    mut pops: Query<(&GridPosition, &mut StressTracker)>,
    tracker_opt: Option<Res<SanctuaryTracker>>,
) {
    if let Some(tracker) = tracker_opt {
        for (pos, mut stress) in pops.iter_mut() {
            let effectiveness = tracker.get_effectiveness(*pos);
            if effectiveness > 0.0 {
                stress.accumulated_stress = (stress.accumulated_stress - (effectiveness * 0.1)).max(0.0);
            }
        }
    }
}
