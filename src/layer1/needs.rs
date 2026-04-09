//! # Pop Needs
//!
//! This module simulates the physiological and psychological needs of every Pop in the colony.
//! Needs are the primary drivers of behavior in the [Utility AI](crate::layer1::utility_ai) system.
//!
//! ## Core Needs
//!
//! Each Pop has three core needs, represented as floating-point values from **0.0** (Critical) to **1.0** (Satisfied).
//!
//! 1.  **Hunger**:
//!     *   Decays over time (approx. 1000 ticks from full to starvation).
//!     *   Replenished by eating food (Action: `SatisfyHunger`).
//!     *   **Consequence**: At 0.0, Pops take starvation damage and eventually die.
//!
//! 2.  **Rest**:
//!     *   Decays over time (approx. 1000 ticks from rested to exhausted).
//!     *   Replenished by sleeping (Action: `SatisfyRest`).
//!     *   **Consequence**: Low rest reduces movement speed and work efficiency.
//!
//! 3.  **Leisure**:
//!     *   Decays slightly faster than other needs.
//!     *   Replenished by socializing, praying, or entertainment (Action: `Socialize`).
//!     *   **Consequence**: Low leisure contributes to stress and mental breaks.
//!
//! ## Decay Mechanics
//!
//! Every tick, `decay_needs_system` reduces these values based on constants and modifiers:
//!
//! *   **Base Decay**: Fixed rate per tick (e.g., `0.001` for Hunger).
//! *   **Traits**: [Traits](crate::layer1::traits) like `Glutton` increase hunger decay.
//! *   **Policies**: [Policies](crate::layer1::edicts) like `Rationing` reduce hunger decay at the cost of morale.
//!
//! ## Morale
//!
//! "Morale" is the aggregate score of all needs. High morale grants efficiency bonuses, while low morale
//! leads to mental breaks (tantrums, depression).

use crate::layer1::edicts::{get_hunger_decay_modifier, ColonyPolicies};
use crate::layer1::health::Health;
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::traits::{
    get_trait_hunger_decay_modifier, get_trait_leisure_decay_modifier, Traits,
};
use bevy_ecs::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NeedType {
    Hunger,
    Rest,
    Leisure,
    Hygiene,
    Food, // Alias used in spec tests
}

/// Pop survival needs.
///
/// Tracks the physical and mental state of a citizen. Values range from 0.0 (Empty/Critical) to 1.0 (Full/Satisfied).
///
/// # Default Values
///
/// New pops start with needs at **0.8** (80%), giving them a buffer before needing to act.
#[derive(Component, Clone, Copy, Debug)]
pub struct Needs {
    /// Hunger level.
    /// *   **1.0**: Full belly.
    /// *   **< 0.2**: Hungry (Urgent).
    /// *   **0.0**: Starving (Taking damage).
    pub hunger: f32,

    /// Rest level.
    /// *   **1.0**: Fully rested.
    /// *   **< 0.2**: Exhausted (Movement penalty).
    /// *   **0.0**: Collapsing.
    pub rest: f32,

    /// Leisure/Social level.
    /// *   **1.0**: Entertained.
    /// *   **< 0.2**: Bored/Stressed.
    pub leisure: f32,

    /// Hygiene level.
    /// *   **1.0**: Clean.
    /// *   **< 0.2**: Dirty/Unhappy.
    pub hygiene: f32,
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            hunger: 0.8,
            rest: 0.8,
            leisure: 0.8,
            hygiene: 0.8,
        }
    }
}

impl Needs {
    /// Returns the worst (lowest) need value.
    ///
    /// Used by the AI to determine the most pressing problem.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::needs::Needs;
    ///
    /// let needs = Needs { hunger: 0.9, rest: 0.2, leisure: 0.5, hygiene: 1.0 };
    /// assert_eq!(needs.worst(), 0.2); // Rest is the lowest
    /// ```
    #[must_use]
    pub fn get(&self, need_type: NeedType) -> f32 {
        match need_type {
            NeedType::Hunger | NeedType::Food => self.hunger,
            NeedType::Rest => self.rest,
            NeedType::Leisure => self.leisure,
            NeedType::Hygiene => self.hygiene,
        }
    }

    pub fn set(&mut self, need_type: NeedType, value: f32) {
        match need_type {
            NeedType::Hunger | NeedType::Food => self.hunger = value,
            NeedType::Rest => self.rest = value,
            NeedType::Leisure => self.leisure = value,
            NeedType::Hygiene => self.hygiene = value,
        }
    }

    pub const fn worst(&self) -> f32 {
        let min_hr = if self.hunger < self.rest {
            self.hunger
        } else {
            self.rest
        };
        let min_lh = if self.leisure < self.hygiene {
            self.leisure
        } else {
            self.hygiene
        };
        if min_hr < min_lh {
            min_hr
        } else {
            min_lh
        }
    }

    /// Calculates aggregate morale score (0.0 to 1.0).
    ///
    /// Simple average of all needs.
    ///
    /// # Examples
    ///
    /// ```
    /// use scale::layer1::needs::Needs;
    ///
    /// let needs = Needs { hunger: 1.0, rest: 0.5, leisure: 0.0, hygiene: 0.5 };
    /// // (1.0 + 0.5 + 0.0 + 0.5) / 4.0 = 0.5
    /// assert_eq!(needs.morale(), 0.5);
    /// ```
    #[must_use]
    pub fn morale(&self) -> f32 {
        (self.hunger + self.rest + self.leisure + self.hygiene) / 4.0
    }
}

/// Returns work efficiency multiplier based on morale.
///
/// *   **High Morale (>= 0.8)**: 120% Work Speed.
/// *   **Low Morale (<= 0.2)**: 50% Work Speed.
/// *   **Normal**: 100% Work Speed.
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

/// Applies damage to pops that are starving (hunger <= 0).
pub fn starvation_damage_system(world: &mut World) {
    let tick = world
        .get_resource::<crate::shared::time::SimulationTime>()
        .map_or(0, |t| t.tick);

    let mut query = world.query::<(&Needs, &mut Health, Option<&mut Memories>)>();
    for (needs, mut health, mut memories) in query.iter_mut(world) {
        if needs.hunger <= 0.0 {
            // Ludwig: Grace Period - Starving should feel urgent but not instant death.
            // 0.2 damage per tick -> 500 ticks (50s) to die.
            health.take_damage(0.2);

            if let Some(mem) = memories.as_mut() {
                mem.add(MemoryType::StarvationTrauma, tick);
            }
        }
    }
}

/// 0.1% decay per tick. Pop starves in ~1000 ticks from full (1.0).
/// (Assuming no traits or rationing).
const HUNGER_DECAY_PER_TICK: f32 = 0.001;

/// 0.1% decay per tick. Pop exhausts in ~1000 ticks from rested (1.0).
const REST_DECAY_PER_TICK: f32 = 0.001;

/// 0.15% decay per tick. Slightly faster than physical needs.
const LEISURE_DECAY_PER_TICK: f32 = 0.0015;

/// Decays needs for all pops each tick.
///
/// This system applies the constant decay rates to every entity with a [`Needs`] component.
/// It also checks for:
/// *   **Policies**: Adjusts hunger decay if [`ColonyPolicies`] are active (e.g. Rationing).
/// *   **Traits**: Adjusts hunger decay if the pop has specific [`Traits`] (e.g. Glutton).
///
type DecayNeedsFilter = (Without<crate::layer1::cryo::CryoStasis>, Without<crate::layer1::somnambulism::Somnambulist>);

/// # Threading
/// Uses `par_iter_mut` for parallel processing, as need decay is independent per pop.
pub fn decay_needs_system(
    mut query: Query<(&mut Needs, Option<&Traits>), DecayNeedsFilter>,
    policies: Option<Res<ColonyPolicies>>,
) {
    let hunger_mod = policies.map_or(1.0, |p| get_hunger_decay_modifier(&p));
    let base_hunger_decay = HUNGER_DECAY_PER_TICK * hunger_mod;

    query.par_iter_mut().for_each(|(mut needs, traits)| {
        let hunger_trait_mod = traits.map_or(1.0, get_trait_hunger_decay_modifier);
        let hunger_decay = base_hunger_decay * hunger_trait_mod;

        let leisure_trait_mod = traits.map_or(1.0, get_trait_leisure_decay_modifier);
        let leisure_decay = LEISURE_DECAY_PER_TICK * leisure_trait_mod;

        needs.hunger = (needs.hunger - hunger_decay).max(0.0);
        needs.rest = (needs.rest - REST_DECAY_PER_TICK).max(0.0);
        needs.leisure = (needs.leisure - leisure_decay).max(0.0);
        // Hygiene is decayed separately in hygiene.rs
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
        assert!((needs.hygiene - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_needs_worst() {
        let needs1 = Needs {
            hunger: 0.5,
            rest: 0.7,
            leisure: 0.8,
            hygiene: 0.9,
        };
        assert!((needs1.worst() - 0.5).abs() < f32::EPSILON);

        let needs2 = Needs {
            hunger: 0.9,
            rest: 0.3,
            leisure: 0.8,
            hygiene: 0.9,
        };
        assert!((needs2.worst() - 0.3).abs() < f32::EPSILON);

        let needs3 = Needs {
            hunger: 0.5,
            rest: 0.5,
            leisure: 0.5,
            hygiene: 0.1,
        };
        assert!((needs3.worst() - 0.1).abs() < f32::EPSILON);
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
                hygiene: 0.8,
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
            hygiene: 1.0,
        };
        assert!((needs.morale() - 1.0).abs() < f32::EPSILON);

        let needs_mixed = Needs {
            hunger: 0.5,
            rest: 0.5,
            leisure: 0.5,
            hygiene: 0.5,
        };
        assert!((needs_mixed.morale() - 0.5).abs() < f32::EPSILON);

        let needs_bad = Needs {
            hunger: 0.0,
            rest: 0.0,
            leisure: 0.0,
            hygiene: 0.0,
        };
        assert!((needs_bad.morale() - 0.0).abs() < f32::EPSILON);

        // Uneven
        let needs_uneven = Needs {
            hunger: 1.0,
            rest: 0.0,
            leisure: 0.5,
            hygiene: 0.5,
        };
        // (1+0+0.5+0.5)/4 = 2.0/4 = 0.5
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

    #[test]
    fn test_trait_hunger_decay() {
        use crate::layer1::traits::{Trait, Traits};

        let mut world = setup();
        let glutton = {
            let mut t = Traits::default();
            t.add(Trait::Glutton);
            t
        };

        world.spawn((Pop, Needs::default(), glutton));

        // Initial hunger 0.8
        // Glutton decay = Base (0.001) * 1.2 = 0.0012

        world.run_system_once(decay_needs_system).unwrap();

        let needs = world.query::<&Needs>().single(&world);
        // 0.8 - 0.0012 = 0.7988
        assert!((needs.hunger - 0.7988).abs() < 0.0001);
    }
}
