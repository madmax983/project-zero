//! The "Empty" Room (Spec 251)
//!
//! "In a crowded station, space is the ultimate luxury."
//!
//! Designating a "Sanctuary" zone requires it to be empty (no furniture, machines, or storage).
//! Pops visit these empty rooms to reduce Stress. The effectiveness scales with the size of the empty space.
//! Entropy fights back. Pops may leave "offerings" (flowers, rocks) or clutter in the Sanctuary,
//! breaking the "Empty" condition and disabling the bonus until cleaned.

use bevy_ecs::prelude::*;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::map::GridPosition;
use crate::layer1::building::Building;
use crate::layer1::items::Item;
use crate::layer1::clutter::{ClutterGrid};
use crate::layer1::stress::StressTracker;
use std::collections::HashSet;

/// Represents a contiguous Sanctuary zone.
#[derive(Component, Default)]
pub struct Sanctuary {
    pub is_valid: bool,
    pub effectiveness: f32,
    pub tiles: Vec<GridPosition>,
}

#[derive(Component)]
pub struct ZoneEntity {
    pub zone_type: ZoneType,
}

// Alternatively, because the existing architecture uses a `ZoneGrid` resource,
// instead of explicitly spawning `Zone` entities, we can maintain `Sanctuary` states
// either via a new Resource or by mapping connected components of `ZoneType::Sanctuary` on the `ZoneGrid`.

#[derive(Resource, Default)]
pub struct SanctuaryManager {
    // A map from a designated center or first tile to its state,
    // or just a vector of contiguous regions.
    pub sanctuaries: Vec<Sanctuary>,
}

pub fn update_sanctuary_system(
    mut manager: ResMut<SanctuaryManager>,
    zone_grid: Res<ZoneGrid>,
    buildings: Query<&GridPosition, With<Building>>,
    items: Query<&GridPosition, With<Item>>,
    clutter_grid: Option<Res<ClutterGrid>>,
) {
    // 1. Identify all contiguous regions of ZoneType::Sanctuary on the ZoneGrid
    // We'll use a simple flood-fill to find them.
    let mut visited = vec![false; zone_grid.width * zone_grid.height];
    let mut regions = Vec::new();

    for y in 0..zone_grid.height {
        for x in 0..zone_grid.width {
            let idx = y * zone_grid.width + x;
            if !visited[idx] && zone_grid.grid[idx] == ZoneType::Sanctuary {
                // Flood fill to find all tiles in this Sanctuary
                let mut tiles = Vec::new();
                let mut stack = vec![(x, y)];

                while let Some((cx, cy)) = stack.pop() {
                    let cidx = cy * zone_grid.width + cx;
                    if visited[cidx] || zone_grid.grid[cidx] != ZoneType::Sanctuary {
                        continue;
                    }
                    visited[cidx] = true;
                    tiles.push(GridPosition { x: cx as i32, y: cy as i32 });

                    if cx > 0 { stack.push((cx - 1, cy)); }
                    if cx < zone_grid.width - 1 { stack.push((cx + 1, cy)); }
                    if cy > 0 { stack.push((cx, cy - 1)); }
                    if cy < zone_grid.height - 1 { stack.push((cx, cy + 1)); }
                }
                regions.push(tiles);
            }
        }
    }

    // 2. Pre-calculate occupied tiles for O(1) lookups
    let mut occupied = HashSet::new();
    for pos in buildings.iter() {
        occupied.insert((pos.x, pos.y));
    }
    for pos in items.iter() {
        occupied.insert((pos.x, pos.y));
    }

    if let Some(clutter) = &clutter_grid {
        for y in 0..clutter.height {
            for x in 0..clutter.width {
                if clutter.get(x, y) > 0.0 {
                    occupied.insert((x as i32, y as i32));
                }
            }
        }
    }

    // 3. Update the Sanctuaries
    manager.sanctuaries.clear();
    for tiles in regions {
        let mut is_valid = true;
        for t in &tiles {
            if occupied.contains(&(t.x, t.y)) {
                is_valid = false;
                break;
            }
        }

        manager.sanctuaries.push(Sanctuary {
            is_valid,
            effectiveness: if is_valid { tiles.len() as f32 } else { 0.0 },
            tiles,
        });
    }
}

pub fn visit_sanctuary_system(
    mut pops: Query<(&GridPosition, &mut StressTracker)>,
    manager: Option<Res<SanctuaryManager>>,
) {
    let Some(manager) = manager else { return };

    for (pop_pos, mut mood) in pops.iter_mut() {
        // Find if pop is in a valid sanctuary
        for sanctuary in &manager.sanctuaries {
            if sanctuary.is_valid && sanctuary.tiles.contains(pop_pos) {
                // Spec says mood.stress = (mood.stress - (sanctuary.effectiveness * 0.1)).max(0.0);
                mood.accumulated_stress = (mood.accumulated_stress - (sanctuary.effectiveness * 0.1)).max(0.0);
                break; // A pop can only be in one sanctuary at a time
            }
        }
    }
}
