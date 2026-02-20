use bevy_ecs::prelude::*;
use crate::layer1::stress::StressTracker;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::justice::Inmate;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::{ActionType, manhattan_distance, PopAction};
use crate::layer1::utility_eval_types::PositionProxy;
use crate::layer1::execution::{AtTarget, MovementTarget};
use crate::layer1::zone::{ZoneGrid, ZoneType};

#[derive(Component, Debug, Clone)]
pub struct Suspect {
    pub probability: f32, // 0.0 to 1.0
    pub predicted_crime: String,
}

#[derive(Resource, Default)]
pub struct PredictionConfig {
    pub threshold: f32,
    pub enabled: bool,
}

// Marker for the "Algo-Hub" building that enables this
#[derive(Component, Default)]
pub struct PredictiveModel;

pub fn check_prediction_system(
    mut commands: Commands,
    query: Query<(Entity, &StressTracker, Option<&Traits>), Without<Suspect>>,
    algo_hubs: Query<&PredictiveModel>, // Check if Algo-Hub exists
    config: Res<PredictionConfig>,
) {
    if !config.enabled { return; }

    // Check if at least one PredictiveModel exists
    if algo_hubs.iter().next().is_none() {
        return;
    }

    for (entity, tracker, traits) in query.iter() {
        let mut risk = 0.0;

        // Stress factor (0 to 1 based on accumulated stress / 100)
        // Assuming accumulated_stress is the raw value.
        // Spec says: "if tracker.ticks_at_low_morale > 50 { risk += 0.5; }"
        // Current StressTracker has `accumulated_stress: f32`.
        // Let's use accumulated_stress > 50.0 as the condition.
        if tracker.accumulated_stress > 50.0 {
            risk += 0.5;
        }

        // Trait factor
        if let Some(t) = traits {
            if t.0.contains(&Trait::Volatile) {
                risk += 0.3;
            }
            if t.0.contains(&Trait::Pyromaniac) {
                risk += 0.4;
            }
        }

        if risk >= config.threshold {
            commands.entity(entity).insert(Suspect {
                probability: risk.min(1.0),
                predicted_crime: "Breakdown: Violence".to_string(), // Default crime for now
            });
        }
    }
}

/// Evaluates the utility of performing a Pre-Crime Arrest action.
///
/// Uses the pre-populated suspects buffer for performance.
#[must_use]
pub fn evaluate_pre_crime_arrest(
    world: &mut World, // Kept for compatibility with test, but we'll try to use proxies if possible or just query if called from test
    warden_pos: &GridPosition,
) -> Option<(f32, Entity)> {
    // If called from utility_ai, we should ideally use proxies.
    // However, to satisfy the test signature which passes &World, I'll implement it using World query.
    // But for performance in the actual game loop, I should have a version that takes &[PositionProxy].

    // Let's implement the "slow" version for the test/spec signature,
    // and a "fast" version for the AI loop.
    // Wait, the spec defined `evaluate_pre_crime_arrest` taking `&World`.
    // But `utility_ai.rs` calls it.

    // I will implement `evaluate_pre_crime_arrest_internal` taking proxies,
    // and `evaluate_pre_crime_arrest` taking World which delegates or does query.

    // Actually, let's look at `justice::evaluate_warden_action`. It takes `criminals: &[PositionProxy]`.
    // I should follow that pattern for the AI loop.

    // The test in the spec calls `evaluate_pre_crime_arrest(&world, &warden_pos)`.
    // I will implement that signature for the test, but I will ALSO implement `evaluate_pre_crime_arrest_proxies` for the system.
    // OR, I can change the test to use the proxy version if I want to be strict.

    // But the instructions say "Write these tests in ...". I should respect the test code if possible.
    // So I will provide `evaluate_pre_crime_arrest` (slow, for tests) and `evaluate_pre_crime_arrest_proxies` (fast, for AI).

    let mut best_target = None;
    let mut min_dist = i32::MAX;

    let mut query = world.query::<(Entity, &GridPosition, &Suspect)>();
    for (entity, pos, _suspect) in query.iter(world) {
        let dist = manhattan_distance(warden_pos, pos);
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(entity);
        }
    }

    if let Some(target) = best_target {
        // Score slightly lower than actual crime arrest (0.8 vs 0.9)
        return Some((0.7, target));
    }
    None
}

/// Optimized evaluation for Utility AI system using proxies.
#[must_use]
pub fn evaluate_pre_crime_arrest_proxies(
    warden_pos: &GridPosition,
    suspects: &[PositionProxy],
    zone_grid: &ZoneGrid,
) -> Option<(f32, Entity)> {
    let mut best_target = None;
    let mut min_dist = i32::MAX;

    for suspect in suspects {
        // Don't arrest if already in sanctuary? (Mirroring justice system logic)
        if zone_grid.get(suspect.pos.x, suspect.pos.y) == ZoneType::Sanctuary {
            continue;
        }

        let dist = manhattan_distance(warden_pos, &suspect.pos);
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(suspect.entity);
        }
    }

    if let Some(target) = best_target {
        return Some((0.7, target));
    }
    None
}

pub fn execute_pre_crime_arrest(
    world: &mut World,
    _warden: Entity,
    target: Entity,
) {
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

    // Teleport to Jail (Reuse logic from justice.rs or duplicate for MVP)
    // Duplicating for MVP to avoid pub(crate) issues or complex dependencies
    let jail_pos = find_jail_spot(world);
    if let Some(pos) = jail_pos {
        if let Some(mut grid_pos) = world.get_mut::<GridPosition>(target) {
            *grid_pos = pos;
        }
    }

    // Pacify Criminal (Clear AI state)
    world
        .entity_mut(target)
        .remove::<MovementTarget>()
        .remove::<AtTarget>()
        .remove::<crate::layer1::utility_types::StartPlan>();

    if let Some(mut action) = world.get_mut::<PopAction>(target) {
        action.current = ActionType::Idle;
        action.current_utility = 0.0;
        action.ticks_committed = 500; // Commit to serving time
    }
}

fn find_jail_spot(world: &World) -> Option<GridPosition> {
    let zone_grid = world.resource::<ZoneGrid>();
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
    None
}

pub fn pre_crime_arrest_execution_system(world: &mut World) {
    // Collect arrests
    let mut arrests = Vec::new();

    let mut query = world.query::<(Entity, &PopAction, &MovementTarget, Option<&AtTarget>)>();
    for (entity, action, target, at_target) in query.iter(world) {
        if action.current == ActionType::PreCrimeArrest && at_target.is_some() {
            arrests.push((entity, target.target_entity));
        }
    }

    // Execute arrests
    for (warden, suspect) in arrests {
        execute_pre_crime_arrest(world, warden, suspect);

        // Reset warden action
        if let Some(mut action) = world.get_mut::<PopAction>(warden) {
            action.current = ActionType::Idle;
            action.ticks_committed = 0;
        }
        world.entity_mut(warden).remove::<MovementTarget>();
        world.entity_mut(warden).remove::<AtTarget>();
    }
}
