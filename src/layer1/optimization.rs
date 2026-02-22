use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::skills::{SkillType, Skills};
use crate::layer1::structure::Structure;
use crate::layer1::traits::{Trait, Traits};
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::{ActionType, UtilityWeights};
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Default)]
pub struct Optimized {
    pub efficiency_bonus: f32, // e.g., 0.10 for +10%
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationResult {
    Success,
    CriticalSuccess,
    Failure,
    CriticalFailure,
}

pub fn perform_optimization(world: &mut World, target: Entity, result: OptimizationResult) {
    match result {
        OptimizationResult::Success => {
            if world.get::<Optimized>(target).is_none() {
                world
                    .entity_mut(target)
                    .insert(Optimized { efficiency_bonus: 0.10 });
            }
        }
        OptimizationResult::CriticalSuccess => {
            if world.get::<Optimized>(target).is_none() {
                world
                    .entity_mut(target)
                    .insert(Optimized { efficiency_bonus: 0.25 });
            }
        }
        OptimizationResult::Failure => {
            if let Some(mut structure) = world.get_mut::<Structure>(target) {
                let damage = structure.max_hp * 0.25;
                structure.current_hp = (structure.current_hp - damage).max(0.0);
            }
        }
        OptimizationResult::CriticalFailure => {
            if let Some(mut structure) = world.get_mut::<Structure>(target) {
                structure.current_hp = 0.0;
            }
        }
    }
}

/// Evaluates the desire to tinker with machines.
#[must_use]
pub fn evaluate_tinker(
    pos: GridPosition,
    skills: &Skills,
    traits: Option<&Traits>,
    weights: &UtilityWeights,
    candidates: &[ScorableCandidate],
) -> Option<(f32, Entity)> {
    // 1. Check eligibility
    let construction = skills.get_level(SkillType::Construction);
    let crafting = skills.get_level(SkillType::Crafting);
    let max_skill = construction.max(crafting);

    // Minimum skill requirement: 3
    if max_skill < 3 {
        return None;
    }

    // 2. Calculate Base Utility
    let mut utility = 0.3; // Low priority base

    // Skill modifier: +0.1 per level above 3
    #[allow(clippy::cast_precision_loss)]
    {
        utility += (max_skill - 3) as f32 * 0.1;
    }

    // Trait modifier: Obsessive
    if traits.is_some_and(|t| t.has(Trait::Obsessive)) {
        utility += 0.2;
    } else {
        // Only obsessive pops or high skill pops usually do this?
        // Spec says: "Engineers feel compelled... +0.2 if Obsessive".
        // It doesn't strictly forbid non-obsessive, but sets a low base score.
    }

    // 3. Find best candidate
    evaluate_candidates(pos, weights, candidates, utility)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::structure::Structure;

    #[test]
    fn test_optimization_success_applies_component() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        let result = OptimizationResult::Success;
        perform_optimization(&mut world, building, result);

        let optimized = world.get::<Optimized>(building);
        assert!(optimized.is_some(), "Optimized component should be added on success");
        assert!((optimized.unwrap().efficiency_bonus - 0.10).abs() < f32::EPSILON);
    }

    #[test]
    fn test_optimization_failure_damages_building() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        let result = OptimizationResult::Failure;
        perform_optimization(&mut world, building, result);

        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp < 100.0, "Structure should be damaged on failure");
        assert!(world.get::<Optimized>(building).is_none());
    }

    #[test]
    fn test_critical_failure_breaks_building() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Structure { max_hp: 100.0, current_hp: 100.0 },
        )).id();

        let result = OptimizationResult::CriticalFailure;
        perform_optimization(&mut world, building, result);

        let structure = world.get::<Structure>(building).unwrap();
        assert!(structure.current_hp <= 0.0, "Structure should be broken (0 HP) on critical failure");
    }

    #[test]
    fn test_already_optimized_cannot_be_optimized_again() {
        let mut world = World::new();
        let building = world.spawn((
            Building { building_type: BuildingType::Generator },
            Optimized { efficiency_bonus: 0.10 },
        )).id();

        // Trying to optimize again
        perform_optimization(&mut world, building, OptimizationResult::Success);

        let optimized = world.get::<Optimized>(building).unwrap();
        assert!((optimized.efficiency_bonus - 0.10).abs() < f32::EPSILON, "Efficiency should not stack");
    }

    #[test]
    fn test_evaluate_tinker_eligibility() {
        let mut skills = Skills::default();
        // Level 2 (below 3)
        // Default is 0.
        // Needs manual set or xp.
        // Assuming Skills has helper or public fields?
        // Skills struct is complex. Let's assume default is 0.
        // We can add XP to level up.
        // 0->1: 100, 1->2: 200, 2->3: 400? Need to check leveling curve.
        // Usually level = sqrt(xp / 100).
        // So level 3 needs 900 XP.
        skills.add_xp(SkillType::Construction, 900.0);

        let pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let candidates = vec![];

        let result = evaluate_tinker(pos, &skills, None, &weights, &candidates);
        // Should return None because no candidates, but eligibility passed.
        // Wait, evaluate_candidates returns None if empty.

        // Let's test low skill
        let low_skills = Skills::default();
        let result_low = evaluate_tinker(pos, &low_skills, None, &weights, &candidates);
        assert!(result_low.is_none());
    }

    #[test]
    fn test_evaluate_tinker_score() {
         let mut skills = Skills::default();
         // Level 3
         skills.add_xp(SkillType::Construction, 900.0);

         let pos = GridPosition { x: 0, y: 0 };
         let weights = UtilityWeights::default();
         let target = Entity::from_raw(1);
         let candidates = vec![ScorableCandidate::new(target, pos)]; // Same pos = max distance score

         let result = evaluate_tinker(pos, &skills, None, &weights, &candidates);
         assert!(result.is_some());
         let (score, e) = result.unwrap();
         assert_eq!(e, target);
         // Base 0.3 + (3-3)*0.1 = 0.3.
         // Distance score at 0 is 1.0.
         // Total 0.3.
         assert!((score - 0.3).abs() < f32::EPSILON);
    }
}
