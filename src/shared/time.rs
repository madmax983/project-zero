use bevy_ecs::prelude::*;

/// Tracks the global simulation time and speed.
#[derive(Resource, Default)]
pub struct SimulationTime {
    /// The current simulation tick (update count).
    pub tick: u64,
    /// The current simulation speed.
    pub speed: SimSpeed,
    /// Accumulator for partial ticks when running at non-integer speeds or variable frame rates.
    ///
    /// NOTE: Currently unused. Reserved for future implementation where speed multipliers
    /// will be applied via fractional tick accumulation. For now, all speeds increment by 1.
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
    ///
    /// NOTE: Currently unused in the main game loop. The tick increment logic (main.rs:89)
    /// always adds 1 per tick. This method is reserved for future implementation where
    /// the speed multiplier will be applied via the `accumulator` field.
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
        assert!(SimSpeed::Fast.label().contains("3x"));
        assert!(SimSpeed::Faster.label().contains("5x"));
    }

    #[test]
    fn test_simulation_time_default() {
        let sim_time = SimulationTime::default();
        assert_eq!(sim_time.tick, 0);
        assert_eq!(sim_time.speed, SimSpeed::Normal);
        assert!((sim_time.accumulator - 0.0).abs() < f32::EPSILON);
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

    #[test]
    fn test_accumulator_field_exists() {
        let mut sim_time = SimulationTime::default();
        assert!((sim_time.accumulator - 0.0).abs() < f32::EPSILON);

        sim_time.accumulator = 0.5;
        assert!((sim_time.accumulator - 0.5).abs() < f32::EPSILON);
    }
}
