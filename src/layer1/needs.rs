use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use ratatui::style::Color;

/// Pop survival needs.
#[derive(Component, Clone, Copy, Debug)]
pub struct Needs {
    /// Hunger level: 0.0 = starving, 1.0 = full.
    pub hunger: f32,
    /// Rest level: 0.0 = exhausted, 1.0 = rested.
    pub rest: f32,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            hunger: 0.8,
            rest: 0.8,
        }
    }
}

impl Needs {
    /// Returns the worst (lowest) need value.
    #[must_use]
    pub const fn worst(&self) -> f32 {
        if self.hunger < self.rest {
            self.hunger
        } else {
            self.rest
        }
    }
}

/// Threshold above which a pop is considered healthy (happy).
pub const HEALTHY_THRESHOLD: f32 = 0.6;
/// Threshold below which a pop is considered in warning state.
pub const WARNING_THRESHOLD: f32 = 0.3;

/// Component marker for entities that have already triggered a starvation warning.
#[derive(Component)]
pub struct StarvationWarned;

const HUNGER_DECAY_PER_TICK: f32 = 0.001; // ~800 ticks to starve from full
const REST_DECAY_PER_TICK: f32 = 0.001; // ~800 ticks to exhaust

/// Decays needs for all pops each tick.
pub fn decay_needs_system(world: &mut World) {
    let mut query = world.query::<&mut Needs>();
    for mut needs in query.iter_mut(world) {
        needs.hunger = (needs.hunger - HUNGER_DECAY_PER_TICK).max(0.0);
        needs.rest = (needs.rest - REST_DECAY_PER_TICK).max(0.0);
    }
}

/// Despawns entities whose hunger has reached zero.
pub fn kill_starving_entities_system(world: &mut World) {
    // Collect entities to despawn (can't despawn while iterating)
    let to_despawn: Vec<Entity> = world
        .query::<(Entity, &Needs)>()
        .iter(world)
        .filter(|(_, needs)| needs.hunger <= 0.0)
        .map(|(entity, _)| entity)
        .collect();

    for entity in to_despawn {
        world.despawn(entity);
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored("A colonist has starved to death!", Color::Red);
        }
    }
}

/// Checks for starving pops and logs a warning.
pub fn check_starvation_warning_system(world: &mut World) {
    // 1. Warn new starving pops
    let mut to_warn = Vec::new();
    for (entity, needs) in world
        .query_filtered::<(Entity, &Needs), Without<StarvationWarned>>()
        .iter(world)
    {
        if needs.hunger < WARNING_THRESHOLD && needs.hunger > 0.0 {
            to_warn.push(entity);
        }
    }

    if !to_warn.is_empty() {
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored("Warning: A colonist is starving!", Color::Rgb(255, 165, 0));
        }
        for entity in to_warn {
            world.entity_mut(entity).insert(StarvationWarned);
        }
    }

    // 2. Reset warning for recovered pops
    let mut to_reset = Vec::new();
    for (entity, needs) in world
        .query_filtered::<(Entity, &Needs), With<StarvationWarned>>()
        .iter(world)
    {
        if needs.hunger >= WARNING_THRESHOLD {
            to_reset.push(entity);
        }
    }

    for entity in to_reset {
        world.entity_mut(entity).remove::<StarvationWarned>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::{Pop, pop_display};
    use ratatui::style::Color;

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
        };
        assert!((needs1.worst() - 0.5).abs() < f32::EPSILON);

        let needs2 = Needs {
            hunger: 0.9,
            rest: 0.3,
        };
        assert!((needs2.worst() - 0.3).abs() < f32::EPSILON);

        let needs3 = Needs {
            hunger: 0.5,
            rest: 0.5,
        };
        assert!((needs3.worst() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_needs_clamped_to_zero() {
        // Needs logic is handled by decay_needs_system, not struct setters.
        // We verify that the system clamps values.
        let mut world = World::new();
        world.spawn((
            Pop,
            Needs {
                hunger: 0.0001,
                rest: 0.0001,
            },
        ));

        decay_needs_system(&mut world);

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.hunger >= 0.0);
        assert!(needs.rest >= 0.0);
        // Hunger: 0.0001 - 0.001 = -0.0009 -> clamped to 0.0
        assert!(needs.hunger < f32::EPSILON);
    }

    // test_needs_clamped_to_one removed as no system currently increases needs.

    #[test]
    fn test_decay_needs_system() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        decay_needs_system(&mut world);

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.hunger < 0.8, "Hunger should have decayed");
        assert!(needs.rest < 0.8, "Rest should have decayed");
        assert!(needs.hunger >= 0.0, "Hunger should not be negative");
        assert!(needs.rest >= 0.0, "Rest should not be negative");
    }

    #[test]
    fn test_decay_multiple_ticks() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        for _ in 0..100 {
            decay_needs_system(&mut world);
        }

        let needs = world.query::<&Needs>().single(&world);
        // After 100 ticks of decay: 0.8 - (100 * 0.001) = 0.7
        // Allow for floating point epsilon
        assert!(needs.hunger < 0.71, "Hunger should decay significantly");
        assert!(needs.rest < 0.71, "Rest should decay");
    }

    #[test]
    fn test_kill_starving_entities_system() {
        let mut world = World::new();

        // Spawn healthy pop
        world.spawn((
            Pop,
            Needs {
                hunger: 0.5,
                rest: 0.5,
            },
        ));

        // Spawn starving pop
        world.spawn((
            Pop,
            Needs {
                hunger: 0.0,
                rest: 0.5,
            },
        ));

        kill_starving_entities_system(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 1, "Only healthy pop should survive");
    }

    #[test]
    fn test_kill_only_when_hunger_zero() {
        let mut world = World::new();

        // Pop with very low hunger but not zero
        world.spawn((
            Pop,
            Needs {
                hunger: 0.01,
                rest: 0.0,
            },
        ));

        kill_starving_entities_system(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 1, "Pop with 0.01 hunger should survive");
    }

    #[test]
    fn test_pop_display_basic_healthy() {
        let needs = Needs {
            hunger: 0.8,
            rest: 0.8,
        };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, "☺");
        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn test_pop_display_basic_warning() {
        let needs = Needs {
            hunger: 0.5,
            rest: 0.8,
        };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, "☻");
        assert_eq!(color, Color::Rgb(255, 165, 0)); // Orange
    }

    #[test]
    fn test_pop_display_basic_critical() {
        let needs = Needs {
            hunger: 0.2,
            rest: 0.8,
        };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, "☹");
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_pop_display_basic_uses_worst_need() {
        // Even if hunger is high, low rest should trigger warning
        let needs = Needs {
            hunger: 0.9,
            rest: 0.4,
        };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, "☻"); // Warning state
        assert_eq!(color, Color::Rgb(255, 165, 0));
    }

    #[test]
    fn test_starve_from_full() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        // Run until pop dies
        let mut ticks = 0;
        while world.query::<&Pop>().iter(&world).count() > 0 && ticks < 2000 {
            decay_needs_system(&mut world);
            kill_starving_entities_system(&mut world);
            ticks += 1;
        }

        assert!(
            ticks < 850,
            "Pop should die within ~800 ticks from full (0.8)"
        );
        assert!(ticks > 750, "Pop should survive at least 750 ticks");
    }

    #[test]
    fn test_check_starvation_warning_warns() {
        let mut world = World::new();
        // Hunger 0.2 < WARNING_THRESHOLD (0.3)
        let entity = world.spawn((
            Pop,
            Needs {
                hunger: 0.2,
                rest: 0.8,
            },
        )).id();
        world.insert_resource(MessageLog::default());

        check_starvation_warning_system(&mut world);

        // Should have StarvationWarned
        assert!(world.get::<StarvationWarned>(entity).is_some());

        // Should have logged
        let log = world.resource::<MessageLog>();
        assert_eq!(
            log.messages.back().unwrap().text,
            "Warning: A colonist is starving!"
        );
    }

    #[test]
    fn test_check_starvation_warning_no_spam() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.2,
                    rest: 0.8,
                },
                StarvationWarned, // Already warned
            ))
            .id();
        world.insert_resource(MessageLog::default());

        check_starvation_warning_system(&mut world);

        // Should still have StarvationWarned
        assert!(world.get::<StarvationWarned>(entity).is_some());

        // Should NOT have logged
        let log = world.resource::<MessageLog>();
        assert!(log.messages.is_empty());
    }

    #[test]
    fn test_check_starvation_warning_reset() {
        let mut world = World::new();
        // Hunger 0.4 > WARNING_THRESHOLD (0.3)
        let entity = world
            .spawn((
                Pop,
                Needs {
                    hunger: 0.4,
                    rest: 0.8,
                },
                StarvationWarned, // Was previously starving
            ))
            .id();

        check_starvation_warning_system(&mut world);

        // Should remove StarvationWarned
        assert!(world.get::<StarvationWarned>(entity).is_none());
    }
}
