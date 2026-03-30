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
    use crate::layer1::traits::Traits;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_types::{ActionType, PopAction};
    use crate::layer1::utility_types::UtilityWeights;

    #[test]
    fn test_evaluate_listen_to_hum() {
        let mut world = World::new();
        let target_entity = world.spawn_empty().id();
        let pop_entity = world.spawn_empty().id();

        let mut traits = Traits::default();

        let data = PopEvalData {
            entity: pop_entity,
            pos: GridPosition { x: 0, y: 0 },
            needs: Needs::default(),
            weights: UtilityWeights::default(),
            action: PopAction::default(),
            equipment: None,
            carrying: None,
            carrying_item: None,
            carrying_item_type: None,
            mental_state: None,
            drafted: None,
            faction_member: None,
            penal_labor: None,
            breakdown: None,
            traits: Some(traits.clone()),
            stress: 0.5,
            hobby_type: None,
            chemical_state: None,
            is_memetic_carrier: false,
            health: None,
            job: None,
            insulation: 0.0,
        };

        let mut buffer = UtilityAIBuffer::default();
        buffer.hum_sources.push(ScorableCandidate {
            entity: target_entity,
            pos: GridPosition { x: 5, y: 5 },
            capacity: 10,
            usage: 0,
            score_bonus: 5.0,
            item_type: None,
            resource_type: None,
        });

        // Test without Sensitive trait (should return 0.0)
        let (action, score, target) = evaluate_listen_to_hum(&data, &buffer);
        assert_eq!(action, ActionType::ListenToTheHum);
        assert_eq!(score, 0.0);
        assert_eq!(target, None);

        // Add Sensitive trait
        traits.add(Trait::Sensitive);
        let data_sensitive = PopEvalData {
            entity: pop_entity,
            pos: GridPosition { x: 0, y: 0 },
            needs: Needs::default(),
            weights: UtilityWeights::default(),
            action: PopAction::default(),
            equipment: None,
            carrying: None,
            carrying_item: None,
            carrying_item_type: None,
            mental_state: None,
            drafted: None,
            faction_member: None,
            penal_labor: None,
            breakdown: None,
            traits: Some(traits),
            stress: 0.5,
            hobby_type: None,
            chemical_state: None,
            is_memetic_carrier: false,
            health: None,
            job: None,
            insulation: 0.0,
        };

        // Test with Sensitive trait
        let (action_sens, score_sens, target_sens) = evaluate_listen_to_hum(&data_sensitive, &buffer);
        assert_eq!(action_sens, ActionType::ListenToTheHum);
        assert!(score_sens > 0.0);
        assert_eq!(target_sens, Some(target_entity));
    }
}
