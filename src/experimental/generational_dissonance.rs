//! Generational Dissonance (Nova Feature).
//!
//! # The Spark
//! The colony evolves over time. What if older Pops who lived through the harsh
//! early days clash with younger, sheltered Pops born into relative comfort?
//!
//! # The Feature
//! A system that checks interactions between adjacent Pops. If their age difference
//! is large (e.g. 50 years), the older Pop gains stress and loses leisure, while
//! the younger Pop loses morale.

use crate::layer1::balance::TICKS_PER_YEAR;
use crate::layer1::lifecycle::Age;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use bevy_ecs::prelude::*;

const DISSONANCE_AGE_GAP_TICKS: u64 = 50 * TICKS_PER_YEAR;
const STRESS_PENALTY: f32 = 0.05;
const LEISURE_PENALTY: f32 = 0.01;
const MORALE_PENALTY: f32 = 0.02;

/// Applies generational friction between adjacent pops with a large age gap.
pub fn generational_dissonance_system(
    mut query: Query<
        (
            &GridPosition,
            &Age,
            &mut Needs,
            &mut Morale,
            &mut StressTracker,
        ),
        With<Pop>,
    >,
) {
    let mut combinations = query.iter_combinations_mut();

    while let Some(
        [(pos1, age1, mut needs1, mut morale1, mut stress1), (pos2, age2, mut needs2, mut morale2, mut stress2)],
    ) = combinations.fetch_next()
    {
        // Check if adjacent
        if pos1.distance_chebyshev(*pos2) <= 1 {
            // Check age difference
            let age_diff = if age1.ticks_alive > age2.ticks_alive {
                age1.ticks_alive - age2.ticks_alive
            } else {
                age2.ticks_alive - age1.ticks_alive
            };

            if age_diff > DISSONANCE_AGE_GAP_TICKS {
                // Determine who is older
                if age1.ticks_alive > age2.ticks_alive {
                    // Pop 1 is older
                    stress1.accumulated_stress =
                        (stress1.accumulated_stress + STRESS_PENALTY).min(100.0);
                    needs1.leisure = (needs1.leisure - LEISURE_PENALTY).max(0.0);

                    // Pop 2 is younger
                    morale2.value = (morale2.value - MORALE_PENALTY).max(0.0);
                } else {
                    // Pop 2 is older
                    stress2.accumulated_stress =
                        (stress2.accumulated_stress + STRESS_PENALTY).min(100.0);
                    needs2.leisure = (needs2.leisure - LEISURE_PENALTY).max(0.0);

                    // Pop 1 is younger
                    morale1.value = (morale1.value - MORALE_PENALTY).max(0.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(generational_dissonance_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        World::new()
    }

    #[test]
    fn test_generational_dissonance_applies_penalties() {
        let mut world = setup_world();

        let older_pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Age {
                    ticks_alive: 60 * TICKS_PER_YEAR,
                    ..Default::default()
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        let younger_pop = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 }, // Adjacent
                Age {
                    ticks_alive: 5 * TICKS_PER_YEAR,
                    ..Default::default()
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        world
            .run_system_once(generational_dissonance_system)
            .unwrap();

        let old_stress = world.get::<StressTracker>(older_pop).unwrap();
        let old_needs = world.get::<Needs>(older_pop).unwrap();

        let young_morale = world.get::<Morale>(younger_pop).unwrap();

        assert!(
            (old_stress.accumulated_stress - 10.05).abs() < f32::EPSILON,
            "Older pop should gain stress"
        );
        assert!(
            (old_needs.leisure - 0.49).abs() < f32::EPSILON,
            "Older pop should lose leisure"
        );
        assert!(
            (young_morale.value - 0.48).abs() < f32::EPSILON,
            "Younger pop should lose morale"
        );
    }

    #[test]
    fn test_generational_dissonance_ignores_small_age_gap() {
        let mut world = setup_world();

        let pop1 = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Age {
                    ticks_alive: 20 * TICKS_PER_YEAR,
                    ..Default::default()
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        let pop2 = world
            .spawn((
                Pop,
                GridPosition { x: 6, y: 5 }, // Adjacent
                Age {
                    ticks_alive: 25 * TICKS_PER_YEAR, // Only 5 years gap
                    ..Default::default()
                },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                Morale {
                    value: 0.5,
                    modifiers: vec![],
                },
                StressTracker {
                    accumulated_stress: 10.0,
                },
            ))
            .id();

        world
            .run_system_once(generational_dissonance_system)
            .unwrap();

        let stress1 = world.get::<StressTracker>(pop1).unwrap();
        let stress2 = world.get::<StressTracker>(pop2).unwrap();

        assert!(
            (stress1.accumulated_stress - 10.0).abs() < f32::EPSILON,
            "Stress should not change"
        );
        assert!(
            (stress2.accumulated_stress - 10.0).abs() < f32::EPSILON,
            "Stress should not change"
        );
    }
}
