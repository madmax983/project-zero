use crate::layer1::building::ShiftSchedule;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::farm::Farm;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use crate::layer1::utility_ai::{calculate_context_score, calculate_success_modifier};
use bevy_ecs::prelude::*;

/// Evaluates the utility of farming.
///
/// Checks for farms with available capacity.
///
/// # Returns
///
/// `Some((utility, target_entity))` if a valid farm is found, `None` otherwise.
pub fn evaluate_farm<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    cycle: &DayNightCycle,
    farms: impl Iterator<
        Item = (
            Entity,
            &'a GridPosition,
            &'a Farm,
            Option<&'a ShiftSchedule>,
        ),
    >,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.5;

    for (entity, pos, farm, schedule) in farms {
        // Check shift schedule
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }

        // Check capacity
        // Note: farm.workers might be legacy/not fully synced, but we use it for capacity check as per spec/tests.
        if farm.workers.len() >= farm.capacity {
            continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            farm.capacity,
            farm.workers.len(),
            weights,
        );

        let success = calculate_success_modifier(ActionType::Farm, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, entity));
        }
    }
    best
}
