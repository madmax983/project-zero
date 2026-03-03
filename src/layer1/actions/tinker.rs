use crate::layer1::skills::SkillType;
use crate::layer1::traits::Trait;
use crate::layer1::utility_eval_types::{PopEvalData, ScorableCandidate, evaluate_candidates};
use bevy_ecs::prelude::*;

/// Evaluates if a highly-skilled Pop feels compelled to tinker with a building.
///
/// Mechanics (Spec 199): Pops with Construction or Crafting skill >= 3 can select this action.
/// The `Obsessive` trait increases the likelihood.
#[must_use]
pub fn evaluate_tinker(
    data: &PopEvalData,
    candidates: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // 1. Check Skills
    let skills = data.skills.as_ref()?;
    let construction_level = skills.get_level(SkillType::Construction);
    let crafting_level = skills.get_level(SkillType::Crafting);
    let max_level = construction_level.max(crafting_level);

    if max_level < 3 {
        return None;
    }

    // 2. Base Score
    let base_score = 0.3;

    // 3. Modifiers
    let level_bonus = (max_level - 3) as f32 * 0.1;
    let trait_bonus = if data
        .traits
        .as_ref()
        .map_or(false, |t| t.0.contains(&Trait::Obsessive))
    {
        0.2
    } else {
        0.0
    };

    let utility = base_score + level_bonus + trait_bonus;

    // 4. Find best candidate
    evaluate_candidates(data.pos, &data.weights, candidates, utility)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::utility_eval_types::{PopEvalData, ScorableCandidate};
    use crate::layer1::utility_types::{PopAction, UtilityWeights};
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use std::collections::HashSet;
    use bevy_ecs::prelude::Entity;

    fn get_dummy_data(skills: Option<Skills>, traits: Option<Traits>) -> PopEvalData {
        PopEvalData {
            entity: Entity::from_raw(0),
            pos: GridPosition { x: 0, y: 0 },
            needs: Needs::default(),
            weights: UtilityWeights::default(),
            action: PopAction::default(),
            equipment: None,
            carrying: None,
            carrying_item: None,
            mental_state: None,
            drafted: None,
            faction_member: None,
            penal_labor: None,
            breakdown: None,
            traits,
            stress: 0.0,
            hobby_type: None,
            chemical_state: None,
            is_memetic_carrier: false,
            health: None,
            insulation: 0.0,
            skills,
        }
    }

    #[test]
    fn test_evaluate_tinker_low_skill() {
        let skills = Skills::default(); // 0 XP
        let data = get_dummy_data(Some(skills), None);
        let candidates = vec![ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 1, y: 0 })];

        let result = evaluate_tinker(&data, &candidates);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_tinker_high_skill() {
        let mut skills = Skills::default();
        // Level 3 requires 900 XP
        skills.add_xp(SkillType::Construction, 900.0);

        let data = get_dummy_data(Some(skills), None);
        let candidates = vec![ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 1, y: 0 })];

        let result = evaluate_tinker(&data, &candidates);
        assert!(result.is_some());
    }

    #[test]
    fn test_evaluate_tinker_obsessive_bonus() {
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Construction, 900.0); // Level 3 -> Base 0.3

        // Without obsessive
        let data_normal = get_dummy_data(Some(skills.clone()), None);
        let candidates = vec![ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 1, y: 0 })];

        let result_normal = evaluate_tinker(&data_normal, &candidates).unwrap().0;

        // With obsessive -> +0.2
        let traits = Traits(HashSet::from([Trait::Obsessive]));
        let data_obsessive = get_dummy_data(Some(skills), Some(traits));

        let result_obsessive = evaluate_tinker(&data_obsessive, &candidates).unwrap().0;

        assert!(result_obsessive > result_normal);
    }
}
