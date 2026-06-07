#![allow(clippy::type_complexity)]
//! Ascetic Fasting (Nova Feature).
//!
//! # The Spark
//! We have the `Trait::Ascetic` personality trait, and physiological/psychological `Needs` (`hunger` and `leisure`).
//!
//! # The Feature
//! Pops with the `Trait::Ascetic` find spiritual or personal fulfillment through self-deprivation.
//! When their `hunger` reaches a critical level (< 0.2), they enter a state of "Ascetic Fasting".
//! Instead of panicking, the physical sensation of fasting actively regenerates their `leisure` (representing spiritual/mental clarity), turning a physiological emergency into a psychological buff.
//!
//! # The Potential
//! This creates an emergent narrative where certain Pops thrive when the colony's food supply is lowest.
//! It also adds a high-risk management strategy: intentionally starving ascetic Pops to keep their morale high during times of extreme unrest.

use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const ASCETIC_FASTING_HUNGER_THRESHOLD: f32 = 0.2;
const ASCETIC_FASTING_LEISURE_REGEN: f32 = 0.05;

pub fn ascetic_fasting_system(mut pops: Query<(&Traits, &mut Needs), With<Pop>>) {
    for (traits, mut needs) in pops.iter_mut() {
        if traits.has(Trait::Ascetic) && needs.hunger < ASCETIC_FASTING_HUNGER_THRESHOLD {
            needs.leisure = (needs.leisure + ASCETIC_FASTING_LEISURE_REGEN).min(1.0);
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(ascetic_fasting_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_ascetic_fasting_effects() {
        let mut world = World::new();

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Ascetic);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                Needs {
                    hunger: 0.1, // Below threshold
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(ascetic_fasting_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.leisure > 0.5,
            "Ascetic should regenerate leisure when starving"
        );
    }

    #[test]
    fn test_ascetic_not_fasting() {
        let mut world = World::new();

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Ascetic);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                Needs {
                    hunger: 0.5, // Above threshold
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(ascetic_fasting_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Ascetic should not regenerate leisure when not starving"
        );
    }

    #[test]
    fn test_non_ascetic_fasting() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                Traits::default(), // Not Ascetic
                Needs {
                    hunger: 0.1, // Below threshold
                    leisure: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(ascetic_fasting_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.leisure - 0.5).abs() < f32::EPSILON,
            "Non-ascetic should not regenerate leisure when starving"
        );
    }
}
