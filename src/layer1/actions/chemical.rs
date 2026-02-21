use crate::layer1::chemical::{ChemicalState, ChemicalType, consume_chemical};
use crate::layer1::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::utility_eval_types::{ScorableCandidate, evaluate_candidates};
use crate::layer1::utility_types::UtilityWeights;
use bevy_ecs::prelude::*;

/// Handles the arrival of a pop at a chemical item.
pub fn handle_consume_chemical_arrival(
    world: &mut World,
    pop_entity: Entity,
    target_entity: Entity,
) {
    // 1. Verify item exists and is valid
    let item_type = if let Some(item) = world.get::<Item>(target_entity) {
        match item.item_type {
            ItemType::Stim => Some(ChemicalType::Stim),
            ItemType::Sedative => Some(ChemicalType::Sedative),
            _ => None,
        }
    } else {
        None
    };

    if let Some(chem_type) = item_type {
        // 2. Consume logic
        consume_chemical(world, pop_entity, chem_type);

        // 3. Destroy item
        world.despawn(target_entity);
    }
}

/// Evaluates the utility of consuming chemicals.
pub fn evaluate_consume_chemical(
    pop_pos: GridPosition,
    needs: &Needs,
    weights: &UtilityWeights,
    chemical_state: Option<&ChemicalState>,
    stress: f32,                         // Normalized 0-1
    item_entities: &[ScorableCandidate], // buffer.item_entities
) -> Option<(f32, Entity)> {
    // 1. Determine Desire
    let mut desire_stim = 0.05_f32; // Base desire (curiosity/habit)
    let mut desire_sedative = 0.05_f32;

    // Withdrawal (Panic!)
    if let Some(state) = chemical_state {
        if state.is_in_withdrawal(ChemicalType::Stim) {
            desire_stim = 1.0;
        }
        if state.is_in_withdrawal(ChemicalType::Sedative) {
            desire_sedative = 1.0;
        }
    }

    // Needs based desire
    if (desire_stim - 1.0).abs() > f32::EPSILON {
        if needs.rest < 0.2 {
            // Very tired
            desire_stim += 0.5;
        }
    }

    if (desire_sedative - 1.0).abs() > f32::EPSILON {
        if stress > 0.8 {
            // High stress
            desire_sedative += 0.5;
        }
    }

    // Filter candidates
    // We need to pass references to ScorableCandidate to evaluate_candidates,
    // but we can't easily construct a vector of references from a vector of owned values
    // without lifetime issues if we are filtering.
    // However, `evaluate_candidates` takes `&[ScorableCandidate]`.
    // So we can collect filtered candidates into a Vec and pass that slice.

    let mut candidates_stim = Vec::new();
    let mut candidates_sedative = Vec::new();

    for candidate in item_entities {
        if let Some(item_type) = &candidate.item_type {
            match item_type {
                ItemType::Stim => candidates_stim.push(candidate.clone()),
                ItemType::Sedative => candidates_sedative.push(candidate.clone()),
                _ => {}
            }
        }
    }

    let mut best_score = 0.0;
    let mut best_target = None;

    // Check Stims
    if desire_stim > 0.1 {
        if let Some((score, target)) =
            evaluate_candidates(pop_pos, weights, &candidates_stim, desire_stim)
        {
            if score > best_score {
                best_score = score;
                best_target = Some(target);
            }
        }
    }

    // Check Sedatives
    if desire_sedative > 0.1 {
        if let Some((score, target)) =
            evaluate_candidates(pop_pos, weights, &candidates_sedative, desire_sedative)
        {
            if score > best_score {
                best_score = score;
                best_target = Some(target);
            }
        }
    }

    if let Some(target) = best_target {
        Some((best_score, target))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::chemical::ChemicalState;
    use crate::layer1::chemical::{Addiction, ChemicalType};
    use crate::layer1::items::ItemType;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::utility_eval_types::ScorableCandidate;
    use crate::layer1::utility_types::UtilityWeights;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_evaluate_consume_chemical_withdrawal() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs::default();
        let weights = UtilityWeights::default();
        let mut addictions = Vec::new();
        addictions.push(Addiction {
            chemical: ChemicalType::Stim,
            severity: 0.8,
            last_consumed_tick: 0,
            withdrawal_threshold: 100,
            in_withdrawal: true,
        });

        let state = ChemicalState {
            active_effects: vec![],
            addictions,
        };

        let mut item = ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 0, y: 0 });
        item.item_type = Some(ItemType::Stim);

        let result =
            evaluate_consume_chemical(pop_pos, &needs, &weights, Some(&state), 0.0, &[item]);

        assert!(result.is_some());
        let (score, _) = result.unwrap();
        assert!(score >= 1.0, "Withdrawal should produce high score");
    }

    #[test]
    fn test_evaluate_consume_chemical_needs() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs {
            rest: 0.1, // Very tired
            ..Default::default()
        };
        let weights = UtilityWeights::default();

        let mut item = ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 0, y: 0 });
        item.item_type = Some(ItemType::Stim);

        let result = evaluate_consume_chemical(pop_pos, &needs, &weights, None, 0.0, &[item]);

        assert!(result.is_some());
        let (score, _) = result.unwrap();
        assert!(score > 0.5, "Tired pop should want Stim");
    }

    #[test]
    fn test_evaluate_consume_chemical_stress() {
        let pop_pos = GridPosition { x: 0, y: 0 };
        let needs = Needs::default();
        let weights = UtilityWeights::default();

        let mut item = ScorableCandidate::new(Entity::from_raw(1), GridPosition { x: 0, y: 0 });
        item.item_type = Some(ItemType::Sedative);

        let result = evaluate_consume_chemical(
            pop_pos,
            &needs,
            &weights,
            None,
            0.9, // High stress
            &[item],
        );

        assert!(result.is_some());
        let (score, _) = result.unwrap();
        assert!(score > 0.5, "Stressed pop should want Sedative");
    }
}
