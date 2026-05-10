//! Cabin fever mechanics simulating the psychological toll of confinement and overcrowding.
//!
//! Pops who spend too long indoors without seeing the sky or who are constantly surrounded by others
//! will accumulate "Cabin Fever" stress. This reduces Morale and can trigger aggressive mental breaks.

use crate::layer1::beauty::BeautyGrid;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::structural_integrity::RoofGrid;
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Default)]
/// Tracks confinement and crowding stress for a Pop.
pub struct CabinFever {
    /// Stress from being indoors (0.0 to 100.0).
    pub confinement: f32,
    /// Stress from being crowded (0.0 to 100.0).
    pub crowding: f32,
}

impl CabinFever {
    /// Returns the total stress level, capped at 100.0.
    #[must_use]
    pub fn total_stress(&self) -> f32 {
        (self.confinement + self.crowding).min(100.0)
    }
}

/// System to update cabin fever levels based on environment.
pub fn update_cabin_fever_system(
    roof: Res<RoofGrid>,
    beauty: Option<Res<BeautyGrid>>,
    mut query: Query<(Entity, &GridPosition, &mut CabinFever)>,
    all_pops: Query<&GridPosition, With<crate::layer1::pop::Pop>>,
) {
    // 1. Confinement Logic
    for (_entity, pos, mut fever) in &mut query {
        if roof.has_roof(pos.x, pos.y) {
            // Check for beauty mitigation (high quality room proxy)
            let is_beautiful = beauty.as_ref().is_some_and(|b| {
                if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                    b.get(x, y) > 10.0 // Threshold for "nice" environment
                } else {
                    false
                }
            });

            if !is_beautiful {
                fever.confinement = (fever.confinement + 0.1).min(100.0);
            }
        } else {
            fever.confinement = (fever.confinement - 0.5).max(0.0);
        }

        // 2. Crowding Logic (Naive O(N*M) for MVP Green)
        let mut neighbors = 0;
        for other_pos in &all_pops {
            if other_pos.x.abs_diff(pos.x) <= 1 && other_pos.y.abs_diff(pos.y) <= 1 {
                neighbors += 1;
            }
        }

        // neighbors includes self, so > 3 means 2 others adjacent
        if neighbors > 3 {
            fever.crowding = (fever.crowding + 0.2).min(100.0);
        } else {
            fever.crowding = (fever.crowding - 0.1).max(0.0);
        }
    }
}

/// System to apply morale penalties based on cabin fever stress.
pub fn apply_cabin_fever_morale_system(mut query: Query<(&CabinFever, &mut Needs)>) {
    for (fever, mut needs) in &mut query {
        let penalty = fever.total_stress() * 0.001; // Scale down
        needs.leisure = (needs.leisure - penalty).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::cabin_fever::{
        apply_cabin_fever_morale_system, update_cabin_fever_system, CabinFever,
    };
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::structural_integrity::RoofGrid;
    use crate::layer1::unrest::MentalState;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        crate::setup::init_task_pools();
        world.insert_resource(crate::layer1::utility_types::UtilityConfig::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        // Setup 10x10 map
        let mut roof = RoofGrid::new(10, 10);
        // (5,5) is indoors
        roof.set(5, 5, true);
        world.insert_resource(roof);
        world
    }

    #[test]
    fn test_confinement_accumulation() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 }, // Indoors
                CabinFever::default(),
            ))
            .id();

        // Run system for 10 ticks
        for _ in 0..10 {
            let _ = world.run_system_once(update_cabin_fever_system);
        }

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(fever.confinement > 0.0);
    }

    #[test]
    fn test_confinement_relief() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 }, // Outdoors (no roof set)
                CabinFever {
                    confinement: 50.0,
                    ..Default::default()
                },
            ))
            .id();

        let _ = world.run_system_once(update_cabin_fever_system);

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(fever.confinement < 50.0);
    }

    #[test]
    fn test_crowding_accumulation() {
        let mut world = setup_world();

        // Spawn 5 pops at (5,5)
        let pops: Vec<Entity> = (0..5)
            .map(|_| {
                world
                    .spawn((Pop, GridPosition { x: 5, y: 5 }, CabinFever::default()))
                    .id()
            })
            .collect();

        // Run system
        // Note: Crowding check might need spatial index in real impl, or simple O(N^2) for tests
        let _ = world.run_system_once(update_cabin_fever_system);

        let fever = world.get::<CabinFever>(pops[0]).unwrap();
        assert!(fever.crowding > 0.0);
    }

    #[test]
    fn test_cabin_fever_affects_morale() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 1.0, // Max morale initially
                    ..Default::default()
                },
                CabinFever {
                    confinement: 100.0, // High fever
                    crowding: 0.0,
                },
            ))
            .id();

        let _ = world.run_system_once(apply_cabin_fever_morale_system);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 1.0);
    }

    #[test]
    fn test_high_fever_triggers_break() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                Needs {
                    leisure: 0.0,
                    ..Default::default()
                }, // Already low
                MentalState::Normal,
                CabinFever {
                    confinement: 100.0, // Maxed out
                    crowding: 100.0,
                },
            ))
            .id();

        // Run break check (from existing system, but enhanced)
        // Check if logic handles new break type or generic break

        let _ = world.run_system_once(crate::layer1::unrest::check_mental_break_system);

        let state = world.get::<MentalState>(pop).unwrap();
        assert!(matches!(state, MentalState::Broken(_)));
        // Ideally specifically Aggressive or similar, but generic Broken is MVP
    }

    #[test]
    fn test_high_beauty_negates_confinement() {
        let mut world = setup_world();
        world.insert_resource(crate::layer1::beauty::BeautyGrid::new(10, 10));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 }, // Indoors
                CabinFever::default(),
            ))
            .id();

        // Set high beauty at (5,5)
        let mut beauty = world.resource_mut::<crate::layer1::beauty::BeautyGrid>();
        beauty.set(5, 5, 20.0);

        // Run system for 10 ticks
        for _ in 0..10 {
            let _ = world.run_system_once(update_cabin_fever_system);
        }

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(
            fever.confinement.abs() < f32::EPSILON,
            "High beauty should prevent confinement gain"
        );
    }

    #[test]
    fn test_crowding_relief() {
        let mut world = setup_world();

        // Spawn 1 pop with high crowding
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                CabinFever {
                    crowding: 50.0,
                    ..Default::default()
                },
            ))
            .id();

        // Run system
        // Pop is alone (neighbors = 1), so crowding should decrease
        let _ = world.run_system_once(update_cabin_fever_system);

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(
            fever.crowding < 50.0,
            "Crowding should decrease when alone (was {}, now {})",
            50.0,
            fever.crowding
        );
    }

    #[test]
    fn test_crowding_exact_threshold() {
        let mut world = setup_world();

        // Spawn 3 pops (including self) at (5,5)
        // Threshold is > 3 neighbors.
        // Neighbors logic counts self.
        // If we spawn 3 pops, each sees 3 neighbors (including self).
        // 3 is NOT > 3, so crowding should decrease.

        let pops: Vec<Entity> = (0..3)
            .map(|_| {
                world
                    .spawn((
                        Pop,
                        GridPosition { x: 5, y: 5 },
                        CabinFever {
                            crowding: 10.0,
                            ..Default::default()
                        },
                    ))
                    .id()
            })
            .collect();

        let _ = world.run_system_once(update_cabin_fever_system);

        let fever = world.get::<CabinFever>(pops[0]).unwrap();
        assert!(
            fever.crowding < 10.0,
            "Crowding should decrease with exactly 3 neighbors (was {}, now {})",
            10.0,
            fever.crowding
        );

        // Now spawn 1 more to make it 4
        world.spawn((Pop, GridPosition { x: 5, y: 5 }, CabinFever::default()));

        // Run again
        let _ = world.run_system_once(update_cabin_fever_system);

        // Reset manual value to verify increase logic
        world
            .entity_mut(pops[0])
            .get_mut::<CabinFever>()
            .unwrap()
            .crowding = 10.0;

        let _ = world.run_system_once(update_cabin_fever_system);
        let fever = world.get::<CabinFever>(pops[0]).unwrap();

        assert!(
            fever.crowding > 10.0,
            "Crowding should increase with 4 neighbors"
        );
    }

    #[test]
    fn test_total_stress_cap() {
        let fever = CabinFever {
            confinement: 60.0,
            crowding: 60.0,
        };
        assert!((fever.total_stress() - 100.0).abs() < f32::EPSILON);

        let fever_low = CabinFever {
            confinement: 10.0,
            crowding: 10.0,
        };
        assert!((fever_low.total_stress() - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_out_of_bounds_confinement_relief() {
        let mut world = setup_world();

        // Spawn pop at negative coordinates (Outdoors/Void)
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: -10, y: -10 },
                CabinFever {
                    confinement: 50.0,
                    ..Default::default()
                },
            ))
            .id();

        let _ = world.run_system_once(update_cabin_fever_system);

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(
            fever.confinement < 50.0,
            "Out of bounds should relieve confinement"
        );
    }

    #[test]
    fn test_beauty_grid_boundary_check() {
        let mut world = setup_world();
        world.insert_resource(crate::layer1::beauty::BeautyGrid::new(10, 10));

        // Spawn pop at exactly map width (index out of bounds for array)
        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 10 },
                // Note: RoofGrid check uses (x < width). 10 is not < 10.
                // So has_roof returns false.
                // Thus confinement relieves.
                // But we want to test if beauty grid access panics if we force code path?
                // Hard to force code path without roof.
                // But let's just ensure system runs without panic.
                CabinFever::default(),
            ))
            .id();

        // To force beauty check, we need `roof.has_roof` to be true.
        // RoofGrid::set checks bounds, so we can't set roof at (10,10).
        // So we can't easily trigger the beauty check path for out of bounds.
        // However, checking that valid coords work is done in other tests.
        // Let's just ensure NO PANIC at edge cases.

        let _ = world.run_system_once(update_cabin_fever_system);

        // Pass if no panic
    }
}
