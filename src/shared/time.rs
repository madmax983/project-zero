use bevy_ecs::prelude::*;

/// Tracks the global simulation time and speed.
#[derive(Resource, Default)]
pub struct SimulationTime {
    /// The current simulation tick (update count).
    pub tick: u64,
    /// The current simulation speed.
    pub speed: SimSpeed,
    /// Accumulator for partial ticks when running at non-integer speeds or variable frame rates.
    pub accumulator: f32,
}

/// Defines the speed at which the simulation runs.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum SimSpeed {
    /// The simulation is paused.
    Paused,
    /// The simulation runs at 1x speed.
    #[default]
    Normal,
    /// The simulation runs at 3x speed.
    Fast,
    /// The simulation runs at 5x speed.
    Faster,
}

impl SimSpeed {
    /// Returns the number of ticks per second for this speed.
    #[must_use]
    pub const fn ticks_per_second(&self) -> f32 {
        match self {
            Self::Paused => 0.0,
            Self::Normal => 1.0,
            Self::Fast => 3.0,
            Self::Faster => 5.0,
        }
    }

    /// Returns a user-friendly label for the UI.
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
    fn test_sim_speed_values() {
        assert!((SimSpeed::Paused.ticks_per_second() - 0.0).abs() < f32::EPSILON);
        assert!((SimSpeed::Normal.ticks_per_second() - 1.0).abs() < f32::EPSILON);
        assert!((SimSpeed::Fast.ticks_per_second() - 3.0).abs() < f32::EPSILON);
        assert!((SimSpeed::Faster.ticks_per_second() - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sim_speed_labels() {
        assert!(SimSpeed::Paused.label().contains("Paused"));
        assert!(SimSpeed::Normal.label().contains("1x"));
    }

    #[test]
    fn test_simulation_time_default() {
        let sim_time = SimulationTime::default();
        assert_eq!(sim_time.tick, 0);
        assert_eq!(sim_time.speed, SimSpeed::Normal);
        assert!((sim_time.accumulator - 0.0).abs() < f32::EPSILON);
    }
}
