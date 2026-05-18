//! Hoarder's Sleepwalking (Nova Feature).
//!
//! # The Spark
//! We have the `Sleepwalking` mental break (`ActionType::Sleepwalking`), which just makes them wander.
//! We also have `PrivateStash`.
//!
//! # The Feature
//! While a Pop is `Sleepwalking`, they passively siphon a tiny bit of `food` from the `ColonyResources`
//! and put it into their `PrivateStash` (if they have one), without realizing it. They wake up to find
//! their stash full of food they didn't consciously steal.

use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::pop::Pop;
use crate::layer1::private_stash::PrivateStash;
use crate::layer1::resources::{ColonyResources, ResourceType};
use bevy_ecs::prelude::*;

const SLEEPWALKING_FOOD_SIPHON: f32 = 0.01;

/// System that allows sleepwalking pops to passively steal food into their stash.
pub fn hoarder_sleepwalking_system(
    mut resources: ResMut<ColonyResources>,
    mut pops: Query<(&PopAction, &mut PrivateStash), With<Pop>>,
) {
    if resources.food <= 0.0 {
        return;
    }

    for (action, mut stash) in pops.iter_mut() {
        if action.current == ActionType::Sleepwalking {
            let steal_amount = SLEEPWALKING_FOOD_SIPHON.min(resources.food);
            if steal_amount > 0.0 {
                resources.food -= steal_amount;
                stash.add(ResourceType::Food, steal_amount);
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(hoarder_sleepwalking_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_hoarder_sleepwalking_siphons_food() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });

        let pop = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Sleepwalking,
                    ..Default::default()
                },
                PrivateStash::default(),
            ))
            .id();

        world.run_system_once(hoarder_sleepwalking_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        assert!((resources.food - (10.0 - SLEEPWALKING_FOOD_SIPHON)).abs() < f32::EPSILON);

        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert!((stash.get(ResourceType::Food) - SLEEPWALKING_FOOD_SIPHON).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hoarder_sleepwalking_ignores_non_sleepwalking() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            food: 10.0,
            ..Default::default()
        });

        let pop = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Idle,
                    ..Default::default()
                },
                PrivateStash::default(),
            ))
            .id();

        world.run_system_once(hoarder_sleepwalking_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        assert!((resources.food - 10.0).abs() < f32::EPSILON);

        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert_eq!(stash.get(ResourceType::Food), 0.0);
    }

    #[test]
    fn test_hoarder_sleepwalking_stops_when_food_empty() {
        let mut world = World::new();

        world.insert_resource(ColonyResources {
            food: 0.005, // Less than the siphon amount
            ..Default::default()
        });

        let pop = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::Sleepwalking,
                    ..Default::default()
                },
                PrivateStash::default(),
            ))
            .id();

        world.run_system_once(hoarder_sleepwalking_system).unwrap();

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.food, 0.0);

        let stash = world.get::<PrivateStash>(pop).unwrap();
        assert!((stash.get(ResourceType::Food) - 0.005).abs() < f32::EPSILON);
    }
}
