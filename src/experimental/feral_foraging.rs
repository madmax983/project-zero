#![allow(clippy::type_complexity)]
//! Feral Foraging (Nova Feature).
//!
//! # The Spark
//! We have `Trait::Feral`, `ActionType::Explore`, `WeatherType::Rain`, and `ColonyResources`.
//!
//! # The Feature
//! Pops with `Trait::Feral` are usually a liability because of their reduced intellect.
//! However, during rainy or stormy weather, their primitive instincts kick in.
//! When they are actively exploring (`ActionType::Explore`), they passively forage for moss, grubs, and wild fungi,
//! generating a slow trickle of `Food` for the colony directly.
//!
//! # The Potential
//! This gives the Feral trait a unique niche and rewards players for having these "less desirable" pops wander the map during bad weather.

use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::pop::Pop;
use crate::layer1::psychology::traits::{Trait, Traits};
use crate::layer1::resources::ColonyResources;
use bevy_ecs::prelude::*;

const FERAL_FORAGING_RATE: f32 = 0.05;

pub fn feral_foraging_system(
    weather_state: Option<Res<WeatherState>>,
    resources: Option<ResMut<ColonyResources>>,
    pops: Query<(&Traits, &PopAction), With<Pop>>,
) {
    if let Some(weather) = weather_state {
        if weather.current_weather == WeatherType::Rain
            || weather.current_weather == WeatherType::Storm
        {
            if let Some(mut res) = resources {
                for (traits, action) in pops.iter() {
                    if traits.has(Trait::Feral) && action.current == ActionType::Explore {
                        res.food += FERAL_FORAGING_RATE;
                    }
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(feral_foraging_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_feral_foraging_gains_food() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain,
            duration_remaining: 100,
        });

        world.insert_resource(ColonyResources {
            food: 10.0,
            ..ColonyResources::default()
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Feral);
            t
        };

        world.spawn((
            Pop,
            traits,
            PopAction {
                current: ActionType::Explore,
                ..Default::default()
            },
        ));

        world.run_system_once(feral_foraging_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(
            res.food > 10.0,
            "Food should increase when feral pop explores in rain"
        );
    }

    #[test]
    fn test_feral_foraging_wrong_weather() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        world.insert_resource(ColonyResources {
            food: 10.0,
            ..ColonyResources::default()
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Feral);
            t
        };

        world.spawn((
            Pop,
            traits,
            PopAction {
                current: ActionType::Explore,
                ..Default::default()
            },
        ));

        world.run_system_once(feral_foraging_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(
            (res.food - 10.0).abs() < f32::EPSILON,
            "Food should not increase in clear weather"
        );
    }

    #[test]
    fn test_feral_foraging_wrong_action() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain,
            duration_remaining: 100,
        });

        world.insert_resource(ColonyResources {
            food: 10.0,
            ..ColonyResources::default()
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Feral);
            t
        };

        world.spawn((
            Pop,
            traits,
            PopAction {
                current: ActionType::SatisfyHunger,
                ..Default::default()
            },
        ));

        world.run_system_once(feral_foraging_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(
            (res.food - 10.0).abs() < f32::EPSILON,
            "Food should not increase if not exploring"
        );
    }

    #[test]
    fn test_feral_foraging_not_feral() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::Rain,
            duration_remaining: 100,
        });

        world.insert_resource(ColonyResources {
            food: 10.0,
            ..ColonyResources::default()
        });

        world.spawn((
            Pop,
            Traits::default(),
            PopAction {
                current: ActionType::Explore,
                ..Default::default()
            },
        ));

        world.run_system_once(feral_foraging_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(
            (res.food - 10.0).abs() < f32::EPSILON,
            "Food should not increase if not feral"
        );
    }
}
