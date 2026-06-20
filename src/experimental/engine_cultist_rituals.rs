#![allow(clippy::type_complexity)]
//! Engine Cultist Rituals (Nova Feature).
//!
//! # The Spark
//! We have Pops with `Trait::EngineCultist` and `BuildingType::Generator`.
//! Let's connect them! Pops who worship the machine should feel calm and
//! rejuvenated near it.
//!
//! # The Feature
//! A system `engine_cultist_rituals_system` that checks if a Pop has
//! `Trait::EngineCultist`. If they are within 3 tiles of any `Generator`,
//! their `accumulated_stress` rapidly depletes, effectively acting as
//! a localized form of extreme relaxation/leisure.

use crate::layer1::architecture::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::psychology::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

const CULTIST_STRESS_RELIEF: f32 = 0.5;
const RITUAL_RADIUS: i32 = 3;

pub fn engine_cultist_rituals_system(
    mut pops: Query<(&GridPosition, &Traits, &mut StressTracker), With<Pop>>,
    buildings: Query<(&GridPosition, &Building)>,
) {
    // Collect generator positions
    let mut generators = Vec::new();
    for (pos, building) in buildings.iter() {
        if building.building_type == BuildingType::Generator {
            generators.push(*pos);
        }
    }

    if generators.is_empty() {
        return;
    }

    for (pos, traits, mut stress) in pops.iter_mut() {
        if traits.has(Trait::EngineCultist) {
            // Check distance to nearest generator
            let mut near_generator = false;
            for gen_pos in &generators {
                let dx = (pos.x - gen_pos.x).abs();
                let dy = (pos.y - gen_pos.y).abs();
                if dx <= RITUAL_RADIUS && dy <= RITUAL_RADIUS {
                    near_generator = true;
                    break;
                }
            }

            if near_generator && stress.accumulated_stress > 0.0 {
                stress.accumulated_stress -= CULTIST_STRESS_RELIEF;
                if stress.accumulated_stress < 0.0 {
                    stress.accumulated_stress = 0.0;
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(engine_cultist_rituals_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_engine_cultist_relieves_stress_near_generator() {
        let mut world = World::new();

        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Generator,

            },
        ));

        let mut traits = Traits::default();
        traits.add(Trait::EngineCultist);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 }, // Adjacent
                traits,
                StressTracker {
                    accumulated_stress: 5.0,
                },
            ))
            .id();

        world
            .run_system_once(engine_cultist_rituals_system)
            .unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress < 5.0);
    }

    #[test]
    fn test_engine_cultist_no_relief_if_far() {
        let mut world = World::new();

        world.spawn((
            GridPosition { x: 5, y: 5 },
            Building {
                building_type: BuildingType::Generator,

            },
        ));

        let mut traits = Traits::default();
        traits.add(Trait::EngineCultist);

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 20, y: 20 }, // Far away
                traits,
                StressTracker {
                    accumulated_stress: 5.0,
                },
            ))
            .id();

        world
            .run_system_once(engine_cultist_rituals_system)
            .unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!((stress.accumulated_stress - 5.0).abs() < f32::EPSILON);
    }
}
