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
        if pop_pos.x >= 0 && pop_pos.y >= 0 {
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
