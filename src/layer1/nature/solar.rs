use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
use crate::layer1::energy::PowerSource;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

/// Phases of the long-term solar cycle affecting global solar power output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SolarCycle {
    /// Solar Minimum (80% power).
    #[default]
    Minimum,
    /// Solar Rising (100% power).
    Rising,
    /// Solar Maximum (150% power).
    Maximum,
    /// Solar Falling (120% power).
    Falling,
}

impl SolarCycle {
    /// Returns the next phase in the cycle.
    #[must_use]
    pub const fn next(&self) -> Self {
        match self {
            Self::Minimum => Self::Rising,
            Self::Rising => Self::Maximum,
            Self::Maximum => Self::Falling,
            Self::Falling => Self::Minimum,
        }
    }

    /// Returns the power generation multiplier for this phase.
    #[must_use]
    pub const fn power_modifier(&self) -> f32 {
        match self {
            Self::Minimum => 0.8,
            Self::Rising => 1.0,
            Self::Maximum => 1.5,
            Self::Falling => 1.2,
        }
    }

    /// Returns the human-readable label for this phase.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Minimum => "Solar Minimum",
            Self::Rising => "Solar Rising",
            Self::Maximum => "Solar Maximum",
            Self::Falling => "Solar Falling",
        }
    }
}

/// Resource tracking the current solar cycle state.
#[derive(Resource, Default)]
pub struct SolarCycleState {
    /// The current active solar cycle phase.
    pub current_cycle: SolarCycle,
}

/// Component for buildings that generate solar power.
/// Tracks the base output before modifiers.
#[derive(Component, Default)]
pub struct SolarPower {
    /// The base power output of the device at 100% efficiency.
    pub base_output: f32,
}

/// System to update the solar cycle based on simulation year.
pub fn update_solar_cycle_system(mut state: ResMut<SolarCycleState>, time: Res<SimulationTime>) {
    let year = time.tick / TICKS_PER_YEAR;
    let cycle_index = year % 4;

    let new_cycle = match cycle_index {
        0 => SolarCycle::Minimum,
        1 => SolarCycle::Rising,
        2 => SolarCycle::Maximum,
        _ => SolarCycle::Falling,
    };

    if state.current_cycle != new_cycle {
        state.current_cycle = new_cycle;
    }
}

/// System to update solar power output based on cycle and time of day.
pub fn update_solar_output_system(
    state: Res<SolarCycleState>,
    day_night: Option<Res<DayNightCycle>>,
    mut query: Query<(&mut PowerSource, &SolarPower)>,
) {
    let cycle_modifier = state.current_cycle.power_modifier();

    let day_modifier = if let Some(cycle) = day_night {
        match cycle.time_of_day {
            TimeOfDay::Night => 0.0,
            TimeOfDay::Dawn | TimeOfDay::Dusk => 0.5,
            TimeOfDay::Day => 1.0,
        }
    } else {
        1.0
    };

    let total_modifier = cycle_modifier * day_modifier;

    for (mut source, solar) in &mut query {
        source.output = solar.base_output * total_modifier;
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::balance::TICKS_PER_YEAR;
    use crate::layer1::energy::PowerSource;
    use crate::layer1::solar::{
        update_solar_cycle_system, update_solar_output_system, SolarCycle, SolarCycleState,
        SolarPower,
    };
    use crate::shared::time::SimulationTime;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_solar_cycle_phases() {
        assert_eq!(SolarCycle::Minimum.next(), SolarCycle::Rising);
        assert_eq!(SolarCycle::Rising.next(), SolarCycle::Maximum);
        assert_eq!(SolarCycle::Maximum.next(), SolarCycle::Falling);
        assert_eq!(SolarCycle::Falling.next(), SolarCycle::Minimum);
    }

    #[test]
    fn test_solar_modifier() {
        assert!((SolarCycle::Minimum.power_modifier() - 0.8).abs() < f32::EPSILON);
        assert!((SolarCycle::Rising.power_modifier() - 1.0).abs() < f32::EPSILON);
        assert!((SolarCycle::Maximum.power_modifier() - 1.5).abs() < f32::EPSILON);
        assert!((SolarCycle::Falling.power_modifier() - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_cycle_update_system() {
        let mut world = World::new();
        world.insert_resource(SolarCycleState::default()); // Defaults to Minimum
        world.insert_resource(SimulationTime {
            tick: 0,
            ..Default::default()
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_solar_cycle_system);

        // Year 0 (0 to TICKS_PER_YEAR-1) -> Minimum
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<SolarCycleState>().current_cycle,
            SolarCycle::Minimum
        );

        // Year 1 (TICKS_PER_YEAR to 2*TICKS_PER_YEAR-1) -> Rising
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR;
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<SolarCycleState>().current_cycle,
            SolarCycle::Rising
        );

        // Year 2 -> Maximum
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR * 2;
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<SolarCycleState>().current_cycle,
            SolarCycle::Maximum
        );

        // Year 3 -> Falling
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR * 3;
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<SolarCycleState>().current_cycle,
            SolarCycle::Falling
        );

        // Year 4 -> Minimum
        world.resource_mut::<SimulationTime>().tick = TICKS_PER_YEAR * 4;
        schedule.run(&mut world);
        assert_eq!(
            world.resource::<SolarCycleState>().current_cycle,
            SolarCycle::Minimum
        );
    }

    #[test]
    fn test_solar_output_scaling() {
        let mut world = World::new();
        world.insert_resource(SolarCycleState {
            current_cycle: SolarCycle::Maximum,
        });

        // Spawn a solar panel with Base Output 10.0
        let panel = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    ..Default::default()
                },
                SolarPower { base_output: 10.0 },
            ))
            .id();

        // Spawn a generator (non-solar)
        let generator = world
            .spawn((PowerSource {
                output: 10.0,
                ..Default::default()
            },))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_solar_output_system);

        // Run system
        schedule.run(&mut world);

        // Panel should be boosted by 1.5x -> 15.0
        let panel_power = world.get::<PowerSource>(panel).unwrap();
        assert!((panel_power.output - 15.0).abs() < 0.001);

        // Generator should remain 10.0
        let gen_power = world.get::<PowerSource>(generator).unwrap();
        assert!((gen_power.output - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_solar_night_output() {
        use crate::layer1::day_night::{DayNightCycle, TimeOfDay};

        let mut world = World::new();
        world.insert_resource(SolarCycleState::default()); // Minimum -> 0.8

        let cycle = DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        };
        world.insert_resource(cycle);

        // Spawn solar panel
        let panel = world
            .spawn((
                PowerSource {
                    output: 10.0,
                    ..Default::default()
                },
                SolarPower { base_output: 10.0 },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_solar_output_system);

        schedule.run(&mut world);

        let power = world.get::<PowerSource>(panel).unwrap();
        assert!(power.output.abs() < f32::EPSILON);
    }
}
