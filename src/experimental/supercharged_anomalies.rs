//! Supercharged Anomalies (Nova Feature)
//!
//! # The Spark
//! We have a `Supercharge` concept from `magnetic_lightning`, `WeatherState`, and a `Battery`
//! component in `src/layer1/energy/mod.rs`. What if a `MagneticStorm` could cause a "Supercharged Anomaly"?
//!
//! # The Feature
//! When `WeatherType` is `MagneticStorm`, unroofed `Battery` entities have a small chance to randomly absorb massive
//! ambient energy. This directly boosts their `charge` beyond their standard capacity limit and emits a `GridOverloadEvent`
//! due to the sudden surge of raw power.
//!
//! # The Potential
//! This ties energy buffering with dangerous weather events, adding a mechanic where exposed batteries can act as
//! "lightning rods" to store immense free power but risk blowing up the grid.

use crate::layer1::core::map::GridPosition;
use crate::layer1::energy::{Battery, GridOverloadEvent};
use crate::layer1::nature::weather::{WeatherState, WeatherType};
use crate::layer1::physics::structural_integrity::RoofGrid;
use bevy_ecs::prelude::*;
use rand::Rng;

const ANOMALY_CHANCE: f64 = 0.05;

/// The system that supercharges exposed batteries during a Magnetic Storm.
pub fn supercharged_battery_system(
    weather_state: Option<Res<WeatherState>>,
    roof_grid: Option<Res<RoofGrid>>,
    mut batteries: Query<(Entity, &GridPosition, &mut Battery)>,
    mut overload_events: EventWriter<GridOverloadEvent>,
    mut log: Option<ResMut<crate::shared::log::MessageLog>>,
) {
    if let Some(state) = weather_state {
        if state.current_weather != WeatherType::MagneticStorm {
            return;
        }
    } else {
        return;
    }

    let mut rng = rand::thread_rng();
    let has_roof = |x: i32, y: i32| -> bool {
        if let Some(ref grid) = roof_grid {
            grid.has_roof(x, y)
        } else {
            false
        }
    };

    for (entity, pos, mut battery) in batteries.iter_mut() {
        if !has_roof(pos.x, pos.y) && rng.gen_bool(ANOMALY_CHANCE) {
            // Massive ambient energy absorbed
            battery.charge += 1000.0;
            overload_events.send(GridOverloadEvent { victim: entity });

            if let Some(ref mut l) = log {
                l.add_colored(
                    format!(
                        "A battery at ({}, {}) absorbed a massive anomaly during the storm!",
                        pos.x, pos.y
                    ),
                    ratatui::style::Color::Yellow,
                );
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(supercharged_battery_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_supercharged_battery_absorbs_energy() {
        let mut world = World::new();

        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 10,
        });

        let grid = RoofGrid::new(10, 10);
        // No roof at (5, 5)
        world.insert_resource(grid);
        world.init_resource::<Events<GridOverloadEvent>>();

        let battery = world
            .spawn((
                GridPosition { x: 5, y: 5 },
                Battery {
                    capacity: 100.0,
                    charge: 50.0,
                    max_throughput: 10.0,
                },
            ))
            .id();

        // Run multiple times to trigger the anomaly reliably
        for _ in 0..1000 {
            world.run_system_once(supercharged_battery_system).unwrap();
            world.flush();
        }

        let batt = world.get::<Battery>(battery).unwrap();
        assert!(
            batt.charge > 100.0,
            "Exposed battery should absorb energy beyond capacity"
        );

        let events = world.resource::<Events<GridOverloadEvent>>();
        let mut reader = events.get_cursor();
        assert!(
            reader.read(events).next().is_some(),
            "Grid overload event should be emitted"
        );
    }
}
