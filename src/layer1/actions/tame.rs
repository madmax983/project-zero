use bevy_ecs::prelude::*;
use crate::layer1::designation::{DesignationType};
use crate::layer1::map::GridPosition;
use crate::layer1::skills::{Skills, SkillType};
use crate::layer1::utility_ai::math::{calculate_context_score, calculate_success_modifier};
use crate::layer1::utility_ai::types::{ActionType, UtilityWeights};

/// Evaluates the Tame action for a given Pop.
///
/// Returns `Some((utility_score, target_entity))` if a valid target is found.
pub fn evaluate_tame(
    pop_pos: &GridPosition,
    skills: Option<&Skills>,
    weights: &UtilityWeights,
    designations: &[(Entity, GridPosition, DesignationType)],
    fauna_positions: &[GridPosition],
) -> Option<(f32, Entity)> {
    let mut best_score = 0.0;
    let mut best_target = None;

    // Iterate all designations looking for Tame
    for (designation_entity, designation_pos, designation_type) in designations {
        if *designation_type != DesignationType::Tame {
            continue;
        }

        // Check if there is actually a fauna at this location
        let has_fauna = fauna_positions.iter().any(|pos| *pos == *designation_pos);

        if !has_fauna {
            continue;
        }

        let distance_score = calculate_context_score(
            *pop_pos,
            Some(*designation_pos),
            1, // capacity (1 tamer per animal)
            0, // occupied (assume available)
            weights,
        );

        // Skill factor
        let skill_level = skills.map_or(0.0, |s| s.get_efficiency(SkillType::Husbandry));
        let skill_factor = 0.5 + (skill_level * 0.1).min(0.5); // 0.5 to 1.0

        let success_mod = calculate_success_modifier(ActionType::Tame, weights);

        let score = distance_score * skill_factor * success_mod;

        if score > best_score {
            best_score = score;
            best_target = Some(*designation_entity);
        }
    }

    best_target.map(|target| (best_score, target))
}
