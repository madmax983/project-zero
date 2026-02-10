//! # Needs System
//!
//! This module simulates the physiological and psychological drives of Pops.
//! Needs decay over time and must be satisfied by actions (Eating, Sleeping, Socializing).
//!
//! ## 🧠 The Three Pillars
//!
//! 1.  **Hunger**: Represents caloric intake. Low hunger leads to starvation and death.
//! 2.  **Rest**: Represents physical energy. Low rest leads to exhaustion and collapse.
//! 3.  **Leisure**: Represents mental well-being. Low leisure leads to stress and mental breaks.
//!
//! ## 📉 The Decay Loop
//!
//! Every tick, `decay_needs_system` reduces each need by a small amount.
//! *   **Hunger**: ~0.001 per tick (Starvation in ~1.5 days).
//! *   **Rest**: ~0.001 per tick (Exhaustion in ~1.5 days).
//! *   **Leisure**: ~0.0015 per tick (Boredom in ~1 day).
//!
//! Policies like `Rationing` can slow down decay rates.

use crate::layer1::edicts::{ColonyPolicies, get_hunger_decay_modifier};
use bevy_ecs::prelude::*;

/// Pop survival needs.
///
/// Tracks the core statistics that keep a Pop alive and happy.
/// Values range from 0.0 (Critical) to 1.0 (Satisfied).
///
/// # The Hero's Journey: A Day in the Life
///
/// ```
/// use scale::layer1::needs::Needs;
///
/// let mut needs = Needs::default(); // Starts at 0.8 (Happy)
///
/// // Morning: Needs decay
/// needs.hunger -= 0.1;
/// needs.rest -= 0.05;
///
/// // Lunch time!
/// needs.hunger += 0.5;
/// needs.hunger = needs.hunger.min(1.0); // Clamp to max
///
/// // Check if happy
/// if needs.morale() > 0.8 {
///     println!("I love this colony!");
/// }
/// ```
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
    ///
    /// This is often used by AI to prioritize which need to address first.
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
    ///
    /// # The Math
    /// Currently a simple average: `(Hunger + Rest + Leisure) / 3.0`.
    #[must_use]
    pub fn morale(&self) -> f32 {
        (self.hunger + self.rest + self.leisure) / 3.0
    }
}

/// Returns work efficiency multiplier based on morale.
///
/// *   **High Morale (>= 0.8)**: 1.2x Speed (Inspired)
/// *   **Low Morale (<= 0.2)**: 0.5x Speed (Depressed)
/// *   **Normal**: 1.0x Speed
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

const HUNGER_DECAY_PER_TICK: f32 = 0.001; // ~800 ticks to starve from full
const REST_DECAY_PER_TICK: f32 = 0.001; // ~800 ticks to exhaust
const LEISURE_DECAY_PER_TICK: f32 = 0.0015; // Slightly faster than hunger/rest

/// Decays needs for all pops each tick.
///
/// Uses `par_iter_mut` for parallel processing across entities.
///
/// # Modifiers
/// *   Applies `ColonyPolicies` modifiers (e.g., Rationing reduces hunger decay).
/// *   Clamps all values to `0.0`.
pub fn decay_needs_system(mut query: Query<&mut Needs>, policies: Option<Res<ColonyPolicies>>) {
    let hunger_mod = policies.map_or(1.0, |p| get_hunger_decay_modifier(&p));
    let hunger_decay = HUNGER_DECAY_PER_TICK * hunger_mod;

    query.par_iter_mut().for_each(|mut needs| {
        needs.hunger = (needs.hunger - hunger_decay).max(0.0);
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
        assert_eq!(super::get_morale_efficiency(0.9), 1.2);
        assert_eq!(super::get_morale_efficiency(0.8), 1.2);
    }

    #[test]
    fn test_morale_efficiency_neutral() {
        // Normal morale (0.2 < m < 0.8) -> 1.0x speed
        assert_eq!(super::get_morale_efficiency(0.5), 1.0);
        assert_eq!(super::get_morale_efficiency(0.79), 1.0);
        assert_eq!(super::get_morale_efficiency(0.21), 1.0);
    }

    #[test]
    fn test_morale_efficiency_penalty() {
        // Low morale (<= 0.2) -> 0.5x speed
        assert_eq!(super::get_morale_efficiency(0.1), 0.5);
        assert_eq!(super::get_morale_efficiency(0.2), 0.5);
    }
}
