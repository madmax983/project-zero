use crate::layer1::map::GridPosition;
use bevy_ecs::prelude::*;

/// Evaluates the utility of fighting an enemy.
///
/// Returns `Some((utility, target))` if drafted and enemies are present.
/// The score is high (0.95 base) minus a distance penalty, prioritizing closest enemies.
pub fn evaluate_fight_action<'a>(
    drafted: bool,
    pop_pos: &GridPosition,
    enemies: impl Iterator<Item = (Entity, &'a GridPosition)>,
) -> Option<(f32, Entity)> {
    if !drafted {
        return None;
    }

    // Find nearest enemy
    let mut best_target = None;
    let mut min_dist = f32::MAX;

    for (entity, pos) in enemies {
        #[allow(clippy::cast_precision_loss)]
        let dist = pop_pos.distance_chebyshev(*pos) as f32;
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(entity);
        }
    }

    if let Some(target) = best_target {
        // High score for combat when drafted
        // Distance penalty applies, but base score is high (e.g., 0.95)
        // 0.95 - (dist * 0.01).min(0.5) ensures high priority even at range,
        // but prefers closer targets.
        return Some((0.95 - (min_dist * 0.01).min(0.5), target));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_evaluate_fight_action_not_drafted() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let enemies = vec![];
        let result = evaluate_fight_action(false, &pop_pos, enemies.into_iter());
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_fight_action_drafted_no_enemies() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let enemies = vec![];
        let result = evaluate_fight_action(true, &pop_pos, enemies.into_iter());
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_fight_action_drafted_with_enemies() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let enemy_pos = GridPosition { x: 5, y: 0 };
        let enemies = vec![(Entity::from_raw(1), &enemy_pos)];

        let result = evaluate_fight_action(true, &pop_pos, enemies.into_iter());
        assert!(result.is_some());
        assert_eq!(result.unwrap().1, Entity::from_raw(1));
    }
}
