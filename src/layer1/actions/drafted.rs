use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::ActionType;
use bevy_ecs::prelude::*;

/// Evaluates actions for a drafted pop (combat).
pub(crate) fn evaluate_drafted_behavior(
    data: &PopEvalData,
    buffer: &UtilityAIBuffer,
) -> Option<(ActionType, f32, Option<Entity>)> {
    data.drafted?;

    let mut best_action = ActionType::Idle;
    let mut best_utility = 0.9; // Just stand there ready
    let mut best_target = None;

    // Helper to evaluate fight
    let evaluate_fight = || -> Option<(f32, Entity)> {
        // Find nearest enemy
        let mut best_fight_target = None;
        let mut min_dist = f32::MAX;

        for candidate in &buffer.enemies {
            #[allow(clippy::cast_precision_loss)]
            let dist = data.pos.distance_chebyshev(candidate.pos) as f32;
            if dist < min_dist {
                min_dist = dist;
                best_fight_target = Some(candidate.entity);
            }
        }

        best_fight_target.map(|target| (0.95 - (min_dist * 0.01).min(0.5), target))
    };

    if let Some((utility, target)) = evaluate_fight() {
        best_action = ActionType::Fight;
        best_utility = utility;
        best_target = Some(target);
    }

    Some((best_action, best_utility, best_target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_evaluate_drafted_behavior() {
        let mut data = PopEvalData::test_instance();
        data.pos = GridPosition { x: 0, y: 0 };
        let mut buffer = UtilityAIBuffer::default();

        // Not drafted
        assert_eq!(evaluate_drafted_behavior(&data, &buffer), None);

        // Drafted, no enemies
        data.drafted = Some(crate::layer1::combat::Drafted);
        let result_no_enemy = evaluate_drafted_behavior(&data, &buffer);
        assert_eq!(result_no_enemy, Some((ActionType::Idle, 0.9, None)));

        // Drafted, with enemy
        buffer.enemies.push(ScorableCandidate {
            entity: Entity::from_raw(1),
            pos: GridPosition { x: 5, y: 5 },
            capacity: 10,
            usage: 0,
            score_bonus: 0.0,
            resource_type: None,
            item_type: None,
            is_advanced_tech: false,
        });

        let result_with_enemy = evaluate_drafted_behavior(&data, &buffer);
        assert!(result_with_enemy.is_some());
        let (action, score, target) = result_with_enemy.unwrap();
        assert_eq!(action, ActionType::Fight);
        assert!(score > 0.0);
        assert_eq!(target, Some(Entity::from_raw(1)));
    }
}
