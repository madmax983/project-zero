//! Synesthesia Simulation (Nova Feature).
//!
//! Pops with the `Synesthete` trait experience colors as emotional sounds.

use crate::layer1::lighting::LightSource;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use bevy_ecs::prelude::*;

pub fn synesthesia_system(
    mut pops: Query<(&GridPosition, &mut Needs, &mut StressTracker, &Traits), With<Pop>>,
    lights: Query<(&GridPosition, &LightSource)>,
) {
    for (pop_pos, mut needs, mut stress, traits) in &mut pops {
        // Only affect synesthetes
        if !traits.0.contains(&Trait::Synesthete) {
            continue;
        }

        // Check nearby lights
        for (light_pos, light) in &lights {
            #[allow(clippy::cast_possible_truncation)]
            let radius = light.radius.ceil() as i32;

            if pop_pos.distance_chebyshev(*light_pos) <= radius.try_into().unwrap_or(0) {
                // Determine color
                let (r, g, b) = light.color;

                // Simple heuristic:
                // Mostly red -> irritant/stress (like an alarm klaxon)
                if r > 200 && g < 100 && b < 100 {
                    stress.accumulated_stress = (stress.accumulated_stress + 0.1).min(100.0);
                }

                // Mostly blue -> calm/leisure (like a soothing hum)
                if b > 200 && r < 100 && g < 100 {
                    needs.leisure = (needs.leisure + 0.005).min(1.0);
                }
            }
        }
    }
}

pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(synesthesia_system);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;
    use std::collections::HashSet;

    #[test]
    fn test_synesthesia_system_red_light() {
        let mut world = World::new();

        let traits = Traits(HashSet::from([Trait::Synesthete]));
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                StressTracker {
                    accumulated_stress: 0.0,
                },
                traits,
            ))
            .id();

        world.spawn((
            LightSource {
                is_outdoor: false,
                radius: 5.0,
                intensity: 1.0,
                color: (255, 0, 0), // Red
            },
            GridPosition { x: 0, y: 0 },
        ));

        world.run_system_once(synesthesia_system).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress > 0.0);
    }

    #[test]
    fn test_synesthesia_system_blue_light() {
        let mut world = World::new();

        let traits = Traits(HashSet::from([Trait::Synesthete]));
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.5,
                    ..Default::default()
                },
                StressTracker {
                    accumulated_stress: 0.0,
                },
                traits,
            ))
            .id();

        world.spawn((
            LightSource {
                is_outdoor: false,
                radius: 5.0,
                intensity: 1.0,
                color: (0, 0, 255), // Blue
            },
            GridPosition { x: 0, y: 0 },
        ));

        world.run_system_once(synesthesia_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.5);
    }

    #[test]
    fn test_synesthesia_system_ignores_non_synesthetes() {
        let mut world = World::new();

        let traits = Traits(HashSet::new());
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                StressTracker {
                    accumulated_stress: 0.0,
                },
                traits,
            ))
            .id();

        world.spawn((
            LightSource {
                is_outdoor: false,
                radius: 5.0,
                intensity: 1.0,
                color: (255, 0, 0), // Red
            },
            GridPosition { x: 0, y: 0 },
        ));

        world.run_system_once(synesthesia_system).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        // Strict clippy settings prohibit direct float comparisons
        assert!((stress.accumulated_stress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_synesthesia_system_out_of_range() {
        let mut world = World::new();

        let traits = Traits(HashSet::from([Trait::Synesthete]));
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                StressTracker {
                    accumulated_stress: 0.0,
                },
                traits,
            ))
            .id();

        world.spawn((
            LightSource {
                is_outdoor: false,
                radius: 1.0,
                intensity: 1.0,
                color: (255, 0, 0), // Red
            },
            GridPosition { x: 10, y: 10 },
        ));

        world.run_system_once(synesthesia_system).unwrap();

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!((stress.accumulated_stress - 0.0).abs() < f32::EPSILON);
    }
}
