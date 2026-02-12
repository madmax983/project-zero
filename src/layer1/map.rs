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
use rand::Rng;

/// Resource to handle screen shake effects.
#[derive(Resource, Default, Debug)]
pub struct ScreenShake {
    /// Current intensity of the shake (0.0 to 5.0).
    pub intensity: f32,
    /// Current offset applied to the viewport.
    pub offset: (i32, i32),
}

impl ScreenShake {
    /// Triggers a screen shake with the given intensity (additive, capped at 5.0).
    pub fn trigger(&mut self, amount: f32) {
        self.intensity = (self.intensity + amount).min(5.0);
    }
}

/// System to update screen shake effects (decay and randomize offset).
pub fn update_screen_shake_system(mut shake: ResMut<ScreenShake>) {
    if shake.intensity > 0.0 {
        let mut rng = rand::thread_rng();
        // Calculate range based on intensity
        #[allow(clippy::cast_possible_truncation)]
        let range = (shake.intensity * 0.5).ceil() as i32;

        if range > 0 {
            shake.offset.0 = rng.gen_range(-range..=range);
            shake.offset.1 = rng.gen_range(-range..=range);
        } else {
            shake.offset = (0, 0);
        }

        // Decay intensity
        shake.intensity *= 0.9;
        if shake.intensity < 0.1 {
            shake.intensity = 0.0;
            shake.offset = (0, 0);
        }
    } else {
        shake.offset = (0, 0);
    }
}

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
    ///
    /// This returns a `u32` to safely handle distances that might overflow `i32`
    /// (e.g. between `i32::MIN` and `i32::MAX`).
    #[must_use]
    pub fn distance_chebyshev(&self, other: Self) -> u32 {
        self.x.abs_diff(other.x).max(self.y.abs_diff(other.y))
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

    #[test]
    fn test_screen_shake_system() {
        use bevy_ecs::system::RunSystemOnce;

        let mut world = World::new();
        world.insert_resource(ScreenShake::default());

        // Trigger shake
        world.resource_mut::<ScreenShake>().trigger(2.0);
        assert!((world.resource::<ScreenShake>().intensity - 2.0).abs() < f32::EPSILON);

        // Run update system
        world.run_system_once(update_screen_shake_system).unwrap();

        let shake = world.resource::<ScreenShake>();
        assert!(shake.intensity < 2.0); // Should decay (2.0 * 0.9 = 1.8)
        assert!(shake.intensity > 1.0);
    }
}
