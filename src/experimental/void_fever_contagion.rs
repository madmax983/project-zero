#![allow(clippy::type_complexity)]
//! Void Fever Contagion (Nova Feature).
//!
//! # The Spark
//! We have `Trait::VoidTouched`, `WeatherType::SporeStorm`, and `Health`.
//! What if pops can catch a disease that spreads during specific weather?
//!
//! # The Feature
//! During a `SporeStorm`, pops with `Trait::VoidTouched` have a chance to contract `VoidFever`.
//! `VoidFever` grants them endless energy (rest never decays below 1.0) but rapidly drains their `Health`.
//! Furthermore, it can spread to adjacent pops, creating a mini-pandemic that burns out its hosts in exchange for frantic productivity.

use crate::layer1::biology::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;
use std::collections::HashSet;

/// Component indicating the pop has Void Fever.
#[derive(Component)]
pub struct VoidFever;

const INFECTION_CHANCE: f64 = 0.05;
const SPREAD_CHANCE: f64 = 0.2;
const HEALTH_DAMAGE_PER_TICK: f32 = 0.5;

/// System that spontaneously infects VoidTouched pops during a SporeStorm.
pub fn void_fever_infection_system(
    mut commands: Commands,
    weather_state: Option<Res<WeatherState>>,
    pops: Query<(Entity, &Traits), (With<Pop>, Without<VoidFever>)>,
) {
    if let Some(weather) = weather_state {
        if weather.current_weather == WeatherType::SporeStorm {
            let mut rng = rand::thread_rng();
            for (entity, traits) in pops.iter() {
                if traits.has(Trait::VoidTouched) && rng.gen_bool(INFECTION_CHANCE) {
                    commands.entity(entity).insert(VoidFever);
                }
            }
        }
    }
}

/// System that progresses the disease: damages health and pegs rest at 1.0.
pub fn void_fever_progression_system(
    mut pops: Query<(&mut Health, &mut Needs), With<VoidFever>>,
) {
    for (mut health, mut needs) in pops.iter_mut() {
        health.current -= HEALTH_DAMAGE_PER_TICK;
        needs.rest = 1.0;
    }
}

/// System that spreads Void Fever to adjacent pops.
pub fn void_fever_spread_system(
    mut commands: Commands,
    infected_pops: Query<&GridPosition, With<VoidFever>>,
    healthy_pops: Query<(Entity, &GridPosition), (With<Pop>, Without<VoidFever>)>,
) {
    let mut rng = rand::thread_rng();

    // Collect infected positions for fast lookup
    let infected_positions: HashSet<_> = infected_pops.iter().map(|p| (p.x, p.y)).collect();

    if infected_positions.is_empty() {
        return;
    }

    for (entity, pos) in healthy_pops.iter() {
        // Check neighbors (Moore neighborhood)
        let mut near_infected = false;
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if infected_positions.contains(&(pos.x + dx, pos.y + dy)) {
                    near_infected = true;
                    break;
                }
            }
            if near_infected {
                break;
            }
        }

        if near_infected && rng.gen_bool(SPREAD_CHANCE) {
            commands.entity(entity).insert(VoidFever);
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((
        void_fever_infection_system,
        void_fever_spread_system,
        void_fever_progression_system,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_void_fever_infection() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::SporeStorm,
            duration_remaining: 10,
        });

        let mut traits = Traits::default();
        traits.add(Trait::VoidTouched);

        let pop = world.spawn((Pop, traits)).id();

        // Run until infected
        for _ in 0..500 {
            world.run_system_once(void_fever_infection_system).unwrap();
            if world.get::<VoidFever>(pop).is_some() {
                break;
            }
        }

        assert!(world.get::<VoidFever>(pop).is_some());
    }

    #[test]
    fn test_void_fever_progression() {
        let mut world = World::new();

        let pop = world.spawn((
            Pop,
            VoidFever,
            Health {
                current: 100.0,
                max: 100.0,
                has_rust_lung: false,
            },
            Needs {
                rest: 0.0,
                ..Default::default()
            },
        )).id();

        world.run_system_once(void_fever_progression_system).unwrap();

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);

        let needs = world.get::<Needs>(pop).unwrap();
        assert_eq!(needs.rest, 1.0);
    }
}
