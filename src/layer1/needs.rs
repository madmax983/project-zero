use crate::shared::log::MessageLog;
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

    /// Calculates aggregate morale score (0.0 to 1.0).
    #[must_use]
    pub fn morale(&self) -> f32 {
        (self.hunger + self.rest + self.leisure) / 3.0
    }
}

/// Returns work efficiency multiplier based on morale.
#[must_use]
pub fn get_morale_efficiency(morale: f32) -> f32 {
    if morale >= 0.8 {
        1.2
    } else if morale <= 0.2 {
        0.5
    } else {
        1.0
    }
}

/// Threshold for high morale (bonus efficiency).
pub const MORALE_HIGH_THRESHOLD: f32 = 0.8;
/// Threshold for low morale (penalty efficiency).
pub const MORALE_LOW_THRESHOLD: f32 = 0.2;

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

/// Despawns entities whose hunger has reached zero.
pub fn kill_starving_entities_system(
    query: Query<(Entity, &Needs)>,
    mut commands: Commands,
    log: Option<ResMut<MessageLog>>,
) {
    let mut log = log;
    for (entity, needs) in &query {
        if needs.hunger <= 0.0 {
            commands.entity(entity).despawn();
            if let Some(ref mut log) = log {
                log.add("DEATH: A colonist has starved to death!");
            }
        }
    }
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

    #[test]
    fn test_kill_starving_entities_system() {
        let mut world = World::new();

        world.spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.5,
                leisure: 0.5,
            },
        ));

        world.spawn((
            Pop,
            Needs {
                hunger: 0.0,
                rest: 0.5,
                leisure: 0.5,
            },
        ));

        world
            .run_system_once(kill_starving_entities_system)
            .unwrap();

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 1, "Only healthy pop should survive");
    }

    #[test]
    fn test_kill_only_when_hunger_zero() {
        let mut world = World::new();

        world.spawn((
            Pop,
            Needs {
                hunger: 0.01,
                rest: 0.0,
                leisure: 0.0,
            },
        ));

        world
            .run_system_once(kill_starving_entities_system)
            .unwrap();

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 1, "Pop with 0.01 hunger should survive");
    }

    #[test]
    fn test_starve_from_full() {
        let mut world = setup();
        world.spawn((Pop, Needs::default()));

        let mut ticks = 0;
        while world.query::<&Pop>().iter(&world).count() > 0 && ticks < 2000 {
            world.run_system_once(decay_needs_system).unwrap();
            world
                .run_system_once(kill_starving_entities_system)
                .unwrap();
            ticks += 1;
        }

        assert!(
            ticks < 850,
            "Pop should die within ~800 ticks from full (0.8)"
        );
        assert!(ticks > 750, "Pop should survive at least 750 ticks");
    }

    // Spec 031: Pop Morale
    #[test]
    fn test_calculate_morale() {
        let needs = Needs {
            hunger: 1.0,
            rest: 1.0,
            leisure: 1.0,
        };
        assert!((needs.morale() - 1.0).abs() < f32::EPSILON);

        let needs_mixed = Needs {
            hunger: 0.5,
            rest: 0.5,
            leisure: 0.5,
        };
        assert!((needs_mixed.morale() - 0.5).abs() < f32::EPSILON);

        let needs_bad = Needs {
            hunger: 0.0,
            rest: 0.0,
            leisure: 0.0,
        };
        assert!((needs_bad.morale() - 0.0).abs() < f32::EPSILON);

        // Uneven
        let needs_uneven = Needs {
            hunger: 1.0,
            rest: 0.0,
            leisure: 0.5,
        };
        assert!((needs_uneven.morale() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_morale_efficiency_bonus() {
        // High morale (>= 0.8) -> 1.2x speed
        assert_eq!(get_morale_efficiency(0.9), 1.2);
        assert_eq!(get_morale_efficiency(0.8), 1.2);
    }

    #[test]
    fn test_morale_efficiency_neutral() {
        // Normal morale (0.2 < m < 0.8) -> 1.0x speed
        assert_eq!(get_morale_efficiency(0.5), 1.0);
        assert_eq!(get_morale_efficiency(0.79), 1.0);
        assert_eq!(get_morale_efficiency(0.21), 1.0);
    }

    #[test]
    fn test_morale_efficiency_penalty() {
        // Low morale (<= 0.2) -> 0.5x speed
        assert_eq!(get_morale_efficiency(0.1), 0.5);
        assert_eq!(get_morale_efficiency(0.2), 0.5);
    }
}
