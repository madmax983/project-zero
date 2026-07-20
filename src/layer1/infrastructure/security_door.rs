//! Security infrastructure for the colony.
//!
//! This module provides components to restrict access to certain areas
//! based on clearance levels.

use bevy_ecs::prelude::*;

/// A door that restricts access based on a minimum clearance level.
///
/// Provides a physical barrier in the colony that only Pops with sufficient
/// clearance can pass through. The door also accounts for temporal drift
/// if the colony is experiencing chronal anomalies.
///
/// ## Examples
///
/// ```
/// use scale::layer1::infrastructure::security_door::SecurityDoor;
///
/// let door = SecurityDoor {
///     required_clearance: 3,
///     max_drift_tolerance: 0.5,
/// };
/// assert_eq!(door.required_clearance, 3);
/// ```
#[derive(Component)]
pub struct SecurityDoor {
    /// The minimum clearance level required to pass through this door.
    pub required_clearance: u32,
    /// The maximum temporal drift the door's sensors can tolerate before failing.
    pub max_drift_tolerance: f32,
}
