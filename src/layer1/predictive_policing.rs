//! Predictive Policing System (Spec 173).
//!
//! This module implements the "Algo-Hub" pre-crime mechanics, where Pops can be marked
//! as `Suspect`s based on high stress and volatile traits before they actually commit a crime.
//! Wardens can then arrest these suspects and place them in protective custody.

#![allow(clippy::collapsible_if)]
use crate::layer1::justice::Inmate;
use crate::layer1::map::GridPosition;
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_ai::{ActionType, PopAction};
use crate::layer1::utility_eval_types::{evaluate_candidates, ScorableCandidate};
use crate::layer1::utility_types::UtilityWeights;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use bevy_ecs::prelude::*;

/// Component marking a Pop as a suspect for pre-crime arrest.
#[derive(Component, Debug, Clone)]
pub struct Suspect {
    /// The probability of committing a crime or breakdown (0.0 to 1.0).
    pub probability: f32,
    /// The specific crime or breakdown predicted.
    pub predicted_crime: String,
}

/// Configuration for the Predictive Policing system.
#[derive(Resource, Default)]
pub struct PredictionConfig {
    /// Probability threshold to mark a Pop as a Suspect.
    pub threshold: f32,
    /// Master switch for the system.
    pub enabled: bool,
}

/// Marker component for the "Algo-Hub" building that enables this system.
#[derive(Component)]
pub struct PredictiveModel;

/// System to identify and mark potential criminals.
///
/// Scans Pops for high stress and risky traits. If the calculated risk exceeds
/// `config.threshold`, the Pop is marked as a [`Suspect`].
///
/// Requires an active [`PredictiveModel`] (Algo-Hub) to function.
pub fn check_prediction_system(
    mut commands: Commands,
    query: Query<(Entity, &StressTracker, Option<&Traits>), Without<Suspect>>,
    config: Res<PredictionConfig>,
    models: Query<Option<&crate::layer1::energy::PowerConsumer>, With<PredictiveModel>>,
) {
    if !config.enabled {
        return;
    }

    // Check for active predictive model (Algo-Hub)
    let has_active_model = models.iter().any(|pc| pc.is_none_or(|p| p.active));
    if !has_active_model {
        return;
    }

    for (entity, tracker, traits) in query.iter() {
        let mut risk = 0.0;

        // Stress factor: > 50 ticks is risky
        if tracker.accumulated_stress > 50.0 {
            risk += 0.5;
        }

        // Trait factor
        if let Some(t) = traits {
            if t.has(Trait::Volatile) {
                risk += 0.3;
            }
            if t.has(Trait::Pyromaniac) {
                risk += 0.4;
            }
        }

        if risk >= config.threshold {
            commands.entity(entity).insert(Suspect {
                probability: risk.min(1.0),
                predicted_crime: "Predicted Breakdown".to_string(),
            });
        }
    }
}

/// Evaluates the utility of arresting a specific Suspect.
///
/// Returns a tuple of (`utility_score`, `target_entity`) if a valid target is found.
///
/// # Arguments
///
/// * `warden_pos`: The position of the pop evaluating the action.
/// * `weights`: Utility weights of the pop.
/// * `suspects`: List of `ScorableCandidate`s representing [`Suspect`]s.
pub fn evaluate_pre_crime_arrest(
    warden_pos: &GridPosition,
    weights: &UtilityWeights,
    suspects: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    if suspects.is_empty() {
        return None;
    }

    // Score slightly lower than actual crime arrest (0.8 vs 0.9 for Warden)
    // But higher than idle.
    evaluate_candidates(*warden_pos, weights, suspects, 0.7)
}

/// Executes the arrest of a Suspect.
///
/// 1. Removes [`Suspect`] status.
/// 2. Adds [`Inmate`] status with a short sentence (Protective Custody).
/// 3. Resets [`StressTracker`].
/// 4. Teleports the pop to a Jail zone.
pub fn execute_pre_crime_arrest(world: &mut World, _warden: Entity, target: Entity) {
    // Check if target is still a suspect (avoid race condition)
    if world.get::<Suspect>(target).is_none() {
        return;
    }

    // Remove Suspect
    world.entity_mut(target).remove::<Suspect>();

    // Add Inmate with "Protective Custody" sentence (shorter than crime)
    world.entity_mut(target).insert(Inmate {
        sentence_ticks: 500, // Short sentence to cool down
    });

    // Reset stress
    if let Some(mut tracker) = world.get_mut::<StressTracker>(target) {
        tracker.accumulated_stress = 0.0;
    }

    // Teleport to jail
    if let Some(pos) = find_jail_spot(world) {
        if let Some(mut grid_pos) = world.get_mut::<GridPosition>(target) {
            *grid_pos = pos;
        }
    }

    // Pacify Criminal (Clear AI state so they don't walk away)
    world
        .entity_mut(target)
        .remove::<crate::layer1::execution::MovementTarget>()
        .remove::<crate::layer1::execution::AtTarget>()
        .remove::<crate::layer1::utility_types::StartPlan>();

    if let Some(mut action) = world.get_mut::<PopAction>(target) {
        action.current = ActionType::Idle;
        action.current_utility = 0.0;
        action.ticks_committed = 1000; // Commit to serving time
    }
}

/// System to execute pre-crime arrests when a warden reaches their target.
pub fn pre_crime_execution_system(world: &mut World) {
    let mut arrests = Vec::new();
    let mut query = world.query::<(
        Entity,
        &PopAction,
        &crate::layer1::execution::MovementTarget,
        Option<&crate::layer1::execution::AtTarget>,
    )>();
    for (entity, action, target, at_target) in query.iter(world) {
        if action.current == ActionType::PreCrimeArrest && at_target.is_some() {
            arrests.push((entity, target.target_entity));
        }
    }

    for (guard, suspect) in arrests {
        execute_pre_crime_arrest(world, guard, suspect);

        // Reset guard action
        if let Some(mut action) = world.get_mut::<PopAction>(guard) {
            action.current = ActionType::Idle;
            action.ticks_committed = 0;
        }
        world
            .entity_mut(guard)
            .remove::<crate::layer1::execution::MovementTarget>();
        world
            .entity_mut(guard)
            .remove::<crate::layer1::execution::AtTarget>();
    }
}

fn find_jail_spot(world: &World) -> Option<GridPosition> {
    if let Some(zone_grid) = world.get_resource::<ZoneGrid>() {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        for y in 0..zone_grid.height {
            for x in 0..zone_grid.width {
                if zone_grid.get(x as i32, y as i32) == ZoneType::Jail {
                    return Some(GridPosition {
                        x: x as i32,
                        y: y as i32,
                    });
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::layer1::justice::{Inmate, Wanted};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::predictive_policing::{
        check_prediction_system, evaluate_pre_crime_arrest, PredictionConfig, PredictiveModel,
        Suspect,
    };
    use crate::layer1::stress::StressTracker;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::utility_types::UtilityWeights;
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(PredictionConfig {
            threshold: 0.8, // 80% probability required
            enabled: true,
        });
        // Spawn a PredictiveModel so system runs
        world.spawn(PredictiveModel);
        world
    }

    // 1. Prediction Logic
    #[test]
    fn test_high_stress_volatile_pop_becomes_suspect() {
        let mut world = setup_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_prediction_system);

        // Volatile pop with high stress ticks
        let mut traits = Traits::default();
        traits.add(Trait::Volatile);

        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 80.0,
                }, // Near breakdown (threshold is 100)
                traits,
                // No Suspect component yet
            ))
            .id();

        schedule.run(&mut world);

        // Should be marked Suspect
        let suspect = world.get::<Suspect>(pop).expect("Should be marked Suspect");
        assert!(suspect.probability >= 0.8);
        assert_eq!(suspect.predicted_crime, "Predicted Breakdown");
    }

    #[test]
    fn test_low_stress_pop_is_safe() {
        let mut world = setup_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_prediction_system);

        let pop = world
            .spawn((
                Pop,
                StressTracker {
                    accumulated_stress: 10.0,
                },
                Traits::default(),
            ))
            .id();

        schedule.run(&mut world);

        assert!(world.get::<Suspect>(pop).is_none());
    }

    // 2. Warden Evaluation
    #[test]
    fn test_warden_targets_suspect() {
        let mut world = setup_world();

        let suspect = world
            .spawn((
                Pop,
                Suspect {
                    probability: 0.9,
                    predicted_crime: "Arson".to_string(),
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let warden_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Create proxy manually for test
        let suspects = vec![ScorableCandidate::new(suspect, GridPosition { x: 5, y: 5 })];

        // Evaluate action
        let result = evaluate_pre_crime_arrest(&warden_pos, &weights, &suspects);

        assert!(result.is_some());
        let (score, target) = result.expect("Missing result");
        assert!(score > 0.0);
        assert_eq!(target, suspect);
    }

    // 3. Arrest Execution
    #[test]
    fn test_pre_crime_arrest_converts_to_inmate() {
        let mut world = setup_world();

        // Need ZoneGrid
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));

        let suspect = world
            .spawn((
                Pop,
                Suspect {
                    probability: 0.95,
                    predicted_crime: "Murder".to_string(),
                },
                GridPosition { x: 1, y: 1 },
                StressTracker {
                    accumulated_stress: 90.0,
                },
            ))
            .id();

        let warden = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();

        // Execute arrest
        crate::layer1::predictive_policing::execute_pre_crime_arrest(&mut world, warden, suspect);

        // Should be Inmate (Protective Custody)
        let inmate = world.get::<Inmate>(suspect).expect("Should be Inmate");
        assert!(inmate.sentence_ticks > 0);

        // Should NOT be Wanted (they haven't done it yet)
        assert!(world.get::<Wanted>(suspect).is_none());

        // Suspect marker removed
        assert!(world.get::<Suspect>(suspect).is_none());

        // Stress should be reset
        let tracker = world
            .get::<StressTracker>(suspect)
            .expect("Should have StressTracker");
        assert_eq!(tracker.accumulated_stress, 0.0);
    }
}
