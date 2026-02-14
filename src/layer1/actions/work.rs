use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::map::GridPosition;
use crate::layer1::utility_types::{ActionEvaluator, ActionType, PopEvalData, UtilityWeights, WorldContext};
use crate::layer1::utility_types::{calculate_context_score, calculate_success_modifier};
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;

/// Evaluates the utility of performing designated work (Mining, Building, etc.).
///
/// This checks all active [`Designation`]s (like "Mine this rock") and calculates
/// a score based on distance and the Pop's work ethic.
///
/// **Note:** This function explicitly filters OUT [`DesignationType::Repair`] tasks,
/// as those are handled separately by [`crate::layer1::actions::repair::evaluate_repair`] to prioritize maintenance.
///
/// # Returns
/// A tuple `(utility, designation_entity)` if a suitable task is found.
#[must_use]
pub fn evaluate_work<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: impl Iterator<Item = (Entity, &'a GridPosition, &'a Designation)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;

    // Base utility for working (could depend on traits later)
    let base_utility = 0.5;

    for (entity, pos, des) in designations {
        // Skip Repair designations (handled by evaluate_repair)
        if des.designation_type == DesignationType::Repair {
            continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity 1 (one worker per tile usually)
            0, // Occupied 0 (simplified for now)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Work, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}

/// Evaluator for the Work action.
pub struct WorkEvaluator {
    system_state: SystemState<Query<'static, 'static, (Entity, &'static GridPosition, &'static Designation)>>,
}

impl WorkEvaluator {
    /// Creates a new `WorkEvaluator`.
    pub fn new(world: &mut World) -> Self {
        Self {
            system_state: SystemState::new(world),
        }
    }
}

impl ActionEvaluator for WorkEvaluator {
    fn evaluate(
        &mut self,
        world: &mut World,
        data: &PopEvalData,
        context: &WorldContext,
    ) -> Option<(ActionType, f32, Option<Entity>)> {
        // Check if striking
        let is_striking = context.factions.as_ref().is_some_and(|map| {
            data.faction_member.as_ref().is_some_and(|member| {
                member.faction_id.is_some_and(|fid| {
                    map.get(&fid).is_some_and(|data| {
                        data.state == crate::layer1::factions::FactionState::Striking
                    })
                })
            })
        });

        if is_striking {
            return None;
        }

        let designations_query = self.system_state.get(world);

        if let Some((utility, target)) = evaluate_work(
            &data.pos,
            &data.weights,
            designations_query.iter(),
        ) {
            let penalty =
                crate::layer1::taboo::evaluate_taboo_penalty(ActionType::Work, context.taboo);
            return Some((ActionType::Work, utility + penalty, Some(target)));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_work_no_designations() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let designations: Vec<(Entity, &GridPosition, &Designation)> = vec![];

        let result = evaluate_work(&pop_pos, &weights, designations.into_iter());
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_work_ignores_repair() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let des = Designation {
            designation_type: DesignationType::Repair,
        };
        let pos = GridPosition { x: 5, y: 5 };
        let entity = Entity::from_raw(1);

        let designations = vec![(entity, &pos, &des)];

        let result = evaluate_work(&pop_pos, &weights, designations.into_iter());
        assert!(result.is_none(), "Should ignore Repair designations");
    }

    #[test]
    fn test_evaluate_work_finds_mining() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let des = Designation {
            designation_type: DesignationType::Mine,
        };
        let pos = GridPosition { x: 2, y: 0 };
        let entity = Entity::from_raw(1);

        let designations = vec![(entity, &pos, &des)];

        let result = evaluate_work(&pop_pos, &weights, designations.into_iter());
        assert!(result.is_some());
        let (utility, best_entity) = result.unwrap();
        assert_eq!(best_entity, entity);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_work_prioritizes_distance() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let des_mine = Designation {
            designation_type: DesignationType::Mine,
        };

        let entity_close = Entity::from_raw(1);
        let pos_close = GridPosition { x: 2, y: 0 };

        let entity_far = Entity::from_raw(2);
        let pos_far = GridPosition { x: 10, y: 0 };

        let designations = vec![
            (entity_far, &pos_far, &des_mine),
            (entity_close, &pos_close, &des_mine),
        ];

        let result = evaluate_work(&pop_pos, &weights, designations.into_iter());
        assert!(result.is_some());
        let (_, best_entity) = result.unwrap();
        assert_eq!(best_entity, entity_close);
    }
}
