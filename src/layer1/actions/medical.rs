use crate::layer1::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::CapacityProxy;
use crate::layer1::utility_types::{
    ActionType, UtilityWeights, calculate_context_score,
};
use bevy_ecs::prelude::*;

/// Evaluates the utility of seeking medical care.
///
/// If health is low (e.g. < 90%), and there is a hospital available, return a score.
/// Score increases as health decreases.
#[must_use]
pub(crate) fn evaluate_seek_medical_care(
    pop_pos: GridPosition,
    _needs: &crate::layer1::needs::Needs,
    health: Health,
    weights: &UtilityWeights,
    hospitals: &[CapacityProxy],
) -> Option<(f32, Entity)> {
    if health.current >= health.max * 0.95 {
        return None;
    }

    let mut best: Option<(f32, Entity)> = None;

    // Urgency based on missing health
    // 50% health -> 0.5 missing -> urgency ?
    // Let's say max urgency 1.0 at 0 health.
    // urgency = (1.0 - health_pct) * scale
    let health_pct = health.current / health.max;
    let urgency = (1.0 - health_pct) * 2.0; // e.g. 50% health = 1.0 urgency. 10% health = 1.8 urgency.

    for hospital in hospitals {
        // Simple context score
        let context = calculate_context_score(
            pop_pos,
            Some(hospital.pos),
            hospital.capacity, // Use proxy capacity (likely 10)
            hospital.usage,    // Occupied (likely 0)
            weights,
        );

        let utility = urgency * context;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, hospital.entity));
        }
    }

    best
}
