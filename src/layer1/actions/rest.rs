use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates the utility of resting at a house.
#[must_use]
pub(crate) fn evaluate_satisfy_rest(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    housing: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // Urgency based on rest level
    let urgency = 1.0 - needs.rest;
    evaluate_candidates(pop_pos, weights, housing, 0.5 + urgency)
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
