use super::{AssignedTo, AssignmentType};
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_ai::{
    calculate_context_score, calculate_success_modifier, need_response_curve,
};
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

/// Evaluates the utility of satisfying rest at available housing.
#[must_use]
pub fn evaluate_satisfy_rest<'a>(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    housing: impl Iterator<Item = (Entity, &'a GridPosition, &'a Housing)>,
) -> Option<(f32, Entity)> {
    let rest_urgency = need_response_curve(needs.rest);

    let mut best: Option<(f32, Entity)> = None;

    for (housing_entity, housing_pos, house) in housing {
        let context_score = calculate_context_score(
            *pop_pos,
            Some(*housing_pos),
            house.capacity,
            house.residents.len(),
            weights,
        );

        let success_mod = calculate_success_modifier(ActionType::SatisfyRest, weights);

        let utility = rest_urgency * context_score * success_mod;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, housing_entity));
        }
    }

    best
}

/// Handles the arrival of a pop at housing to satisfy rest.
pub fn handle_arrival(
    pop_entity: Entity,
    target_entity: Entity,
    housing: &mut Query<&mut Housing>,
    commands: &mut Commands,
) {
    #[allow(clippy::collapsible_if)]
    if let Ok(mut house) = housing.get_mut(target_entity) {
        if house.residents.len() < house.capacity {
            house.residents.push(pop_entity);
            commands.entity(pop_entity).insert(AssignedTo {
                entity: target_entity,
                assignment_type: AssignmentType::HousingResident,
            });
        }
    }
}
