//! Cabin Fever system (Spec 082).
//!
//! Simulates the psychological toll of confinement and overcrowding.
//! Pops who spend too long indoors or crowded accumulate stress, reducing morale.

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::structural_integrity::RoofGrid;
use crate::layer1::needs::Needs;

/// Tracks confinement and crowding stress for a Pop.
#[derive(Component, Debug, Clone, Default)]
pub struct CabinFever {
    /// Stress from being indoors (0.0 to 100.0).
    pub confinement: f32,
    /// Stress from being crowded by other pops (0.0 to 100.0).
    pub crowding: f32,
}

impl CabinFever {
    /// Returns the total stress value (clamped to 100.0).
    #[must_use]
    pub fn total_stress(&self) -> f32 {
        (self.confinement + self.crowding).min(100.0)
    }
}

/// Updates confinement and crowding stress based on environment.
pub fn update_cabin_fever_system(
    roof: Res<RoofGrid>,
    mut query: Query<(Entity, &GridPosition, &mut CabinFever)>,
    all_pops: Query<&GridPosition, With<crate::layer1::pop::Pop>>,
) {
    // 1. Confinement Logic
    for (_entity, pos, mut fever) in &mut query {
        if roof.has_roof(pos.x, pos.y) {
            fever.confinement = (fever.confinement + 0.1).min(100.0);
        } else {
            fever.confinement = (fever.confinement - 0.5).max(0.0);
        }

        // 2. Crowding Logic (Naive O(N^2) for MVP)
        let mut neighbors = 0;
        // Optimization: Checking against all pops is O(N*M).
        // Since we are iterating query (M pops) and inner loop all_pops (N pops), it is O(N^2).
        // For 100 pops, 10,000 checks. Feasible for MVP.
        for other_pos in &all_pops {
            // Check Chebyshev distance <= 1 (adjacent + diagonal)
            if (other_pos.x - pos.x).abs() <= 1 && (other_pos.y - pos.y).abs() <= 1 {
                neighbors += 1;
            }
        }

        // neighbors includes self, so > 3 means 3 neighbors (4 total in 3x3 area)
        if neighbors > 3 {
             fever.crowding = (fever.crowding + 0.2).min(100.0);
        } else {
             fever.crowding = (fever.crowding - 0.1).max(0.0);
        }
    }
}

/// Applies morale penalty based on cabin fever stress.
pub fn apply_cabin_fever_morale_system(
    mut query: Query<(&CabinFever, &mut Needs)>,
) {
    for (fever, mut needs) in &mut query {
        let penalty = fever.total_stress() * 0.001; // Scale down
        needs.leisure = (needs.leisure - penalty).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce; // Needed for run_system_once
    use crate::layer1::pop::Pop;
    use crate::layer1::unrest::MentalState;
    use crate::layer1::needs::Needs;
    use crate::layer1::map::GridPosition;
    use crate::layer1::structural_integrity::RoofGrid;
    use crate::layer1::cabin_fever::{CabinFever, update_cabin_fever_system, apply_cabin_fever_morale_system};

    fn setup_world() -> World {
        let mut world = World::new();
        crate::setup::init_task_pools();
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
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

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 }, // Indoors
            CabinFever::default(),
        )).id();

        // Run system for 10 ticks
        for _ in 0..10 {
             world.run_system_once(update_cabin_fever_system).unwrap();
        }

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(fever.confinement > 0.0);
    }

    #[test]
    fn test_confinement_relief() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 }, // Outdoors (no roof set)
            CabinFever {
                confinement: 50.0,
                ..Default::default()
            },
        )).id();

         world.run_system_once(update_cabin_fever_system).unwrap();

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(fever.confinement < 50.0);
    }

    #[test]
    fn test_crowding_accumulation() {
        let mut world = setup_world();

        // Spawn 5 pops at (5,5)
        let pops: Vec<Entity> = (0..5).map(|_| {
            world.spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                CabinFever::default(),
            )).id()
        }).collect();

        // Run system
        world.run_system_once(update_cabin_fever_system).unwrap();

        let fever = world.get::<CabinFever>(pops[0]).unwrap();
        assert!(fever.crowding > 0.0);
    }

    #[test]
    fn test_cabin_fever_affects_morale() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            Needs {
                leisure: 1.0, // Max morale initially
                ..Default::default()
            },
            CabinFever {
                confinement: 100.0, // High fever
                crowding: 0.0,
            },
        )).id();

        world.run_system_once(apply_cabin_fever_morale_system).unwrap();

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 1.0);
    }

    #[test]
    fn test_high_fever_triggers_break() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            Needs { leisure: 0.0, ..Default::default() }, // Already low
            MentalState::Normal,
            CabinFever {
                confinement: 100.0, // Maxed out
                crowding: 100.0,
            },
        )).id();

        // Run break check (from existing system, but enhanced)
        // Check if logic handles new break type or generic break
        world.run_system_once(crate::layer1::unrest::check_mental_break_system).unwrap();

        let state = world.get::<MentalState>(pop).unwrap();
        assert!(matches!(state, MentalState::Broken(_)));
        // Ideally specifically Aggressive or similar, but generic Broken is MVP
    }
}
