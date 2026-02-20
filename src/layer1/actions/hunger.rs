use super::{AssignedTo, AssignmentType};
use crate::layer1::farm::Farm;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::pop::Job;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::{UtilityWeights, need_response_curve};
use bevy_ecs::prelude::*;

/// Evaluates the utility of satisfying hunger at available farms.
#[must_use]
pub(crate) fn evaluate_satisfy_hunger(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    farms: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    let urgency = need_response_curve(needs.hunger);
    evaluate_candidates(pop_pos, weights, farms, urgency)
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
            commands.entity(pop_entity).insert((
                AssignedTo {
                    entity: target_entity,
                    assignment_type: AssignmentType::FarmWorker,
                },
                Job {
                    workplace: target_entity,
                    job_type: AssignmentType::FarmWorker,
                },
            ));
        }
    }
}
