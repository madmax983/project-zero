use crate::layer1::traits::Trait;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;

/// Evaluates the desire to gossip.
#[must_use]
pub(crate) fn evaluate_gossip(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> (ActionType, f32, Option<Entity>) {
    let mut desire = 0.0;

    // Social need
    desire += (1.0 - data.needs.leisure) * 1.5;

    // Trait bonuses
    if let Some(traits) = data.traits.as_ref() {
        if traits.has(Trait::Greedy) || traits.has(Trait::Anxious) {
            desire += 0.5;
        }
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    // Find someone else to gossip with (closest pop).
    // In this simple iteration we evaluate taverns to socialize/gossip at.
    for candidate in &buffer.taverns {
        let context_score = crate::layer1::utility_types::calculate_context_score(
            data.pos,
            Some(candidate.pos),
            candidate.capacity,
            candidate.usage,
            &data.weights,
        );

        let total_score = desire * context_score;

        if total_score > best_score {
            best_score = total_score;
            best_target = Some(candidate.entity);
        }
    }

    (ActionType::Gossip, best_score, best_target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::psychology::traits::{Trait, Traits};
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_gossip_low_leisure() {
        let mut data = PopEvalData::test_instance();
        data.needs.leisure = 0.0;
        data.pos = GridPosition { x: 0, y: 0 };
        data.weights = UtilityWeights::default();

        let mut buffer = UtilityAIBuffer::default();
        buffer.taverns.push(ScorableCandidate {
            entity: Entity::from_raw(1),
            pos: GridPosition { x: 1, y: 1 },
            capacity: 10,
            usage: 0,
            score_bonus: 0.0,
            resource_type: None,
            item_type: None,
            is_advanced_tech: false,
        });

        let (action, score, target) = evaluate_gossip(&data, &buffer);
        assert_eq!(action, ActionType::Gossip);
        assert!(score > 0.0, "Should have a positive score for gossiping when leisure is 0");
        assert_eq!(target, Some(Entity::from_raw(1)));
    }

    #[test]
    fn test_evaluate_gossip_with_traits() {
        let mut data = PopEvalData::test_instance();
        data.needs.leisure = 1.0; // Fully satisfied leisure, normally no desire
        data.pos = GridPosition { x: 0, y: 0 };
        data.weights = UtilityWeights::default();

        let mut traits = Traits::default();
        traits.add(Trait::Greedy);
        data.traits = Some(traits);

        let mut buffer = UtilityAIBuffer::default();
        buffer.taverns.push(ScorableCandidate {
            entity: Entity::from_raw(2),
            pos: GridPosition { x: 1, y: 1 },
            capacity: 10,
            usage: 0,
            score_bonus: 0.0,
            resource_type: None,
            item_type: None,
            is_advanced_tech: false,
        });

        let (action, score, target) = evaluate_gossip(&data, &buffer);
        assert_eq!(action, ActionType::Gossip);
        assert!(score > 0.0, "Greedy trait should give base desire even if leisure is full");
        assert_eq!(target, Some(Entity::from_raw(2)));
    }
}
