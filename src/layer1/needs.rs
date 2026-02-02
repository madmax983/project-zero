use super::pop::Pop;
use bevy_ecs::prelude::*;

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

const HUNGER_DECAY_PER_TICK: f32 = 0.02; // ~50 ticks to starve from full
const REST_DECAY_PER_TICK: f32 = 0.01; // ~100 ticks to exhaust

/// Decays needs for all pops each tick.
pub fn decay_needs_system(world: &mut World) {
    let mut query = world.query::<&mut Needs>();
    for mut needs in query.iter_mut(world) {
        needs.hunger = (needs.hunger - HUNGER_DECAY_PER_TICK).max(0.0);
        needs.rest = (needs.rest - REST_DECAY_PER_TICK).max(0.0);
    }
}

/// Despawns pops whose hunger has reached zero.
pub fn kill_starving_pops_system(world: &mut World) {
    // Collect entities to despawn (can't despawn while iterating)
    let to_despawn: Vec<Entity> = world
        .query_filtered::<(Entity, &Needs), With<Pop>>()
        .iter(world)
        .filter(|(_, needs)| needs.hunger <= 0.0)
        .map(|(entity, _)| entity)
        .collect();

    for entity in to_despawn {
        world.despawn(entity);
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
        assert_eq!(needs.hunger, 0.8);
        assert_eq!(needs.rest, 0.8);
    }

    #[test]
    fn test_needs_worst() {
        let needs1 = Needs {
            hunger: 0.5,
            rest: 0.7,
        };
        assert_eq!(needs1.worst(), 0.5);

        let needs2 = Needs {
            hunger: 0.9,
            rest: 0.3,
        };
        assert_eq!(needs2.worst(), 0.3);

        let needs3 = Needs {
            hunger: 0.5,
            rest: 0.5,
        };
        assert_eq!(needs3.worst(), 0.5);
    }

    #[test]
    fn test_needs_clamped_to_zero() {
        // Needs logic is handled by decay_needs_system, not struct setters.
        // We verify that the system clamps values.
        let mut world = World::new();
        world.spawn((
            Pop,
            Needs {
                hunger: 0.01,
                rest: 0.01,
            },
        ));

        decay_needs_system(&mut world);

        let needs = world.query::<&Needs>().single(&world);
        assert!(needs.hunger >= 0.0);
        assert!(needs.rest >= 0.0);
        // Hunger: 0.01 - 0.02 = -0.01 -> clamped to 0.0
        assert_eq!(needs.hunger, 0.0);
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

        for _ in 0..10 {
            decay_needs_system(&mut world);
        }

        let needs = world.query::<&Needs>().single(&world);
        // After 10 ticks of decay: 0.8 - (10 * 0.02) = 0.6
        // Allow for floating point epsilon
        assert!(needs.hunger < 0.61, "Hunger should decay significantly");
        assert!(needs.rest < 0.71, "Rest should decay");
    }

    #[test]
    fn test_kill_starving_pops_system() {
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

        kill_starving_pops_system(&mut world);

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

        kill_starving_pops_system(&mut world);

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

        assert_eq!(ch, '☺');
        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn test_pop_display_basic_warning() {
        let needs = Needs {
            hunger: 0.5,
            rest: 0.8,
        };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, '☻');
        assert_eq!(color, Color::Rgb(255, 165, 0)); // Orange
    }

    #[test]
    fn test_pop_display_basic_critical() {
        let needs = Needs {
            hunger: 0.2,
            rest: 0.8,
        };
        let (ch, color) = pop_display(&needs);

        assert_eq!(ch, '☹');
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

        assert_eq!(ch, '☻'); // Warning state
        assert_eq!(color, Color::Rgb(255, 165, 0));
    }

    #[test]
    fn test_starve_from_full() {
        let mut world = World::new();
        world.spawn((Pop, Needs::default()));

        // Run until pop dies
        let mut ticks = 0;
        while world.query::<&Pop>().iter(&world).count() > 0 && ticks < 100 {
            decay_needs_system(&mut world);
            kill_starving_pops_system(&mut world);
            ticks += 1;
        }

        assert!(
            ticks < 50,
            "Pop should die within ~50 ticks from full (0.8)"
        );
        assert!(ticks > 30, "Pop should survive at least 30 ticks");
    }
}
