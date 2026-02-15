use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_types::{ActionType, HousingProxy, UtilityWeights};
use crate::layer1::utility_types::{calculate_context_score, calculate_success_modifier};
use bevy_ecs::prelude::*;

/// Evaluates the utility of resting at a house.
///
/// Housing restores rest faster than sleeping on the ground.
///
/// # Returns
///
/// `Some((utility, target_entity))` if a valid house is found, `None` otherwise.
#[must_use]
pub fn evaluate_satisfy_rest(
    pop_pos: &GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    housing: &[HousingProxy],
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.5;

    // Urgency based on rest level
    let urgency = 1.0 - needs.rest;

    for house in housing {
        let context = calculate_context_score(
            *pop_pos,
            Some(house.pos),
            house.capacity,
            house.occupants,
            weights,
        );

        let success = calculate_success_modifier(ActionType::SatisfyRest, weights);
        let utility = (base_utility + urgency) * context * success;

        if best.is_none_or(|(u, _)| utility > u) {
            best = Some((utility, house.entity));
        }
    }
    best
}

/// Handles the arrival of a pop at housing to rest.
#[allow(clippy::collapsible_if)]
pub fn handle_arrival(
    pop_entity: Entity,
    target_entity: Entity,
    housing: &mut Query<&mut Housing>,
    commands: &mut Commands,
) {
    if let Ok(mut house) = housing.get_mut(target_entity) {
        if house.residents.len() < house.capacity {
            house.residents.push(pop_entity);
            commands
                .entity(pop_entity)
                .insert(crate::layer1::actions::AssignedTo {
                    entity: target_entity,
                    assignment_type: crate::layer1::actions::AssignmentType::HousingResident,
                });
        }
    }
}
