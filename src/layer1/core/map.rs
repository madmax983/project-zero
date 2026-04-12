#![allow(clippy::cast_precision_loss)]
//! Spatial primitives and map utilities.
//!
//! This module defines the foundational spatial component [`GridPosition`] and the camera/viewport logic.
//!
//! # The Coordinate System
//!
//! The world is represented as a 2D grid. The coordinate system follows these rules:
//! * **Origin (0,0)**: The top-left corner of the map.
//! * **X-Axis**: Increases to the right (East).
//! * **Y-Axis**: Increases downwards (South).
//!
//! ## Type Safety (`i32`)
//! We use `i32` for coordinates rather than `usize` or `u32` because:
//! 1.  **Safety**: Prevents underflow panics when calculating relative offsets (e.g., `x - 1` at the left edge).
//! 2.  **Flexibility**: Allows for off-map entities (spawners, cinematic cameras) or infinite scrolling in the future.
//! 3.  **Chebyshev Distance**: Simplifies distance math (`abs()`).
//!
//! # Logical vs. Visual Position
//!
//! The simulation has two concepts of "position":
//!
//! 1.  **Logical ([`GridPosition`])**: Discrete tile coordinates. Used for gameplay logic (pathfinding, collision).
//!     *   *Truth*: An entity is exactly at (10, 5).
//!
//! 2.  **Visual ([`CameraCurrent`])**: Continuous float coordinates. Used for rendering interpolation.
//!     *   *Truth*: The camera is currently looking at (10.2, 5.0) while panning.
//!
//! Systems like [`update_camera_smooth`] bridge this gap by lerping the visual position towards the logical target.

use bevy_ecs::prelude::*;
use rand::Rng;

/// Resource to handle screen shake effects (juice).
///
/// Use this to add impact to events like explosions, heavy machinery, or earthquakes.
///
/// # Examples
///
/// ```
/// use scale::layer1::map::ScreenShake;
///
/// // Create a shake instance
/// let mut shake = ScreenShake::default();
///
/// // Trigger a small rumble (e.g. door slam)
/// shake.trigger(0.5);
///
/// // Trigger a massive quake
/// shake.trigger(5.0); // Clamped to 5.0 max
/// ```
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
///
/// This runs every frame to update the visual offset.
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

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShadowLayer;

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

    /// Returns the Manhattan distance (taxicab distance) between two positions.
    /// This is `|x1 - x2| + |y1 - y2|`.
    ///
    /// Returns `u64` because the sum of two max `u32` diffs can exceed `u32::MAX`.
    #[must_use]
    pub fn distance_manhattan(&self, other: Self) -> u64 {
        u64::from(self.x.abs_diff(other.x)) + u64::from(self.y.abs_diff(other.y))
    }

    /// Returns the direction vector (dx, dy) to another position.
    /// Values are -1, 0, or 1.
    /// Uses `cmp` to avoid overflow from `other.x - self.x`.
    #[must_use]
    pub fn direction_to(&self, other: Self) -> (i32, i32) {
        (
            match other.x.cmp(&self.x) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            },
            match other.y.cmp(&self.y) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            },
        )
    }
}

/// Target position for smooth camera movement (The "Destination").
///
/// This is where the camera *wants* to be.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct CameraTarget {
    /// Target X coordinate.
    pub x: f32,
    /// Target Y coordinate.
    pub y: f32,
}

/// Current interpolated position of the camera (Float precision).
///
/// This is where the camera *actually is* this frame.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct CameraCurrent {
    /// Current X coordinate.
    pub x: f32,
    /// Current Y coordinate.
    pub y: f32,
}

/// Updates the viewport position to smoothly interpolate towards the camera target.
///
/// This function implements a standard "Asymptotic Averaging" (Lerp) smoothing.
/// It should be called every frame (update loop) rather than every simulation tick
/// to ensure smooth animation at high refresh rates.
#[allow(clippy::cast_possible_truncation)]
pub fn update_camera_smooth(world: &mut World) {
    // 1. Ensure Resources exist
    // We access Viewport first to initialize others if needed
    let viewport_vals = if let Some(v) = world.get_resource::<crate::layer1::terrain::Viewport>() {
        (v.x, v.y)
    } else {
        return;
    };

    if world.get_resource::<CameraTarget>().is_none() {
        world.insert_resource(CameraTarget {
            x: viewport_vals.0 as f32,
            y: viewport_vals.1 as f32,
        });
    }
    if world.get_resource::<CameraCurrent>().is_none() {
        world.insert_resource(CameraCurrent {
            x: viewport_vals.0 as f32,
            y: viewport_vals.1 as f32,
        });
    }

    // 2. Interpolate
    // We limit the scope of borrows
    let (tx, ty) = {
        let t = world.resource::<CameraTarget>();
        (t.x, t.y)
    };

    let (mut cx, mut cy) = {
        let c = world.resource::<CameraCurrent>();
        (c.x, c.y)
    };

    // Lerp Factor (Frame independent ideally, but assuming ~60fps for now or rate-limited in main)
    // 0.2 provides a snappy but smooth feel.
    let t = 0.2;

    cx += (tx - cx) * t;
    cy += (ty - cy) * t;

    // Snap if close enough (to avoid infinite micro-floats)
    if (tx - cx).abs() < 0.01 {
        cx = tx;
    }
    if (ty - cy).abs() < 0.01 {
        cy = ty;
    }

    // Update CameraCurrent
    {
        let mut c = world.resource_mut::<CameraCurrent>();
        c.x = cx;
        c.y = cy;
    }

    // 3. Update Viewport (Integer representation for rendering)
    if let Some(mut v) = world.get_resource_mut::<crate::layer1::terrain::Viewport>() {
        v.x = cx.round() as i32;
        v.y = cy.round() as i32;
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

    #[test]
    fn test_camera_smooth_initialization() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::Viewport { x: 10, y: 20 });

        // Run once to lazy-init
        update_camera_smooth(&mut world);

        let target = world.resource::<CameraTarget>();
        assert!((target.x - 10.0).abs() < f32::EPSILON);
        assert!((target.y - 20.0).abs() < f32::EPSILON);

        let current = world.resource::<CameraCurrent>();
        assert!((current.x - 10.0).abs() < f32::EPSILON);
        assert!((current.y - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_camera_smooth_interpolation() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::Viewport { x: 0, y: 0 });
        world.insert_resource(CameraTarget { x: 10.0, y: 0.0 });
        world.insert_resource(CameraCurrent { x: 0.0, y: 0.0 });

        update_camera_smooth(&mut world);

        let current = world.resource::<CameraCurrent>();
        // Lerp 0 -> 10 with t=0.2 => 0 + (10-0)*0.2 = 2.0
        assert!((current.x - 2.0).abs() < 0.001);

        let viewport = world.resource::<crate::layer1::terrain::Viewport>();
        assert_eq!(viewport.x, 2);
    }

    #[test]
    fn test_camera_smooth_snap() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::Viewport { x: 10, y: 0 });
        world.insert_resource(CameraTarget { x: 10.0, y: 0.0 });
        world.insert_resource(CameraCurrent { x: 9.995, y: 0.0 });

        update_camera_smooth(&mut world);

        let current = world.resource::<CameraCurrent>();
        // Should snap to 10.0 because abs diff < 0.01
        assert!((current.x - 10.0).abs() < f32::EPSILON);

        let viewport = world.resource::<crate::layer1::terrain::Viewport>();
        assert_eq!(viewport.x, 10);
    }
}
