#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
//! Window Views System (Spec 216).
//!
//! This module implements the "View" mechanic, where `Window`s capture beauty from the environment
//! and project it into the room as a [`BeautySource`].
//!
//! # Mechanics
//!
//! 1.  **Placement**: Windows are placed on walls. They have a `Direction` facing outwards.
//! 2.  **Raycasting**: The `update_window_views_system` casts a ray from the window in its facing direction.
//! 3.  **Beauty Capture**: The system sums the beauty values of tiles along the ray (from the [`BeautyGrid`]).
//! 4.  **Obstruction**: Buildings (like Walls) block the view, terminating the ray.
//! 5.  **Projection**: The total captured beauty is scaled (currently 10%) and set as the value of the
//!     window's local [`BeautySource`].
//!
//! # Example
//!
//! A window facing a garden (High Beauty) will become a source of beauty inside the room.
//! A window facing a landfill (Negative Beauty) will become a source of ugliness.

use crate::layer1::beauty::{BeautyGrid, BeautySource};
use crate::layer1::building::{Building, BuildingMap, Direction, OccupiedTiles};
use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

/// Component representing a window that captures view beauty.
///
/// Windows are typically attached to wall entities. They must also have a [`BeautySource`] component
/// to emit the captured beauty into the room.
#[derive(Component, Default)]
pub struct Window {
    /// The direction the window faces (e.g., North, South).
    /// The ray is cast in this direction.
    pub direction: Direction,
    /// The maximum distance (in tiles) the window can "see".
    pub range: u32,
    /// The angle of view (unused in MVP, reserved for future cone-based views).
    pub view_cone: f32,
}

/// System to update window beauty sources based on their view.
///
/// This system performs the following for each window:
/// 1.  Iterates from `1` to `range` in the window's `direction`.
/// 2.  Checks for map bounds (adds a small bonus for "Sky View" if hitting edge).
/// 3.  Checks for obstructions (buildings that block wind/view).
/// 4.  Sums the beauty value of valid tiles from the [`BeautyGrid`].
/// 5.  Updates the window's [`BeautySource`] value with a scaled total (10%).
///
/// # Performance
/// This system runs every few ticks (configured in `SystemSet`).
/// It performs a raycast for every window, so it scales linearly with window count * range.
pub fn update_window_views_system(
    grid: Res<BeautyGrid>,
    occupied: Res<OccupiedTiles>,
    building_map: Res<BuildingMap>,
    buildings: Query<&Building>,
    mut windows: Query<(&GridPosition, &Window, &mut BeautySource)>,
) {
    for (pos, window, mut source) in &mut windows {
        let (dx, dy) = window.direction.to_delta();
        let mut total_view_beauty = 0.0;

        for i in 1..=window.range {
            let tx = pos.x + dx * (i as i32);
            let ty = pos.y + dy * (i as i32);

            // Bounds check
            if tx < 0 || ty < 0 || tx >= grid.width as i32 || ty >= grid.height as i32 {
                // Hit map edge. Add "Sky" bonus?
                total_view_beauty += 5.0; // Sky view bonus
                break;
            }

            // Obstruction check
            if occupied.0.contains(&(tx, ty)) {
                // Check if it's a building that blocks view
                if let Some(entity) = building_map.0.get(&(tx, ty)) {
                    if let Ok(building) = buildings.get(*entity) {
                        // Use blocks_wind as proxy for blocks_view
                        // Windows also block wind, but shouldn't block view?
                        // But Window component is usually separate.
                        // If we look through another window, we might see through.
                        // For MVP, assume blocks_wind blocks view.
                        // Exception: If the building is a Window?
                        // BuildingType::Window returns true for blocks_wind.
                        // So a window looking at a window sees... the window.
                        if building.building_type.blocks_wind() {
                            // Blocked
                            break;
                        }
                    } else {
                        // Entity in map but not in query? Assume blocked.
                        break;
                    }
                }
            }

            // Read beauty from grid
            // Note: grid contains values from update_beauty_grid_system which runs before this.
            // But wait, if we modify grid in this loop, does it affect subsequent rays?
            // Yes. But beauty from windows is sparse.
            let tile_beauty = grid.get(tx as usize, ty as usize);

            // Distance attenuation?
            // Spec says: "Attenuate by distance? ... total_view_beauty += tile_beauty;"
            // Let's just sum it for now.
            total_view_beauty += tile_beauty;
        }

        // Apply scale factor
        let final_value = total_view_beauty * 0.1;
        source.value = final_value;
    }
}

#[cfg(test)]
mod tests {
    use super::{update_window_views_system, Window};
    use crate::layer1::beauty::{BeautyGrid, BeautySource};
    use crate::layer1::building::{Building, BuildingMap, BuildingType, Direction, OccupiedTiles};
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

    // Helper to setup world
    fn setup_world() -> World {
        let mut world = World::new();
        let width = 20;
        let height = 20;
        world.insert_resource(BeautyGrid::new(width, height));
        world.insert_resource(TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; width.checked_mul(height).expect("overflow")],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(BuildingMap::default());
        world
    }

    // Helper to update building map since we don't run the system
    fn update_building_map(world: &mut World) {
        let mut map = world.resource_mut::<BuildingMap>();
        map.0.clear();
        let mut query = world.query::<(Entity, &GridPosition, &Building)>();
        let mut updates = Vec::new();
        for (e, pos, _) in query.iter(world) {
            updates.push((*pos, e));
        }
        // Re-borrow map mutably
        let mut map = world.resource_mut::<BuildingMap>();
        for (pos, e) in updates {
            map.0.insert((pos.x, pos.y), e);
        }
    }

    #[test]
    fn test_window_captures_distant_beauty() {
        let mut world = setup_world();

        // 1. Place a Statue at (10, 5) with high beauty
        world.spawn((
            Building {
                building_type: BuildingType::Statue,
            },
            GridPosition { x: 10, y: 5 },
            BeautySource {
                value: 10.0,
                radius: 2.0,
            },
        ));

        // 2. Place a Window at (5, 5) facing East (towards Statue)
        world.spawn((
            Building {
                building_type: BuildingType::Window,
            },
            GridPosition { x: 5, y: 5 },
            Window {
                direction: Direction::East,
                range: 10,
                view_cone: 0.0,
            },
            // Window itself has base beauty 0.0, but will gain "View Beauty"
            BeautySource {
                value: 0.0,
                radius: 2.0,
            },
        ));

        // Update helper map
        update_building_map(&mut world);

        // 3. Manually populate grid with Statue beauty (since we skip update_beauty_grid_system)
        let mut grid = world.resource_mut::<BeautyGrid>();
        grid.set(10, 5, 10.0);

        // Run the system once
        let mut schedule = Schedule::default();
        schedule.add_systems(update_window_views_system);
        schedule.run(&mut world);

        // 4. Check Window BeautySource
        // 10.0 beauty * 0.1 scale = 1.0
        let mut query = world.query::<(&Window, &BeautySource)>();
        let (_, source) = query.single(&world);
        assert!(
            source.value > 0.0,
            "Window should have positive beauty value from view"
        );
        assert!((source.value - 1.0).abs() < 0.1, "Expected ~1.0 beauty");
    }

    #[test]
    fn test_window_obstruction() {
        let mut world = setup_world();

        // Statue at (10, 5)
        let mut grid = world.resource_mut::<BeautyGrid>();
        grid.set(10, 5, 10.0);

        // Wall at (8, 5) - Blocking the view
        world.spawn((
            Building {
                building_type: BuildingType::Wall,
            },
            GridPosition { x: 8, y: 5 },
        ));
        let mut occupied = world.resource_mut::<OccupiedTiles>();
        occupied.0.insert((8, 5));

        // Window at (5, 5) facing East
        world.spawn((
            Building {
                building_type: BuildingType::Window,
            },
            GridPosition { x: 5, y: 5 },
            Window {
                direction: Direction::East,
                range: 10,
                view_cone: 0.0,
            },
            BeautySource {
                value: 0.0,
                radius: 2.0,
            },
        ));

        update_building_map(&mut world);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_window_views_system);
        schedule.run(&mut world);

        let mut query = world.query::<(&Window, &BeautySource)>();
        let (_, source) = query.single(&world);
        // View blocked by wall, beauty should be 0.0
        assert_eq!(source.value, 0.0);
    }

    #[test]
    fn test_window_negative_view() {
        let mut world = setup_world();

        // Landfill at (10, 5) - Ugly
        let mut grid = world.resource_mut::<BeautyGrid>();
        grid.set(10, 5, -10.0);

        // Window at (5, 5) facing East
        world.spawn((
            Building {
                building_type: BuildingType::Window,
            },
            GridPosition { x: 5, y: 5 },
            Window {
                direction: Direction::East,
                range: 10,
                view_cone: 0.0,
            },
            BeautySource {
                value: 0.0,
                radius: 2.0,
            },
        ));

        update_building_map(&mut world);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_window_views_system);
        schedule.run(&mut world);

        let mut query = world.query::<(&Window, &BeautySource)>();
        let (_, source) = query.single(&world);
        // View should be negative
        // -10.0 * 0.1 = -1.0
        assert!(source.value < 0.0);
        assert!((source.value - (-1.0)).abs() < 0.1);
    }
}
