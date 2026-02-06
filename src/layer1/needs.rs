use bevy_ecs::prelude::*;

/// Pop survival needs.
#[derive(Component, Clone, Copy, Debug)]
pub struct Needs {
    /// Hunger level: 0.0 = starving, 1.0 = full.
    pub hunger: f32,
    /// Rest level: 0.0 = exhausted, 1.0 = rested.
    pub rest: f32,
    /// Leisure level: 0.0 = bored, 1.0 = entertained.
    pub leisure: f32,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            hunger: 0.8,
            rest: 0.8,
            leisure: 0.8,
        }
    }
}

impl Needs {
    /// Returns the worst (lowest) need value.
    #[must_use]
    pub const fn worst(&self) -> f32 {
        let min_hr = if self.hunger < self.rest {
            self.hunger
        } else {
            self.rest
        };
        if min_hr < self.leisure {
            min_hr
        } else {
            self.leisure
        }
    }
}

const HUNGER_DECAY_PER_TICK: f32 = 0.001; // ~800 ticks to starve from full
const REST_DECAY_PER_TICK: f32 = 0.001; // ~800 ticks to exhaust
const LEISURE_DECAY_PER_TICK: f32 = 0.0015; // Slightly faster than hunger/rest

/// Decays needs for all pops each tick.
///
/// Uses `par_iter_mut` for parallel processing across entities.
pub fn decay_needs_system(mut query: Query<&mut Needs>) {
    query.par_iter_mut().for_each(|mut needs| {
        needs.hunger = (needs.hunger - HUNGER_DECAY_PER_TICK).max(0.0);
        needs.rest = (needs.rest - REST_DECAY_PER_TICK).max(0.0);
        needs.leisure = (needs.leisure - LEISURE_DECAY_PER_TICK).max(0.0);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use bevy_ecs::system::RunSystemOnce;

    fn setup() -> World {
        crate::setup::init_task_pools();
        World::new()
    }

    #[test]
    fn test_needs_default() {
        let needs = Needs::default();
        assert!((needs.hunger - 0.8).abs() < f32::EPSILON);
        assert!((needs.rest - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_needs_worst() {
        let needs1 = Needs {
            hunger: 0.5,
            rest: 0.7,
            leisure: 0.8,
        };
        assert!((needs1.worst() - 0.5).abs() < f32::EPSILON);

        let needs2 = Needs {
            hunger: 0.9,
            rest: 0.3,
            leisure: 0.8,
        };
        assert!((needs2.worst() - 0.3).abs() < f32::EPSILON);

        let needs3 = Needs {
            hunger: 0.5,
            rest: 0.5,
            leisure: 0.5,
        };
        assert!((needs3.worst() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_needs_clamped_to_zero() {
        let mut world = setup();
        world.spawn((
            Pop,
            Needs {
                hunger: 0.0001,
                rest: 0.0001,
                leisure: 0.0001,
            },
        ));

        world.run_system_once(decay_needs_system).unwrap();

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.hunger >= 0.0);
        assert!(needs.rest >= 0.0);
        assert!(needs.hunger < f32::EPSILON);
    }

    #[test]
    fn test_decay_needs_system() {
        let mut world = setup();
        world.spawn((Pop, Needs::default()));

        world.run_system_once(decay_needs_system).unwrap();

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.hunger < 0.8, "Hunger should have decayed");
        assert!(needs.rest < 0.8, "Rest should have decayed");
        assert!(needs.hunger >= 0.0, "Hunger should not be negative");
        assert!(needs.rest >= 0.0, "Rest should not be negative");
    }

    #[test]
    fn test_decay_multiple_ticks() {
        let mut world = setup();
        world.spawn((Pop, Needs::default()));

        for _ in 0..100 {
            world.run_system_once(decay_needs_system).unwrap();
        }

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.hunger < 0.71, "Hunger should decay significantly");
        assert!(needs.rest < 0.71, "Rest should decay");
    }
}
