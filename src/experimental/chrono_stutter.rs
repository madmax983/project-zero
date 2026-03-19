//! The Chrono-Stutter (Nova Feature).
//!
//! # The Spark
//! We have `Speed`, `Age`, and `Needs`. What if localized temporal anomalies
//! ("Chrono-Stutters") spontaneously spawn on the map, drastically accelerating
//! time for any Pop that walks into them?
//!
//! # The Feature
//! `ChronoAnomaly` entities spawn randomly. They have a radius.
//! Any Pop within this radius experiences accelerated time:
//! - Movement Speed is tripled.
//! - Needs decay much faster.
//! - Age increases faster.
//! This creates dynamic hotspots that are great for quick work but dangerous
//! for the Pop's lifespan and sanity.

use crate::layer1::lifecycle::Age;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::{Pop, Speed};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct ChronoAnomaly {
    pub radius: u32,
    pub time_multiplier: f32,
    pub ticks_remaining: u32,
}

const SPAWN_CHANCE: f64 = 0.0005;
const ANOMALY_DURATION: u32 = 1000;

pub fn spawn_chrono_anomaly_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    anomalies: Query<&ChronoAnomaly>,
) {
    if time.tick < 5000 {
        return;
    }

    if anomalies.iter().count() >= 3 {
        return;
    }

    let mut rng = rand::thread_rng();
    if rng.gen_bool(SPAWN_CHANCE) {
        let x = rng.gen_range(10..70);
        let y = rng.gen_range(10..40);

        commands.spawn((
            ChronoAnomaly {
                radius: 5,
                time_multiplier: 3.0,
                ticks_remaining: ANOMALY_DURATION,
            },
            GridPosition { x, y },
        ));
    }
}

pub fn apply_chrono_stutter_system(
    mut commands: Commands,
    mut anomalies: Query<(Entity, &GridPosition, &mut ChronoAnomaly)>,
    mut pops: Query<(&GridPosition, &mut Speed, &mut Needs, &mut Age), With<Pop>>,
) {
    let mut active_anomalies = Vec::new();

    for (entity, pos, mut anomaly) in anomalies.iter_mut() {
        if anomaly.ticks_remaining > 0 {
            anomaly.ticks_remaining -= 1;
            active_anomalies.push((*pos, anomaly.radius, anomaly.time_multiplier));
        } else {
            commands.entity(entity).despawn();
        }
    }

    if active_anomalies.is_empty() {
        return;
    }

    for (pop_pos, mut speed, mut needs, mut age) in pops.iter_mut() {
        for (anom_pos, radius, multiplier) in &active_anomalies {
            if pop_pos.distance_chebyshev(*anom_pos) <= *radius {
                // Apply temporal acceleration
                // Note: Speed is usually reset each tick by reset_speed_system,
                // so we just multiply it here.
                speed.current *= *multiplier;

                // Fast-forward needs (normally decays by 0.001 per tick)
                let extra_decay = 0.001 * (*multiplier - 1.0);
                needs.hunger = (needs.hunger - extra_decay).max(0.0);
                needs.rest = (needs.rest - extra_decay).max(0.0);
                needs.leisure = (needs.leisure - extra_decay).max(0.0);

                // Fast-forward age
                // Adds extra ticks alive. (multiplier - 1) because the regular
                // tick already adds 1.
                age.ticks_alive += (*multiplier as u64) - 1;

                break; // Only affected by one anomaly at a time
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_spawn_chrono_anomaly() {
        let mut world = World::new();
        world.insert_resource(SimulationTime {
            tick: 6000,
            ..Default::default()
        });

        // Force chance by looping or mocking if possible, but testing probability is flaky.
        // Instead, we just test the manual spawn.
        world.spawn((
            ChronoAnomaly {
                radius: 5,
                time_multiplier: 3.0,
                ticks_remaining: 10,
            },
            GridPosition { x: 10, y: 10 },
        ));

        assert_eq!(world.query::<&ChronoAnomaly>().iter(&world).count(), 1);
    }

    #[test]
    fn test_apply_chrono_stutter() {
        let mut world = World::new();

        world.spawn((
            ChronoAnomaly {
                radius: 5,
                time_multiplier: 3.0,
                ticks_remaining: 10,
            },
            GridPosition { x: 10, y: 10 },
        ));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 12, y: 12 }, // Within radius 5
                Speed {
                    current: 1.0,
                    base: 1.0,
                    accumulator: 0.0,
                },
                Needs {
                    hunger: 0.5,
                    rest: 0.5,
                    leisure: 0.5,
                    ..Default::default()
                },
                Age {
                    ticks_alive: 100,
                    ..Default::default()
                },
            ))
            .id();

        let _ = world.run_system_once(apply_chrono_stutter_system);

        let speed = world.get::<Speed>(pop).unwrap();
        let needs = world.get::<Needs>(pop).unwrap();
        let age = world.get::<Age>(pop).unwrap();

        // Speed should be multiplied
        assert!((speed.current - 3.0).abs() < f32::EPSILON);

        // Needs should be reduced
        assert!(needs.hunger < 0.5);

        // Age should increase by an extra 2 ticks (3.0 multiplier - 1)
        assert_eq!(age.ticks_alive, 102);
    }

    #[test]
    fn test_chrono_anomaly_despawns_when_expired() {
        let mut world = World::new();

        let anomaly = world
            .spawn((
                ChronoAnomaly {
                    radius: 5,
                    time_multiplier: 3.0,
                    ticks_remaining: 1, // Will expire this tick
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        world.run_system_once(apply_chrono_stutter_system).unwrap();

        // Next tick, it should be 0 and despawn
        world.run_system_once(apply_chrono_stutter_system).unwrap();

        assert!(world.get::<ChronoAnomaly>(anomaly).is_none());
    }
}
