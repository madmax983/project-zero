//! Spatial primitives and map utilities.
//!
//! This module defines the core spatial components used throughout the simulation,
//! such as `GridPosition`.

use bevy_ecs::prelude::*;

/// Grid position in world space.
///
/// Used for any entity that occupies a specific tile on the `TerrainGrid`.
///
/// # Examples
///
/// ```
/// use scale::layer1::map::GridPosition;
///
/// let pos = GridPosition { x: 10, y: 5 };
/// assert_eq!(pos.x, 10);
/// ```
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPosition {
    /// The X coordinate (horizontal).
    pub x: i32,
    /// The Y coordinate (vertical).
    pub y: i32,
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
