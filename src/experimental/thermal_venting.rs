#![allow(clippy::type_complexity)]
//! Thermal Venting (Nova Feature).
//!
//! # The Spark
//! We have `ActionType::ExtinguishFire`, `WeatherType::ThermalInversion`, `ColonyResources`, and `Pop`.
//!
//! # The Feature
//! If a pop is extinguishing a fire during a `ThermalInversion`, the intense trapped heat causes the water used
//! to flash-boil. This slightly damages the Pop (Steam Burn), immediately reduces the colony's `water` resource,
//! and randomly destroys a small amount of `wood` in the colony due to the explosive steam pressure ruining stored materials.
//!
//! # Boundaries
//! Purely additive. We query Pops currently performing `ExtinguishFire` while `WeatherType::ThermalInversion` is active.
//!
use crate::layer1::mind::utility_types::{ActionType, PopAction};
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::pop::Pop;
use crate::layer1::resources::ColonyResources;
use crate::layer1::shields::DamageEvent;
use bevy_ecs::prelude::*;
use rand::Rng;

pub fn thermal_venting_system(
    weather_state: Option<Res<WeatherState>>,
    resources: Option<ResMut<ColonyResources>>,
    pops: Query<(Entity, &PopAction), With<Pop>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    if let Some(weather) = weather_state {
        if weather.current_weather == WeatherType::ThermalInversion {
            if let Some(mut res) = resources {
                let mut rng = rand::thread_rng();
                for (entity, action) in pops.iter() {
                    if action.current == ActionType::ExtinguishFire {
                        // Flash boil!
                        // Damage the pop
                        damage_events.send(DamageEvent {
                            target: entity,
                            amount: 5.0,
                            velocity: 0.0, // Environmental damage
                        });

                        // Consume water rapidly
                        res.water = (res.water - 1.0).max(0.0);

                        // Randomly destroy some wood
                        if rng.gen_bool(0.25) {
                            res.wood = (res.wood - 1.0).max(0.0);
                        }
                    }
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(thermal_venting_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_thermal_venting() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::ThermalInversion,
            duration_remaining: 100,
        });

        world.insert_resource(ColonyResources {
            water: 10.0,
            wood: 10.0,
            ..ColonyResources::default()
        });

        world.init_resource::<Events<DamageEvent>>();

        let _pop = world
            .spawn((
                Pop,
                PopAction {
                    current: ActionType::ExtinguishFire,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(thermal_venting_system).unwrap();

        let res = world.resource::<ColonyResources>();
        assert!(res.water < 10.0, "Water should be consumed");

        let events = world.resource::<Events<DamageEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1, "Pop should be damaged");
    }
}
