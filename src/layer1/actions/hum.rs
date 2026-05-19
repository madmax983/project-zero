use crate::layer1::traits::Trait;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::{calculate_context_score, ActionType};
use bevy_ecs::prelude::*;

/// Evaluates the desire to listen to "The Hum".
#[must_use]
pub(crate) fn evaluate_listen_to_hum(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> (ActionType, f32, Option<Entity>) {
    // 1. Check Trait (Early Exit)
    if !data
        .traits
        .as_ref()
        .is_some_and(|t| t.has(Trait::Sensitive))
    {
        return (ActionType::ListenToTheHum, 0.0, None);
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    for candidate in &buffer.hum_sources {
        let intensity = candidate.score_bonus;
        let desire = intensity * (1.0 + data.stress);

        let context_score = calculate_context_score(
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

    (ActionType::ListenToTheHum, best_score, best_target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::psychology::traits::Traits;
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_listen_to_hum() {
        let mut data = PopEvalData::test_instance();
        data.pos = GridPosition { x: 0, y: 0 };
        data.weights = UtilityWeights::default();

        let mut buffer = UtilityAIBuffer::default();
        buffer.hum_sources.push(ScorableCandidate {
            entity: Entity::from_raw(1),
            pos: GridPosition { x: 1, y: 1 },
            capacity: 10,
            usage: 0,
            score_bonus: 2.0, // Intensity
            resource_type: None,
            item_type: None,
            is_advanced_tech: false,
        });

        // No sensitive trait => Score 0
        let (action, score, target) = evaluate_listen_to_hum(&data, &buffer);
        assert_eq!(action, ActionType::ListenToTheHum);
        assert_eq!(score, 0.0);
        assert_eq!(target, None);

        // Add Sensitive trait
        let mut traits = Traits::default();
        traits.add(Trait::Sensitive);
        data.traits = Some(traits);

        let (action2, score2, target2) = evaluate_listen_to_hum(&data, &buffer);
        assert_eq!(action2, ActionType::ListenToTheHum);
        assert!(score2 > 0.0, "Score should be positive with sensitive trait");
        assert_eq!(target2, Some(Entity::from_raw(1)));
    }
}
