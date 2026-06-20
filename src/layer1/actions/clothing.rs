use crate::layer1::map::GridPosition;
use crate::layer1::resources::ColonyResources;
use crate::layer1::utility_eval_types::*;
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Evaluates if a pop should fetch clothing.
///
/// # The "Shiver Factor"
///
/// Pops don't just upgrade clothes for fashion; they do it to survive.
/// This evaluator checks:
///
/// 1.  **Nakedness:** If `insulation == 0.0`, urgency is high (0.95).
/// 2.  **Hypothermia Risk:** If the local temperature is below what the current
///     clothes can handle (`insulation * -30 + 10`), urgency becomes critical (0.99),
///     prompting an immediate upgrade even if they are already dressed.
#[must_use]
pub(crate) fn evaluate_fetch_clothing(
    pop_pos: GridPosition,
    current_insulation: f32,
    resources: &ColonyResources,
    stockpiles: &[ScorableCandidate],
    temperature_grid: Option<&crate::layer1::temperature::TemperatureGrid>,
) -> Option<(f32, Entity)> {
    let mut score = 0.0;

    if current_insulation == 0.0 {
        score = 0.95; // Need clothes!
    } else if let Some(grid) = temperature_grid {
        // Check if freezing despite clothes
        #[allow(clippy::cast_sign_loss)]
        let temp = grid.get(pop_pos.x as usize, pop_pos.y as usize);
        let safe_temp = current_insulation.mul_add(-30.0, 10.0);
        if temp < safe_temp {
            // If insulation is already high (e.g. >= 2.0), fetching won't help unless we have super-parka.
            if current_insulation < 2.0 {
                score = 0.99; // Upgrade needed immediately!
            }
        }
    }

    if score == 0.0 {
        return None;
    }

    // If no clothing available in colony, can't fetch
    if resources.clothing < 1.0 {
        return None;
    }

    // Use evaluate_candidates with default weights (clothing is basic need, traits matter less)
    let weights = UtilityWeights::default();
    evaluate_candidates(pop_pos, &weights, stockpiles, score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::temperature::TemperatureGrid;
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use bevy_ecs::prelude::Entity;

    #[test]
    fn should_return_none_when_resources_clothing_is_zero() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let resources = ColonyResources {
            clothing: 0.0,
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 1, y: 0 },
        )];

        let result = evaluate_fetch_clothing(pop_pos, 0.0, &resources, &candidates, None);
        assert_eq!(result, None, "Expected None when no clothing is available");
    }

    #[test]
    fn should_return_some_when_naked_and_clothing_available() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let resources = ColonyResources {
            clothing: 5.0,
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 1, y: 0 },
        )];

        let result = evaluate_fetch_clothing(pop_pos, 0.0, &resources, &candidates, None);
        assert!(
            result.is_some(),
            "Expected Some when naked and clothing is available"
        );
        let (score, _) = result.unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn should_return_none_when_dressed_and_no_temperature_grid() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let resources = ColonyResources {
            clothing: 5.0,
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 1, y: 0 },
        )];

        // Insulation > 0.0 and no temperature_grid
        let result = evaluate_fetch_clothing(pop_pos, 1.0, &resources, &candidates, None);
        assert_eq!(result, None, "Expected None when dressed and no temp grid");
    }

    #[test]
    fn should_return_some_when_freezing_despite_clothes() {
        let pop_pos = GridPosition { x: 5, y: 5 };
        let resources = ColonyResources {
            clothing: 5.0,
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 1, y: 0 },
        )];

        let mut grid = TemperatureGrid::new(10, 10, 0.0);
        // Current insulation is 1.0, safe temp is 1.0 * -30.0 + 10.0 = -20.0
        // Set temp to -25.0 (below safe temp)
        grid.set(5, 5, -25.0);

        let result = evaluate_fetch_clothing(pop_pos, 1.0, &resources, &candidates, Some(&grid));
        assert!(
            result.is_some(),
            "Expected Some when freezing despite clothes"
        );
        let (score, _) = result.unwrap();
        assert!(score > 0.0);
    }

    #[test]
    fn should_return_none_when_freezing_but_insulation_already_maxed() {
        let pop_pos = GridPosition { x: 5, y: 5 };
        let resources = ColonyResources {
            clothing: 5.0,
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 1, y: 0 },
        )];

        let mut grid = TemperatureGrid::new(10, 10, 0.0);
        // Current insulation is 2.5 (>= 2.0 max).
        // Safe temp is 2.5 * -30.0 + 10.0 = -65.0
        // Set temp to -70.0 (below safe temp, but insulation is maxed out)
        grid.set(5, 5, -70.0);

        let result = evaluate_fetch_clothing(pop_pos, 2.5, &resources, &candidates, Some(&grid));
        assert_eq!(
            result, None,
            "Expected None when insulation is already high (>= 2.0)"
        );
    }

    #[test]
    fn should_return_none_when_temperature_is_safe() {
        let pop_pos = GridPosition { x: 5, y: 5 };
        let resources = ColonyResources {
            clothing: 5.0,
            ..Default::default()
        };
        let candidates = vec![ScorableCandidate::new(
            Entity::from_raw(1),
            GridPosition { x: 1, y: 0 },
        )];

        let mut grid = TemperatureGrid::new(10, 10, 0.0);
        // Current insulation is 1.0, safe temp is 1.0 * -30.0 + 10.0 = -20.0
        // Set temp to 0.0 (safe)
        grid.set(5, 5, 0.0);

        let result = evaluate_fetch_clothing(pop_pos, 1.0, &resources, &candidates, Some(&grid));
        assert_eq!(result, None, "Expected None when temperature is safe");
    }
}
