use super::{AssignedTo, AssignmentType};
use crate::layer1::farm::Farm;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use crate::layer1::utility_types::{
    calculate_context_score, calculate_success_modifier, need_response_curve,
};
use bevy_ecs::prelude::*;

/// Evaluates the utility of satisfying hunger at available farms.
#[must_use]
pub fn evaluate_satisfy_hunger<'a>(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    farms: impl Iterator<Item = (Entity, &'a GridPosition, &'a Farm)>,
) -> Option<(f32, Entity)> {
    let hunger_urgency = need_response_curve(needs.hunger);

    let mut best: Option<(f32, Entity)> = None;

    for (farm_entity, farm_pos, farm) in farms {
        let context_score = calculate_context_score(
            *pop_pos,
            Some(*farm_pos),
            farm.capacity,
            farm.workers.len(),
            weights,
        );

        let success_mod = calculate_success_modifier(ActionType::SatisfyHunger, weights);

        let utility = hunger_urgency * context_score * success_mod;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, farm_entity));
        }
    }

    best
}

/// Handles the arrival of a pop at a farm to satisfy hunger.
pub fn handle_arrival(
    pop_entity: Entity,
    target_entity: Entity,
    farms: &mut Query<&mut Farm>,
    commands: &mut Commands,
) {
    #[allow(clippy::collapsible_if)]
    if let Ok(mut farm) = farms.get_mut(target_entity) {
        if farm.workers.len() < farm.capacity {
            farm.workers.push(pop_entity);
            commands.entity(pop_entity).insert(AssignedTo {
                entity: target_entity,
                assignment_type: AssignmentType::FarmWorker,
            });
        }
    }
}
