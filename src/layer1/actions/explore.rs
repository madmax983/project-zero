use crate::layer1::map::GridPosition;
use crate::layer1::science::Anomaly;
use crate::layer1::utility_ai::math::{calculate_context_score, calculate_success_modifier};
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

/// Evaluates the utility of exploring an [`Anomaly`].
///
/// Anomalies (ruins, mysterious plants) provide unique rewards or trigger events.
/// Exploration is a medium-priority task (0.55 utility) - slightly better than
/// regular work but less critical than hauling food or healing.
#[must_use]
pub fn evaluate_explore<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    anomalies: impl Iterator<Item = (Entity, &'a GridPosition, &'a Anomaly)>,
) -> Option<(f32, Entity)> {
    let mut best: Option<(f32, Entity)> = None;
    let base_utility = 0.55;

    for (entity, pos, _) in anomalies {
        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity (simplified)
            0, // Occupied (simplified)
            weights,
        );

        let success = calculate_success_modifier(ActionType::Explore, weights);
        let utility = base_utility * context * success;

        if best.is_none_or(|(best_u, _)| utility > best_u) {
            best = Some((utility, entity));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::science::{Anomaly, AnomalyType};
    use crate::layer1::utility_ai::UtilityWeights;

    #[test]
    fn test_evaluate_explore_no_anomalies() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let anomalies: Vec<(Entity, &GridPosition, &Anomaly)> = vec![];

        let result = evaluate_explore(&pop_pos, &weights, anomalies.into_iter());
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_explore_finds_closest_anomaly() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        let anomaly_close = Anomaly {
            anomaly_type: AnomalyType::Ruins,
            reward_amount: 10.0,
        };
        let anomaly_far = Anomaly {
            anomaly_type: AnomalyType::Geode,
            reward_amount: 10.0,
        };
        let pos_close = GridPosition { x: 2, y: 0 };
        let pos_far = GridPosition { x: 10, y: 0 };
        let entity_close = Entity::from_raw(1);
        let entity_far = Entity::from_raw(2);

        let anomalies = vec![
            (entity_far, &pos_far, &anomaly_far),
            (entity_close, &pos_close, &anomaly_close),
        ];

        let result = evaluate_explore(&pop_pos, &weights, anomalies.into_iter());
        assert!(result.is_some());
        let (_, best_entity) = result.unwrap();
        assert_eq!(best_entity, entity_close);
    }

    #[test]
    fn test_evaluate_explore_handles_zero_distance() {
        let pop_pos = GridPosition { x: 5, y: 5 };
        let weights = UtilityWeights::default();

        let anomaly = Anomaly {
            anomaly_type: AnomalyType::Ruins,
            reward_amount: 10.0,
        };
        let pos = GridPosition { x: 5, y: 5 };
        let entity = Entity::from_raw(1);

        let anomalies = vec![(entity, &pos, &anomaly)];

        let result = evaluate_explore(&pop_pos, &weights, anomalies.into_iter());
        assert!(result.is_some());
        let (utility, best_entity) = result.unwrap();
        assert_eq!(best_entity, entity);
        assert!(utility > 0.0);
    }
}
