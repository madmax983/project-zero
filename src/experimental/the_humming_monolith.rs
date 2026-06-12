//! The Humming Monolith (Nova Feature).
//!
//! # The Spark
//! The colony can become quite stable and predictable. What if there was a rare, mysterious
//! event that spawned a "Monolith" which slowly alters the minds of Pops who approach it?
//!
//! # The Feature
//! A `HummingMonolith` randomly spawns on the map after a certain amount of time.
//! It acts as a powerful light source. When Pops get too close, it randomly alters their
//! `Traits` (making them `VoidTouched` or `Synesthete`), maxes out their `leisure` (they
//! find it mesmerizing), and drastically slows their `Speed` (they stop to stare).
//! It connects the environment (`LightMap`) with Pop psychology (`Traits`, `Needs`, `Speed`).

use crate::layer1::lighting::LightSource;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::{Pop, Speed};
use crate::layer1::traits::{Trait, Traits};
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Component indicating an entity is the Humming Monolith.
#[derive(Component)]
pub struct HummingMonolith;

const SPAWN_TICK_THRESHOLD: u64 = 10_000;
const SPAWN_CHANCE_PER_TICK: f64 = 0.0001;
const MONOLITH_INFLUENCE_RADIUS: u32 = 8;
const MAX_MONOLITHS: usize = 1;

/// System to randomly spawn a Humming Monolith.
pub fn detect_and_spawn_monolith_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
    monoliths: Query<&HummingMonolith>,
) {
    if time.tick < SPAWN_TICK_THRESHOLD {
        return;
    }

    if monoliths.iter().count() >= MAX_MONOLITHS {
        return;
    }

    let mut rng = rand::thread_rng();
    if rng.gen_bool(SPAWN_CHANCE_PER_TICK) {
        // Spawn near the center conceptually, or just at a random location.
        // For simplicity, spawn at a random coordinate.
        let x = rng.gen_range(10..70);
        let y = rng.gen_range(10..40);

        commands.spawn((
            HummingMonolith,
            GridPosition { x, y },
            LightSource {
                radius: 15.0,
                intensity: 1.5,
                color: (150, 0, 255), // Mysterious purple glow
                is_outdoor: true,
            },
        ));
    }
}

/// System to apply the Monolith's influence to nearby Pops.
pub fn monolith_influence_system(
    monoliths: Query<&GridPosition, With<HummingMonolith>>,
    mut pops: Query<(&GridPosition, &mut Needs, &mut Speed, &mut Traits), With<Pop>>,
) {
    if monoliths.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();

    for monolith_pos in monoliths.iter() {
        for (pop_pos, mut needs, mut speed, mut traits) in pops.iter_mut() {
            if pop_pos.distance_chebyshev(*monolith_pos) <= MONOLITH_INFLUENCE_RADIUS {
                // Mesmerized: Max out leisure
                needs.leisure = 1.0;

                // Transfixed: Tank movement speed
                speed.current = (speed.current * 0.5).max(0.1);

                // Rare chance to alter traits
                if rng.gen_bool(0.01) {
                    if rng.gen_bool(0.5) {
                        traits.add(Trait::VoidTouched);
                    } else {
                        traits.add(Trait::Synesthete);
                    }
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems((detect_and_spawn_monolith_system, monolith_influence_system));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_spawn_monolith_respects_threshold_and_limit() {
        let mut world = World::new();

        // Under threshold
        world.insert_resource(SimulationTime {
            tick: 5000,
            ..Default::default()
        });
        world
            .run_system_once(detect_and_spawn_monolith_system)
            .unwrap();
        assert_eq!(world.query::<&HummingMonolith>().iter(&world).count(), 0);

        // Over limit
        world.insert_resource(SimulationTime {
            tick: 15000,
            ..Default::default()
        });
        world.spawn((HummingMonolith, GridPosition { x: 0, y: 0 }));
        world
            .run_system_once(detect_and_spawn_monolith_system)
            .unwrap();
        assert_eq!(world.query::<&HummingMonolith>().iter(&world).count(), 1); // Still 1
    }

    #[test]
    fn test_monolith_influence() {
        let mut world = World::new();

        world.spawn((HummingMonolith, GridPosition { x: 10, y: 10 }));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 12, y: 12 }, // Within radius 8
                Needs {
                    leisure: 0.1,
                    ..Default::default()
                },
                Speed {
                    current: 1.0,
                    ..Default::default()
                },
                Traits::default(),
            ))
            .id();

        world.run_system_once(monolith_influence_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        let speed = world.get::<Speed>(pop).unwrap();

        assert!((needs.leisure - 1.0).abs() < f32::EPSILON);
        assert!(speed.current < 1.0);
    }
}
