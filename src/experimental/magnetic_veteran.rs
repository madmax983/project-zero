#![allow(clippy::type_complexity)]
//! Magnetic Veteran (Nova Feature).
//!
//! # The Spark
//! We have `WeatherType::MagneticStorm`, the `Trait::Veteran` personality trait, and psychological `Needs` (`rest`) and `StressTracker`.
//!
//! # The Feature
//! During a `MagneticStorm`, Pops with `Trait::Veteran` experience sensory overload reminiscent of past warfare.
//! This causes their `accumulated_stress` to rise rapidly (PTSD trigger), but the surge of adrenaline makes them hyper-vigilant, completely halting their need for `rest` and even slightly regenerating it.

use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::pop::Pop;
use crate::layer1::psychology::needs::Needs;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::psychology::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const VETERAN_MAGNETIC_STRESS: f32 = 0.5;
const VETERAN_MAGNETIC_REST_REGEN: f32 = 0.05;

pub fn magnetic_veteran_system(
    weather_state: Option<Res<WeatherState>>,
    mut pops: Query<(&Traits, &mut Needs, &mut StressTracker), With<Pop>>,
) {
    if let Some(weather) = weather_state {
        if weather.current_weather == WeatherType::MagneticStorm {
            for (traits, mut needs, mut stress) in pops.iter_mut() {
                if traits.has(Trait::Veteran) {
                    stress.accumulated_stress += VETERAN_MAGNETIC_STRESS;
                    needs.rest = (needs.rest + VETERAN_MAGNETIC_REST_REGEN).min(1.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(magnetic_veteran_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_magnetic_veteran_effects() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        let traits = {
            let mut t = Traits::default();
            t.add(Trait::Veteran);
            t
        };

        let pop = world
            .spawn((
                Pop,
                traits,
                Needs {
                    rest: 0.5,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
            ))
            .id();

        world.run_system_once(magnetic_veteran_system).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(
            stress.accumulated_stress > 0.0,
            "Veteran should gain stress during magnetic storm"
        );

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(
            needs.rest > 0.5,
            "Veteran should regenerate rest during magnetic storm"
        );
    }
}
