//! The Nostalgia Contagion (Nova Feature).
//!
//! # The Spark
//! What if a pop's yearning for the past became a literal, contagious disease?
//! The `Homesick` trait lowers their work speed massively as they refuse to work on new tasks.
//! But at the same time, it is intensely comforting.
//!
//! # The Feature
//! Pops with the `Homesick` trait will periodically talk to nearby pops about Earth/home.
//! This has a chance to spread the `Homesick` trait to the listener.
//! At the same time, any pop with `Homesick` gets a massive `+0.5` Morale boost labeled "Nostalgia".
//!
//! # Tension
//! Perfect morale vs complete stagnation of progress.

use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;
use rand::Rng;

const CONTAGION_RADIUS: u32 = 3;
const CONTAGION_CHANCE: f32 = 0.05; // 5% chance per tick to spread to a nearby pop
const MORALE_BOOST: f32 = 0.5;

/// System to spread Nostalgia and apply its morale effects.
pub fn nostalgia_contagion_system(
    mut pops: Query<(Entity, &GridPosition, &mut Traits, &mut Morale), With<Pop>>,
) {
    let mut to_infect = Vec::new();
    let mut rng = rand::thread_rng();

    // First, find all infected positions and apply the morale boost
    let mut infected_positions = Vec::new();

    for (_entity, pos, traits, mut morale) in pops.iter_mut() {
        if traits.has(Trait::Homesick) {
            infected_positions.push(*pos);

            // Apply morale boost if not already present
            let has_boost = morale.modifiers.iter().any(|m| m.label == "Nostalgia");
            if !has_boost {
                morale.add_modifier(MoodModifier {
                    label: "Nostalgia".to_string(),
                    value: MORALE_BOOST,
                    duration: 100, // Reapplied so long as they have the trait
                });
            } else {
                // Refresh the duration
                if let Some(modifier) = morale.modifiers.iter_mut().find(|m| m.label == "Nostalgia")
                {
                    modifier.duration = 100;
                }
            }
        }
    }

    // Now, attempt to spread to uninfected pops
    for (entity, pos, traits, _morale) in pops.iter() {
        if !traits.has(Trait::Homesick) {
            let mut near_infected = false;
            for infected_pos in &infected_positions {
                if pos.distance_chebyshev(*infected_pos) <= CONTAGION_RADIUS {
                    near_infected = true;
                    break;
                }
            }

            if near_infected && rng.gen_bool(CONTAGION_CHANCE as f64) {
                to_infect.push(entity);
            }
        }
    }

    // Apply new infections
    for entity in to_infect {
        if let Ok((_, _, mut traits, _)) = pops.get_mut(entity) {
            traits.add(Trait::Homesick);
            // Optionally log or send an event
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(nostalgia_contagion_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_nostalgia_applies_morale_boost() {
        let mut world = World::new();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                {
                    let mut t = Traits::default();
                    t.add(Trait::Homesick);
                    t
                },
                Morale::default(),
            ))
            .id();

        world.run_system_once(nostalgia_contagion_system).unwrap();

        let morale = world.get::<Morale>(pop).unwrap();
        assert!(morale.modifiers.iter().any(|m| m.label == "Nostalgia"));
        let modifier = morale
            .modifiers
            .iter()
            .find(|m| m.label == "Nostalgia")
            .unwrap();
        assert_eq!(modifier.value, MORALE_BOOST);
    }

    #[test]
    fn test_nostalgia_spreads() {
        let mut world = World::new();

        // One infected pop
        world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            {
                let mut t = Traits::default();
                t.add(Trait::Homesick);
                t
            },
            Morale::default(),
        ));

        // One uninfected pop nearby
        let target = world
            .spawn((
                Pop,
                GridPosition { x: 1, y: 0 },
                Traits::default(),
                Morale::default(),
            ))
            .id();

        // Run enough times to practically guarantee the RNG hits (0.05 chance -> 0.95^1000 ≈ 5e-23)
        for _ in 0..1000 {
            world.run_system_once(nostalgia_contagion_system).unwrap();
        }

        let target_traits = world.get::<Traits>(target).unwrap();
        assert!(target_traits.has(Trait::Homesick));
    }

    #[test]
    fn test_nostalgia_does_not_spread_out_of_range() {
        let mut world = World::new();

        // One infected pop
        world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            {
                let mut t = Traits::default();
                t.add(Trait::Homesick);
                t
            },
            Morale::default(),
        ));

        // One uninfected pop far away
        let target = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                Traits::default(),
                Morale::default(),
            ))
            .id();

        // Run many times
        for _ in 0..100 {
            world.run_system_once(nostalgia_contagion_system).unwrap();
        }

        let target_traits = world.get::<Traits>(target).unwrap();
        assert!(!target_traits.has(Trait::Homesick));
    }
}
