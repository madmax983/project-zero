//! Spatial primitives and map utilities.
//!
//! This module defines the foundational spatial component `GridPosition`.
//!
//! # The Coordinate System
//!
//! The world is represented as a 2D grid. The coordinate system follows these rules:
//! * **Origin (0,0)**: The top-left corner of the map.
//! * **X-Axis**: Increases to the right (East).
//! * **Y-Axis**: Increases downwards (South).
//!
//! While `GridPosition` uses `i32` to allow for flexibility (and potentially negative
//! coordinates for off-map entities or infinite scrolling in the future), the valid
//! gameplay area is typically bounded by the `TerrainGrid` dimensions (0..width, 0..height).
//!
//! # Relationship with `TerrainGrid`
//!
//! The `TerrainGrid` resource stores tile data in a flat vector. To access tile data
//! for a given `GridPosition`, you must convert the (x, y) coordinates to a linear index:
//! `index = y * width + x`.
//!
//! Always check bounds before accessing the grid, as `GridPosition` does not enforce
//! map limits itself.

use bevy_ecs::prelude::*;

/// Grid position in world space.
///
/// This component marks an entity's physical location in the colony. It is used by
/// rendering systems, pathfinding, and spatial queries.
///
/// # Examples
///
/// Basic usage:
/// ```
/// use scale::layer1::map::GridPosition;
///
/// let pos = GridPosition { x: 10, y: 5 };
/// assert_eq!(pos.x, 10);
/// ```
///
/// Calculating Manhattan distance:
/// ```
/// use scale::layer1::map::GridPosition;
///
/// let a = GridPosition { x: 0, y: 0 };
/// let b = GridPosition { x: 3, y: 4 };
/// let distance = (a.x - b.x).abs() + (a.y - b.y).abs();
/// assert_eq!(distance, 7);
/// ```
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPosition {
    /// The X coordinate (horizontal column).
    pub x: i32,
    /// The Y coordinate (vertical row).
    pub y: i32,
}

impl GridPosition {
    /// Returns the Chebyshev distance (chessboard distance) between two positions.
    /// This is the number of moves a King would take to get from A to B.
    #[must_use]
    pub fn distance_chebyshev(&self, other: GridPosition) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_position_creation() {
        let pos = GridPosition { x: 5, y: 10 };
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_grid_position_is_copy() {
        let pos1 = GridPosition { x: 3, y: 7 };
        let pos2 = pos1; // Should copy, not move
        assert_eq!(pos1.x, pos2.x);
        assert_eq!(pos1.y, pos2.y);
    }

    #[test]
    fn test_grid_position_negative_coords() {
        let pos = GridPosition { x: -5, y: -10 };
        assert_eq!(pos.x, -5);
        assert_eq!(pos.y, -10);
    }

    #[test]
    fn test_grid_position_debug() {
        let pos = GridPosition { x: 10, y: -5 };
        let debug_str = format!("{pos:?}");
        assert!(debug_str.contains("GridPosition"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_grid_position_clone() {
        let pos1 = GridPosition { x: 7, y: 14 };
        #[allow(clippy::clone_on_copy)]
        let pos2 = pos1.clone();
        assert_eq!(pos1.x, pos2.x);
        assert_eq!(pos1.y, pos2.y);
    }

    #[test]
    fn test_grid_position_fields() {
        let pos = GridPosition { x: 42, y: -7 };
        assert_eq!(pos.x, 42);
        assert_eq!(pos.y, -7);

        let pos2 = GridPosition { x: 0, y: 0 };
        assert_eq!(pos2.x, 0);
        assert_eq!(pos2.y, 0);
    }
}
