//! Simulation time management.
//!
//! This module handles the passage of time in the game world, distinguishing between
//! "Simulation Time" (game state updates) and "Wall Time" (real-world seconds).
//!
//! # The Time Model
//!
//! 1.  **Ticks ([`crate::shared::time::SimulationTime`])**: The fundamental quantum of game logic.
//!     *   Simulation systems run once per tick.
//!     *   1 tick is roughly 100ms of "game time" (at 1x speed).
//!     *   The game is deterministic based on ticks, not real time.
//!
//! 2.  **Speed ([`crate::shared::time::SimSpeed`])**: Controls how many ticks occur per second.
//!     *   **Paused**: 0 ticks/sec.
//!     *   **1x**: 10 ticks/sec.
//!     *   **3x**: 30 ticks/sec.
//!     *   **5x**: 50 ticks/sec.
//!
//! 3.  **Wall Time ([`crate::shared::time::WallTime`])**: Measures real-world seconds since app start.
//!     *   Used for UI animations (pulsing cursors, fading notifications) that must
//!         continue even when the game is paused.

use bevy_ecs::prelude::*;

/// Tracks the global simulation time and speed.
///
/// This resource is the "clock" of the colony.
///
/// # Examples
///
/// Reading the current tick:
/// ```
/// use scale::shared::time::SimulationTime;
///
/// let time = SimulationTime::default();
/// println!("Current tick: {}", time.tick);
/// ```
#[derive(Resource, Default)]
pub struct SimulationTime {
    /// The current simulation tick (update count).
    ///
    /// Monotonically increasing. Only increments when the game is unpaused.
    pub tick: u64,
    /// The current simulation speed target.
    pub speed: SimSpeed,
}

/// Tracks the wall-clock time for UI animations (e.g. pulsing cursors).
///
/// This is updated every frame (render loop), independent of simulation speed or pausing.
/// Use this for visual effects that shouldn't freeze when the game is paused.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct WallTime(pub f32);

/// Defines the speed at which the simulation runs.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SimSpeed {
    /// The simulation is paused (0 ticks/sec).
    Paused,
    /// The simulation runs at normal speed (10 ticks/sec).
    #[default]
    Normal,
    /// The simulation runs at fast speed (30 ticks/sec).
    Fast,
    /// The simulation runs at maximum speed (50 ticks/sec).
    Faster,
}

impl SimSpeed {
    /// Returns a user-friendly label for the UI.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::shared::time::SimSpeed;
    ///
    /// assert!(SimSpeed::Paused.label().contains("Paused"));
    /// assert!(SimSpeed::Normal.label().contains("1x"));
    /// ```
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Paused => "⏸ Paused",
            Self::Normal => "▶ 1x",
            Self::Fast => "▶▶ 3x",
            Self::Faster => "▶▶▶ 5x",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_speed_labels() {
        assert!(SimSpeed::Paused.label().contains("Paused"));
        assert!(SimSpeed::Normal.label().contains("1x"));
        assert!(SimSpeed::Fast.label().contains("3x"));
        assert!(SimSpeed::Faster.label().contains("5x"));
    }

    #[test]
    fn test_simulation_time_default() {
        let sim_time = SimulationTime::default();
        assert_eq!(sim_time.tick, 0);
        assert_eq!(sim_time.speed, SimSpeed::Normal);
    }

    #[test]
    fn test_wall_time_default() {
        let wall_time = WallTime::default();
        assert!((wall_time.0 - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sim_speed_transitions() {
        let mut speed = SimSpeed::Normal;
        assert_eq!(speed, SimSpeed::Normal);

        speed = SimSpeed::Fast;
        assert_eq!(speed, SimSpeed::Fast);

        speed = SimSpeed::Paused;
        assert_eq!(speed, SimSpeed::Paused);

        speed = SimSpeed::Faster;
        assert_eq!(speed, SimSpeed::Faster);
    }

    #[test]
    fn test_tick_increment() {
        let mut sim_time = SimulationTime::default();
        assert_eq!(sim_time.tick, 0);

        sim_time.tick += 1;
        assert_eq!(sim_time.tick, 1);

        sim_time.tick += 10;
        assert_eq!(sim_time.tick, 11);
    }

    #[test]
    fn test_speed_is_copy() {
        let speed1 = SimSpeed::Fast;
        let speed2 = speed1; // Should copy, not move
        assert_eq!(speed1, speed2);
    }

    #[test]
    fn test_speed_debug_format() {
        let speed = SimSpeed::Normal;
        let debug_str = format!("{speed:?}");
        assert!(debug_str.contains("Normal"));
    }

    #[test]
    fn test_simulation_time_speed_changes() {
        let mut sim_time = SimulationTime::default();
        assert_eq!(sim_time.speed, SimSpeed::Normal);

        sim_time.speed = SimSpeed::Fast;
        assert_eq!(sim_time.speed, SimSpeed::Fast);
        assert_eq!(sim_time.tick, 0); // Speed change doesn't affect tick

        sim_time.tick = 42;
        sim_time.speed = SimSpeed::Paused;
        assert_eq!(sim_time.tick, 42); // Tick preserved across speed change
    }
}
