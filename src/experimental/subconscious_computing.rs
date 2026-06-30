#![allow(clippy::type_complexity)]
//! Subconscious Computing (Nova Feature).
//!
//! # The Spark
//! We have `Trait::Intellectual`, `ActionType::SatisfyRest`, and `ColonyResources::knowledge`.
//! Can pops be productive even while sleeping?
//!
//! # The Feature
//! Pops with `Trait::Intellectual` generate a trickle of `knowledge` passively while they
//! are performing `ActionType::SatisfyRest` (sleeping). However, because their brain never
//! truly shuts off to process these complex thoughts, their `rest` regeneration is
//! slightly penalized during sleep.
//!
//! # The Tension
//! The player gets free `knowledge` generation from intellectual pops, but these pops
//! will spend slightly more time sleeping and potentially burn out faster.

use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

const SUBCONSCIOUS_KNOWLEDGE_YIELD: f32 = 0.05;
const SUBCONSCIOUS_REST_PENALTY: f32 = 0.005;

pub fn subconscious_computing_system(
    resources: Option<ResMut<ColonyResources>>,
    mut pops: Query<(&Traits, &PopAction, &mut Needs), With<Pop>>,
) {
    if let Some(mut res) = resources {
        for (traits, action, mut needs) in pops.iter_mut() {
            if traits.has(Trait::Intellectual) && action.current == ActionType::SatisfyRest {
                res.knowledge += SUBCONSCIOUS_KNOWLEDGE_YIELD;
                // Penalize rest, but keep it bounded
                needs.rest = (needs.rest - SUBCONSCIOUS_REST_PENALTY).max(0.0);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(subconscious_computing_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_subconscious_computing_generates_knowledge_and_penalizes_rest() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            knowledge: 10.0,
            ..ColonyResources::default()
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Intellectual);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                PopAction {
                    current: ActionType::SatisfyRest,
                    ..Default::default()
                },
                Needs {
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(subconscious_computing_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(
            (res.knowledge - (10.0 + SUBCONSCIOUS_KNOWLEDGE_YIELD)).abs() < f32::EPSILON,
            "Knowledge should increase by {}", SUBCONSCIOUS_KNOWLEDGE_YIELD
        );

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            (needs.rest - (0.5 - SUBCONSCIOUS_REST_PENALTY)).abs() < f32::EPSILON,
            "Rest should decrease by {}", SUBCONSCIOUS_REST_PENALTY
        );
    }

    #[test]
    fn test_subconscious_computing_ignores_non_intellectuals() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            knowledge: 10.0,
            ..ColonyResources::default()
        });

        world.spawn((
            Pop,
            Traits::default(), // No Intellectual trait
            PopAction {
                current: ActionType::SatisfyRest,
                ..Default::default()
            },
            Needs {
                rest: 0.5,
                ..Default::default()
            },
        ));

        world.run_system_once(subconscious_computing_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(
            (res.knowledge - 10.0).abs() < f32::EPSILON,
            "Knowledge should not increase for non-intellectuals"
        );
    }

    #[test]
    fn test_subconscious_computing_ignores_awake_pops() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            knowledge: 10.0,
            ..ColonyResources::default()
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Intellectual);
            t
        };

        world.spawn((
            Pop,
            traits,
            PopAction {
                current: ActionType::Idle, // Awake
                ..Default::default()
            },
            Needs {
                rest: 0.5,
                ..Default::default()
            },
        ));

        world.run_system_once(subconscious_computing_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(
            (res.knowledge - 10.0).abs() < f32::EPSILON,
            "Knowledge should not increase when pop is awake"
        );
    }
}
